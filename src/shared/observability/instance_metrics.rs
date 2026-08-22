//! Per-instance metrics — part of #6 observability
//! Thin per-instance counters keyed by UUID; complements `shared::observability::metrics`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Per-instance counters (atomic)
#[derive(Debug, Default)]
pub struct InstanceMetrics {
    pub messages_sent: AtomicU64,
    pub errors: AtomicU64,
    pub warmups: AtomicU64,
    pub last_activity_unix: AtomicU64,
}

impl InstanceMetrics {
    pub fn track_message_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
        self.touch();
    }
    pub fn track_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
        self.touch();
    }
    pub fn track_warmup(&self) {
        self.warmups.fetch_add(1, Ordering::Relaxed);
        self.touch();
    }
    fn touch(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_activity_unix.store(now, Ordering::Relaxed);
    }
    pub fn snapshot(&self) -> InstanceMetricsSnapshot {
        InstanceMetricsSnapshot {
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            warmups: self.warmups.load(Ordering::Relaxed),
            last_activity_unix: {
                let t = self.last_activity_unix.load(Ordering::Relaxed);
                if t == 0 {
                    None
                } else {
                    Some(t)
                }
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstanceMetricsSnapshot {
    pub messages_sent: u64,
    pub errors: u64,
    pub warmups: u64,
    pub last_activity_unix: Option<u64>,
}

/// Registry-wide metrics collection keyed by instance UUID
#[derive(Default)]
pub struct InstanceMetricsRegistry {
    by_id: RwLock<HashMap<uuid::Uuid, Arc<InstanceMetrics>>>,
}

impl InstanceMetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create counters for an instance
    pub async fn for_instance(&self, id: uuid::Uuid) -> Arc<InstanceMetrics> {
        let mut map = self.by_id.write().await;
        map.entry(id).or_default().clone()
    }

    /// Remove counters when instance is deleted
    pub async fn remove_instance(&self, id: uuid::Uuid) {
        self.by_id.write().await.remove(&id);
    }

    /// Snapshot all instances (for `/api/metrics` enrichment)
    pub async fn snapshot_all(&self) -> HashMap<uuid::Uuid, InstanceMetricsSnapshot> {
        let map = self.by_id.read().await;
        map.iter().map(|(id, m)| (*id, m.snapshot())).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_track_and_snapshot() {
        let reg = InstanceMetricsRegistry::new();
        let id = uuid::Uuid::new_v4();
        let m = reg.for_instance(id).await;
        m.track_message_sent();
        m.track_message_sent();
        m.track_error();
        m.track_warmup();

        let snap = reg.snapshot_all().await;
        let s = snap.get(&id).unwrap();
        assert_eq!(s.messages_sent, 2);
        assert_eq!(s.errors, 1);
        assert_eq!(s.warmups, 1);
        assert!(s.last_activity_unix.is_some());
    }

    #[tokio::test]
    async fn test_remove_instance() {
        let reg = InstanceMetricsRegistry::new();
        let id = uuid::Uuid::new_v4();
        let _ = reg.for_instance(id).await;
        reg.remove_instance(id).await;
        assert!(reg.snapshot_all().await.is_empty());
    }
}
