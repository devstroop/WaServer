//! Messaging infrastructure adapters — implements `BrowserSendPort`
//! over `InstanceManager`/`InstanceService` (part of #7)
//!
//! This is the only place where `application::messaging::ports` meets the browser.
//! `SendService` stays pure; handlers wire this adapter in.

use crate::application::messaging::ports::BrowserSendPort;
use crate::domain::instance::InstanceId;
use crate::domain::messaging::MediaType;
use crate::domain::shared::error::{DomainError, DomainResult};
use crate::services::InstanceManager;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, info};

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
    ) -> DomainResult<String> {
        let account = self
            .manager
            .get_instance_by_id(instance)
            .await
            .ok_or_else(|| DomainError::not_found("instance", &instance.to_string()))?;

        // Ensure browser warm before sending
        account
            .ensure_warm()
            .await
            .map_err(|e| DomainError::Internal(format!("warmup failed: {}", e)))?;

        let msg_id = uuid::Uuid::new_v4().to_string();
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
                Err(DomainError::Internal(e.to_string()))
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
    ) -> DomainResult<String> {
        self.send_inner(instance, to, Some(text), None).await
    }

    async fn send_media(
        &self,
        instance: InstanceId,
        to: &str,
        _media_type: MediaType,
        path: &str,
        caption: Option<&str>,
    ) -> DomainResult<String> {
        self.send_inner(instance, to, caption, Some(path)).await
    }
}

// Sliding-window rate limiting now lives in
// `infrastructure::messaging::InMemoryRateLimiter` (shared via `InstanceManager.rate_limiter`).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapters_exist() {
        let _ = std::any::type_name::<ManagerBrowserAdapter>();
    }
}
