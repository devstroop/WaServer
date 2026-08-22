//! Messaging infrastructure adapters — implements `BrowserSendPort` + `RateLimitPort`
//! over `InstanceManager`/`InstanceService` (part of #7)
//!
//! This is the only place where `application::messaging::ports` meets the browser.
//! `SendService` stays pure; handlers wire this adapter in.

use crate::application::messaging::ports::{BrowserSendPort, RateLimitPort};
use crate::domain::instance::InstanceId;
use crate::domain::messaging::{MediaType, MessageStatus};
use crate::services::InstanceManager;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::{error, info};
use uuid::Uuid;

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Adapter: browser sends via `ChatService` inside busy-flag guard
pub struct ManagerBrowserAdapter {
    pub manager: Arc<InstanceManager>,
}

impl ManagerBrowserAdapter {
    async fn send_inner(
        &self,
        instance: InstanceId,
        to: &str,
        text: Option<&str>,
        attachment: Option<&str>,
    ) -> Result<String, String> {
        let account = self
            .manager
            .get_instance_by_id(instance)
            .await
            .ok_or_else(|| format!("instance '{}' not found", instance))?;

        // Ensure browser warm before sending
        account
            .ensure_warm()
            .await
            .map_err(|e| format!("warmup failed: {}", e))?;

        let msg_id = Uuid::new_v4().to_string();
        let result = account
            .execute_with_busy_flag(async {
                account
                    .chat_service()
                    .send_message(to, text, attachment, None)
                    .await
            })
            .await;

        match result {
            Ok(()) => {
                account.track_message_sent();
                let metrics = self.manager.observability.for_instance(instance).await;
                metrics.track_message_sent();
                info!("Instance {} - sent to {} (id: {})", instance, to, msg_id);
                Ok(msg_id)
            }
            Err(e) => {
                account.track_error();
                let metrics = self.manager.observability.for_instance(instance).await;
                metrics.track_error();
                error!("Instance {} - send failed to {}: {}", instance, to, e);
                Err(e.to_string())
            }
        }
    }
}

#[async_trait]
impl BrowserSendPort for ManagerBrowserAdapter {
    async fn send_text(
        &self,
        instance: InstanceId,
        to: &str,
        text: &str,
    ) -> Result<String, String> {
        self.send_inner(instance, to, Some(text), None).await
    }

    async fn send_media(
        &self,
        instance: InstanceId,
        to: &str,
        _media_type: MediaType,
        path: &str,
        caption: Option<&str>,
    ) -> Result<String, String> {
        self.send_inner(instance, to, caption, Some(path)).await
    }
}

/// Sliding-window rate limiter backed by instance config (`messages_per_minute`)
#[derive(Debug, Default)]
struct WindowState {
    /// Timestamps (secs) of recent sends
    window: VecDeque<u64>,
}

/// Adapter: rate limiting per instance from its `InstanceConfig.rate_limits`
///
/// Clones share the same window state (`Arc` inner) — safe to construct per-request
/// while keeping one global sliding-window per instance.
#[derive(Clone)]
pub struct ManagerRateAdapter {
    pub manager: Arc<InstanceManager>,
    windows: Arc<Mutex<std::collections::HashMap<InstanceId, WindowState>>>,
}

impl ManagerRateAdapter {
    pub fn new(manager: Arc<InstanceManager>) -> Self {
        Self {
            manager,
            windows: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl RateLimitPort for ManagerRateAdapter {
    async fn check_and_record(&self, instance: InstanceId) -> Result<(), String> {
        // Resolve limit from instance config (default 60/min)
        let max_per_minute = self
            .manager
            .registry
            .get_config(instance)
            .await
            .map(|c| c.rate_limits.messages_per_minute)
            .unwrap_or(60);

        let now = now_secs();
        let mut windows = self.windows.lock().await;
        let state = windows.entry(instance).or_default();

        // Evict entries older than 60s
        while let Some(front) = state.window.front() {
            if now.saturating_sub(*front) >= 60 {
                state.window.pop_front();
            } else {
                break;
            }
        }

        if state.window.len() >= max_per_minute as usize {
            return Err(format!(
                "rate limited: {} messages in the last minute (limit {})",
                state.window.len(),
                max_per_minute
            ));
        }

        state.window.push_back(now);
        Ok(())
    }

    async fn get_status(&self, instance: InstanceId) -> MessageStatus {
        match self.manager.get_instance_by_id(instance).await {
            Some(_) => MessageStatus::Sent,
            None => MessageStatus::Failed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_window_eviction_logic() {
        // Pure window logic check without a real manager: direct VecDeque behavior
        let mut w = WindowState {
            window: VecDeque::new(),
        };
        let now = now_secs();
        w.window.push_back(now - 61); // stale
        w.window.push_back(now);
        while let Some(front) = w.window.front() {
            if now.saturating_sub(*front) >= 60 {
                w.window.pop_front();
            } else {
                break;
            }
        }
        assert_eq!(w.window.len(), 1);
    }

    #[test]
    fn test_adapters_exist() {
        let _ = std::any::type_name::<ManagerBrowserAdapter>();
        let _ = std::any::type_name::<ManagerRateAdapter>();
    }
}
