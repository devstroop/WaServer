//! Maintenance tasks — staging janitor + browser availability (#46 #47)

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tracing::{info, warn};

/// Browser (Chrome/Chromium) availability for operators (#47).
/// Mirrors chromiumoxide's own detection so the warning matches reality.
pub fn detect_browser() -> Result<PathBuf, String> {
    chromiumoxide::detection::default_executable(
        chromiumoxide::detection::DetectionOptions::default(),
    )
}

/// Spawn the hourly staging janitor; also runs once at startup (#46).
/// Best-effort: individual failures are logged, never propagated.
pub fn start_staging_janitor(dir: PathBuf, ttl_hours: u64) -> tokio::task::JoinHandle<()> {
    info!(dir = %dir.display(), ttl_hours, "staging janitor started");
    tokio::spawn(async move {
        // run immediately at boot, then hourly
        cleanup_once(&dir, cutoff_for(ttl_hours)).await;
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            cleanup_once(&dir, cutoff_for(ttl_hours)).await;
        }
    })
}

fn cutoff_for(ttl_hours: u64) -> SystemTime {
    SystemTime::now()
        .checked_sub(Duration::from_secs(ttl_hours * 3600))
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

async fn cleanup_once(dir: &Path, cutoff: SystemTime) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return; // dir may not exist yet — nothing to clean
    };
    let mut removed = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = entry.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let stale = meta.modified().map(|m| m < cutoff).unwrap_or(false);
        if stale {
            match std::fs::remove_file(&path) {
                Ok(()) => removed += 1,
                Err(e) => {
                    warn!(path = %path.display(), error = %e, "staging janitor could not delete file")
                }
            }
        }
    }
    if removed > 0 {
        info!(removed, dir = %dir.display(), "staging janitor purged stale uploads");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deletes_files_older_than_cutoff() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), b"x").unwrap();
        std::fs::write(dir.path().join("b.txt"), b"y").unwrap();
        // cutoff in the future → everything is stale
        let cutoff = SystemTime::now() + Duration::from_secs(3600);
        cleanup_once(dir.path(), cutoff).await;
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[tokio::test]
    async fn test_keeps_fresh_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("fresh.txt"), b"z").unwrap();
        // cutoff in the past → nothing stale
        let cutoff = SystemTime::now() - Duration::from_secs(3600);
        cleanup_once(dir.path(), cutoff).await;
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[tokio::test]
    async fn test_missing_dir_is_noop() {
        cleanup_once(
            Path::new("/nonexistent/staging/xyz"),
            SystemTime::UNIX_EPOCH,
        )
        .await;
    }
}
