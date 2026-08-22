//! Instance Lifecycle — browser lifecycle impl block split from `instance.rs` (part of #5)
//!
//! Owns `warmup`, `ensure_warm`, idle auto-sleep timer, `sleep`, `reset`.
//! State machine transitions validated by `application::instance::InstanceState`.

use super::instance::InstanceService;
use crate::{
    models::instance::InstanceStatus, services::whatsapp::instance_auth::auth_watcher_loop,
};
use anyhow::{anyhow, Result};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

impl InstanceService {
    /// Warmup the instance (launch browser, navigate to WhatsApp)
    /// If already active, this is a no-op. If warming up, returns error.
    pub async fn warmup(&self) -> Result<()> {
        // If status is Active, verify the browser is actually alive
        {
            let status = self.status.read().await;
            if matches!(&*status, InstanceStatus::Active) {
                drop(status);
                if self.browser_service.is_responsive().await {
                    self.touch_activity().await;
                    return Ok(());
                }
                // Browser died externally — clean up stale handles and re-warm
                warn!(
                    "Instance '{}' — browser dead while status=Active, re-warming",
                    self.id
                );
                self.cancel_idle_timer().await;
                // Close clears the dead browser/page handles so initialize() starts fresh
                let _ = self.browser_service.close().await;
                let mut s = self.status.write().await;
                *s = InstanceStatus::Sleeping;
                drop(s);
            }
        }

        let mut status = self.status.write().await;

        match &*status {
            InstanceStatus::Active => {
                // Raced with another caller that already re-warmed
                drop(status);
                self.touch_activity().await;
                return Ok(());
            }
            InstanceStatus::WarmingUp => {
                return Err(anyhow!("Instance '{}' is already warming up", self.id));
            }
            _ => {}
        }

        *status = InstanceStatus::WarmingUp;
        drop(status);

        info!("Warming up account '{}'", self.id);

        match self.browser_service.initialize().await {
            Ok(()) => {
                // Start auth watcher immediately in background (parallel to UI check)
                let instance_id = self.id;
                let phone_number = self._config.phone_number.clone();
                let browser_service = self.browser_service.clone();
                let auth_service = self.auth_service.clone();
                let auth_cache = self.auth_cache.clone();
                let account_status = self.status.clone();
                tokio::spawn(async move {
                    auth_watcher_loop(
                        instance_id,
                        phone_number,
                        browser_service,
                        auth_service,
                        auth_cache,
                        account_status,
                    )
                    .await;
                });

                // Wait for WhatsApp Web UI to be ready (either logged in or showing QR)
                // This prevents "Not authorized" errors when the UI hasn't loaded yet
                debug!(
                    "Instance '{}' — waiting for WhatsApp UI to be ready...",
                    self.id
                );

                let max_wait = Duration::from_secs(10);
                let poll_interval = Duration::from_millis(200);
                let start = std::time::Instant::now();

                let ui_ready = loop {
                    if start.elapsed() > max_wait {
                        warn!(
                            "Instance '{}' — UI ready check timed out, proceeding anyway",
                            self.id
                        );
                        break false;
                    }

                    // Check if either #pane-side (logged in) or QR canvas exists
                    if let Ok(page) = self.browser_service.get_whatsapp_page().await {
                        let check_script = r#"
                            document.querySelector('#pane-side') !== null ||
                            document.querySelector('canvas[aria-label]') !== null ||
                            document.querySelector('[data-ref]') !== null
                        "#;
                        if let Ok(result) = page.evaluate(check_script).await {
                            if result.into_value::<bool>().unwrap_or(false) {
                                break true;
                            }
                        }
                    }

                    tokio::time::sleep(poll_interval).await;
                };

                if ui_ready {
                    debug!("Instance '{}' — WhatsApp UI is ready", self.id);
                }

                let mut status = self.status.write().await;
                *status = InstanceStatus::Active;
                let mut initialized = self.initialized.lock().await;
                *initialized = true;
                drop(initialized);
                drop(status);

                self.touch_activity().await;
                self.start_idle_timer().await;

                info!("Instance '{}' warmed up successfully", self.id);

                Ok(())
            }
            Err(e) => {
                let mut status = self.status.write().await;
                *status = InstanceStatus::Error(e.to_string());
                error!("Failed to warm up instance '{}': {}", self.id, e);
                Err(e)
            }
        }
    }

