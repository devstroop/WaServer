//! Instance Auth Watcher — DOM auth observer + phone verification impl block (part of #5)
//!
//! Split from `instance.rs`: AUTH_OBSERVER_JS injection, background watcher loop,
//! cached auth status with phone-mismatch auto-logout.

use super::instance::{CachedAuthStatus, InstanceService, AUTH_CACHE_TTL};
use crate::{
    models::auth::AuthStatusResponse, models::instance::InstanceStatus,
    services::auth::AuthServiceTrait,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

impl InstanceService {
    /// JavaScript that installs a MutationObserver on the DOM to detect when
    /// WhatsApp transitions to the authenticated state (chat list appears).
    /// When detected, it reads the sender ID from localStorage and writes
    /// a timestamped event to `__was_auth_event` so the Rust watcher can pick it up.
    pub(super) const AUTH_OBSERVER_JS: &'static str = r##"
        (function() {
            if (window.__was_auth_observer) return 'already_injected';

            // Helper: extract phone from WID
            function extractPhone() {
                try {
                    var wid = localStorage.getItem('last-wid') ?? localStorage.getItem('last-wid-md');
                    if (!wid) return null;
                    return wid.replace(/"/g, '').split('@')[0].split(':')[0];
                } catch(e) { return null; }
            }

            // Record an auth event
            function recordAuthEvent() {
                var phone = extractPhone();
                var evt = JSON.stringify({ ts: Date.now(), phone: phone || null });
                localStorage.setItem('__was_auth_event', evt);
            }

            // Check immediately if already authorized
            if (document.querySelector('#pane-side')
                || document.querySelector('[data-testid="chat-list"]')
                || document.querySelector('div[aria-label="Chat list"]')) {
                recordAuthEvent();
            }

            // Watch for future auth transitions
            var observer = new MutationObserver(function() {
                if (document.querySelector('#pane-side')
                    || document.querySelector('[data-testid="chat-list"]')
                    || document.querySelector('div[aria-label="Chat list"]')) {
                    recordAuthEvent();
                }
            });
            observer.observe(document.body, { childList: true, subtree: true });
            window.__was_auth_observer = observer;
            return 'injected';
        })()
    "##;

    /// Inject the DOM auth observer into the WhatsApp Web page.
    async fn inject_auth_observer(
        instance_id: crate::models::instance::InstanceId,
        browser_service: &Arc<crate::browser::BrowserService>,
    ) -> bool {
        let page = match browser_service.get_whatsapp_page().await {
            Ok(p) => p,
            Err(e) => {
                warn!(
                    "Instance '{}' — failed to get page for auth observer injection: {}",
                    instance_id, e
                );
                return false;
            }
        };

        match page.evaluate(Self::AUTH_OBSERVER_JS).await {
            Ok(result) => {
                let status = result
                    .into_value::<serde_json::Value>()
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default();
                info!("Instance '{}' — auth observer: {}", instance_id, status);
                true
            }
            Err(e) => {
                warn!(
                    "Instance '{}' — failed to inject auth observer: {}",
                    instance_id, e
                );
                false
            }
        }
    }

    /// Read and consume the `__was_auth_event` from localStorage.
    /// Returns the phone number from the event, if any.
    async fn consume_auth_event(
        instance_id: crate::models::instance::InstanceId,
        browser_service: &Arc<crate::browser::BrowserService>,
    ) -> Option<Option<String>> {
        let page = match browser_service.get_whatsapp_page().await {
            Ok(p) => p,
            Err(_) => return None,
        };

        let js = r#"
            (function() {
                var raw = localStorage.getItem('__was_auth_event');
                if (!raw) return null;
                localStorage.removeItem('__was_auth_event');
                try { return JSON.parse(raw); } catch(e) { return null; }
            })()
        "#;

        let result = match page.evaluate(js).await {
            Ok(r) => r,
            Err(e) => {
                debug!(
                    "Instance '{}' — failed to read auth event: {}",
                    instance_id, e
                );
                return None;
            }
        };

        let value: serde_json::Value = match result.into_value() {
            Ok(v) => v,
            Err(_) => return None,
        };

        if value.is_null() {
            return None;
        }

        // Event found — extract phone (may be null in the JSON)
        let phone = value
            .get("phone")
            .and_then(|v| v.as_str())
            .map(String::from);

        Some(phone)
    }
}

/// Continuous background watcher that polls for auth events and verifies the phone number.
/// The DOM observer is already injected by `warmup()` — this just consumes events.
/// Runs until the instance is stopped.
pub(super) async fn auth_watcher_loop(
    instance_id: crate::models::instance::InstanceId,
    phone_number: Option<String>,
    browser_service: Arc<crate::browser::BrowserService>,
    auth_service: Arc<dyn AuthServiceTrait>,
    auth_cache: Arc<Mutex<Option<CachedAuthStatus>>>,
    status: Arc<RwLock<InstanceStatus>>,
) {
    let expected_phone = match phone_number {
        Some(p) => p,
        None => {
            info!(
                "Instance '{}' — no phone configured, auth watcher not needed",
                instance_id
            );
            return;
        }
    };

    info!(
        "Instance '{}' — auth watcher started (expected phone: {})",
        instance_id, expected_phone
    );

    // Poll loop: check for auth events
    // First check is immediate, then every 2 seconds
    let poll_interval = std::time::Duration::from_secs(2);
    let mut first_check = true;

    loop {
        if !first_check {
            tokio::time::sleep(poll_interval).await;
        }
        first_check = false;

        // Stop if account is no longer running
        if !is_running_status(&status).await {
            info!(
                "Instance '{}' — auth watcher exiting (account stopped)",
                instance_id
            );
            return;
        }

        // Check for auth event from the DOM observer
        let event = InstanceService::consume_auth_event(instance_id, &browser_service).await;

        let browser_phone = match event {
            None => continue, // no event yet
            Some(phone_opt) => match phone_opt {
                Some(phone) if !phone.is_empty() => phone,
                _ => {
                    // Event fired but couldn't read phone from localStorage yet.
                    // Fall back to fetching via auth_service.
                    debug!(
                        "Instance '{}' — auth event with no phone, fetching sender ID...",
                        instance_id
                    );
                    match auth_service.get_sender_id().await {
                        Ok(Some(p)) => p,
                        Ok(None) => {
                            warn!(
                                "Instance '{}' — auth event but sender ID still null",
                                instance_id
                            );
                            continue;
                        }
                        Err(e) => {
                            warn!(
                                "Instance '{}' — auth event but get_sender_id failed: {}",
                                instance_id, e
                            );
                            continue;
                        }
                    }
                }
            },
        };

        info!(
            "Instance '{}' — auth event detected, browser phone: {}",
            instance_id, browser_phone
        );

        if browser_phone == expected_phone {
            info!(
                "Instance '{}' — phone verified: {}",
                instance_id, expected_phone
            );
            // Keep watching — user could log out and re-auth with wrong phone later
            continue;
        }

        warn!(
            "Instance '{}' — phone mismatch: expected {}, found {}. Triggering logout.",
            instance_id, expected_phone, browser_phone
        );

        if let Err(e) = auth_service.logout().await {
            error!("Instance '{}' — auto-logout failed: {}", instance_id, e);
        }
        // Invalidate auth cache
        let mut cache = auth_cache.lock().await;
        *cache = None;
        drop(cache);

        // After logout the page reloads — re-inject observer once it settles
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        if is_running_status(&status).await {
            // Re-inject by touching the page (triggers run_page_scripts if page was recreated,
            // otherwise inject manually since the in-memory page may have reloaded)
            InstanceService::inject_auth_observer(instance_id, &browser_service).await;
        }
    }
}

/// Check if the instance status is Active
async fn is_running_status(status: &Arc<RwLock<InstanceStatus>>) -> bool {
    matches!(&*status.read().await, InstanceStatus::Active)
}

impl InstanceService {
    /// Check authentication status directly
    pub async fn check_auth_status_directly(&self) -> anyhow::Result<bool> {
        let page = self.browser_service.get_whatsapp_page().await?;

        let script = r#"
            document.querySelector('#pane-side') !== null
        "#;

        match page.evaluate(script).await {
            Ok(result) => {
                let is_authorized = match result.into_value()? {
                    serde_json::Value::Bool(b) => b,
                    _ => false,
                };
                debug!(
                    "Instance '{}' - auth check result: {}",
                    self.id, is_authorized
                );
                Ok(is_authorized)
            }
            Err(e) => {
                error!("Instance '{}' - error checking auth status: {}", self.id, e);
                Ok(false)
            }
        }
    }

    /// Get authentication status with caching
    pub async fn get_auth_status(&self) -> anyhow::Result<AuthStatusResponse> {
        // Check cache first
        {
            let cache = self.auth_cache.lock().await;
            if let Some(cached) = cache.as_ref() {
                if cached.cached_at.elapsed() < AUTH_CACHE_TTL {
                    debug!("Instance '{}' - returning cached auth status", self.id);
                    return Ok(cached.status.clone());
                }
            }
        }

        // Cache miss or expired
        self.metrics.increment_auth_attempts();
        match self.auth_service.check_auth_status().await {
            Ok(auth_result) => {
                self.metrics.update_last_activity();

                let phone_number = if auth_result.authorized {
                    self.auth_service.get_sender_id().await.unwrap_or(None)
                } else {
                    None
                };

                // Phone mismatch guard: if the instance has a configured phone number
                // and the browser is authorized with a different number, auto-logout.
                if auth_result.authorized {
                    if let Some(ref browser_phone) = phone_number {
                        if let Some(ref expected_phone) = self._config.phone_number {
                            if browser_phone != expected_phone {
                                warn!(
                                    "Instance '{}' — phone mismatch: expected {}, got {}. Auto-logging out.",
                                    self.id, expected_phone, browser_phone
                                );
                                // Trigger logout in the background — don't block the status response
                                let auth_svc = self.auth_service.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = auth_svc.logout().await {
                                        tracing::error!("Auto-logout failed: {}", e);
                                    }
                                });
                                self.invalidate_auth_cache().await;

                                let status = AuthStatusResponse {
                                    authenticated: false,
                                    status: format!(
                                        "Phone mismatch — expected {}, got {}. Session terminated.",
                                        expected_phone, browser_phone
                                    ),
                                    phone_number: Some(browser_phone.clone()),
                                };
                                return Ok(status);
                            }
                        }
                    }
                }

                let status = AuthStatusResponse {
                    authenticated: auth_result.authorized,
                    status: auth_result.status,
                    phone_number,
                };

                // Update cache
                {
                    let mut cache = self.auth_cache.lock().await;
                    *cache = Some(CachedAuthStatus {
                        status: status.clone(),
                        cached_at: Instant::now(),
                    });
                }

                Ok(status)
            }
            Err(e) => {
                self.metrics.increment_error_count();
                Err(e)
            }
        }
    }

    /// Invalidate auth cache
    pub async fn invalidate_auth_cache(&self) {
        let mut cache = self.auth_cache.lock().await;
        *cache = None;
    }
}