    /// Ensure the instance is warm (active). Auto-warms if sleeping.
    /// Call this before any operation that requires the browser.
    pub async fn ensure_warm(&self) -> Result<()> {
        let status = self.status.read().await.clone();
        match status {
            InstanceStatus::Active => {
                // Verify the browser is actually alive — it may have been closed externally
                if !self.browser_service.is_responsive().await {
                    warn!(
                        "Instance '{}' — browser dead while status=Active, re-warming",
                        self.id
                    );
                    self.cancel_idle_timer().await;
                    let _ = self.browser_service.close().await;
                    *self.status.write().await = InstanceStatus::Sleeping;
                    return self.warmup().await;
                }
                self.touch_activity().await;
                Ok(())
            }
            InstanceStatus::Sleeping | InstanceStatus::Error(_) => self.warmup().await,
            InstanceStatus::WarmingUp => {
                // Wait for warmup to complete (poll with short interval)
                drop(status);
                for _ in 0..300 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    let s = self.status.read().await.clone();
                    match s {
                        InstanceStatus::Active => {
                            self.touch_activity().await;
                            return Ok(());
                        }
                        InstanceStatus::Error(e) => {
                            return Err(anyhow!("Instance warmup failed: {}", e));
                        }
                        InstanceStatus::Sleeping => {
                            return self.warmup().await;
                        }
                        InstanceStatus::WarmingUp => continue,
                    }
                }
                Err(anyhow!(
                    "Timeout waiting for account '{}' to warm up",
                    self.id
                ))
            }
        }
    }

    /// Update last activity timestamp (resets the idle timer)
    pub async fn touch_activity(&self) {
        let mut last = self.last_activity.write().await;
        *last = Instant::now();
    }

    /// Start the idle-sleep background timer
    pub(super) async fn start_idle_timer(&self) {
        // Cancel any existing idle timer
        self.cancel_idle_timer().await;

        let idle_timeout_val = self.instance_config.read().await.idle_timeout;
        if idle_timeout_val == 0 {
            debug!(
                "Instance '{}' — idle auto-sleep disabled (timeout=0)",
                self.id
            );
            return;
        }

        let instance_id = self.id;
        let status = self.status.clone();
        let last_activity = self.last_activity.clone();
        let browser_service = self.browser_service.clone();
        let initialized = self.initialized.clone();
        let idle_timeout = Duration::from_secs(idle_timeout_val);

        let handle = tokio::spawn(async move {
            let check_interval = Duration::from_secs(30);
            loop {
                tokio::time::sleep(check_interval).await;

                // Check if still active
                if !matches!(&*status.read().await, InstanceStatus::Active) {
                    return;
                }

                let elapsed = last_activity.read().await.elapsed();
                if elapsed >= idle_timeout {
                    info!(
                        "Instance '{}' — idle for {:?}, auto-sleeping",
                        instance_id, elapsed
                    );

                    // Perform sleep: close browser, set status
                    let mut s = status.write().await;
                    *s = InstanceStatus::Sleeping;
                    drop(s);

                    let mut init = initialized.lock().await;
                    *init = false;
                    drop(init);

                    if let Err(e) = browser_service.close().await {
                        warn!(
                            "Instance '{}' — error during auto-sleep: {}",
                            instance_id, e
                        );
                    }

                    info!("Instance '{}' — auto-slept", instance_id);
                    return;
                }
            }
        });

        let mut guard = self.idle_sleep_handle.lock().await;
        *guard = Some(handle);
    }

    /// Cancel the idle-sleep timer
    pub(super) async fn cancel_idle_timer(&self) {
        let mut guard = self.idle_sleep_handle.lock().await;
        if let Some(handle) = guard.take() {
            handle.abort();
        }
    }

    /// Sleep the instance (close browser, set to sleeping)
    pub async fn sleep(&self) -> Result<()> {
        info!("Sleeping account '{}'", self.id);

        self.cancel_idle_timer().await;

        let mut status = self.status.write().await;
        *status = InstanceStatus::Sleeping;
        drop(status);

        let mut initialized = self.initialized.lock().await;
        *initialized = false;
        drop(initialized);

        self.browser_service.close().await?;

        info!("Instance '{}' is now sleeping", self.id);
        Ok(())
    }

    /// Reset the instance — sleep browser and wipe all session data (chrome profile, sessions, media).
    /// The account record and config are preserved; only runtime/browser data is cleared.
    pub async fn reset(&self) -> Result<()> {
        info!("Resetting account '{}'", self.id);

        // Sleep browser first
        self.sleep().await.ok(); // ignore if already sleeping

        // Wipe session-related directories
        let dirs_to_clear = ["chrome-profile", "sessions", "media"];
        for dir_name in &dirs_to_clear {
            let dir = self.data_dir.join(dir_name);
            if dir.exists() {
                tokio::fs::remove_dir_all(&dir).await?;
                tokio::fs::create_dir_all(&dir).await?;
            }
        }

        // Clear auth cache
        self.invalidate_auth_cache().await;

        info!("Instance '{}' reset — session data cleared", self.id);
        Ok(())
    }
}
