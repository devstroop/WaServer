//! Send Service — application use-case extracted from `services/whatsapp/chat.rs:910` and `handlers/api/chat.rs:285`
//! Thin, testable, no browser/DB deps — depends on ports.

use crate::application::messaging::policy::{SendPolicy, ValidatePhone};
use crate::application::messaging::ports::{BrowserSendPort, RateLimitPort};
use crate::domain::instance::InstanceId;
use crate::domain::messaging::MediaType;
use crate::domain::shared::error::{DomainError, DomainResult};
use std::sync::Arc;
use std::time::Instant;

pub struct SendMessageCommand {
    pub instance: InstanceId,
    pub to: String,
    pub text: Option<String>,
    pub media_type: MediaType,
    pub media_path: Option<String>,
}

pub struct SendService {
    validator: Arc<dyn ValidatePhone + Send + Sync>,
    browser: Arc<dyn BrowserSendPort + Send + Sync>,
    rate: Arc<dyn RateLimitPort + Send + Sync>,
    policy: SendPolicy,
}

impl SendService {
    pub fn new(
        validator: Arc<dyn ValidatePhone + Send + Sync>,
        browser: Arc<dyn BrowserSendPort + Send + Sync>,
        rate: Arc<dyn RateLimitPort + Send + Sync>,
    ) -> Self {
        Self {
            validator,
            browser,
            rate,
            policy: SendPolicy::default_for_instance(),
        }
    }

    pub fn with_policy(mut self, policy: SendPolicy) -> Self {
        self.policy = policy;
        self
    }

    pub async fn send(&self, cmd: SendMessageCommand) -> DomainResult<String> {
        let start = Instant::now();
        let to = self.validator.validate(&cmd.to)?;
        // policy checks before rate (media, queue, cooldown)
        let status = self.rate.get_status(cmd.instance).await;
        self.policy.can_send(cmd.media_type, status, 0, None, 0)?;
        self.rate.check_and_record(cmd.instance).await?;
        let result = if let Some(text) = cmd.text {
            self.browser.send_text(cmd.instance, &to, &text).await
        } else if let Some(path) = cmd.media_path {
            self.browser
                .send_media(cmd.instance, &to, cmd.media_type, &path, None)
                .await
        } else {
            Err(DomainError::Validation("no content".into()))
        };
        // observability
        let elapsed = start.elapsed().as_millis() as u64;
        tracing::info!(instance=%cmd.instance, to=%to, elapsed_ms=elapsed, success=result.is_ok(), "send attempted");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::messaging::policy::E164Validator;
    use crate::domain::instance::InstanceId;
    use crate::domain::messaging::{MediaType, MessageStatus};
    use crate::domain::shared::error::DomainResult;
    use async_trait::async_trait;

    struct OkBrowser;
    #[async_trait]
    impl BrowserSendPort for OkBrowser {
        async fn send_text(&self, _i: InstanceId, _to: &str, _t: &str) -> DomainResult<String> {
            Ok("msg-1".into())
        }
        async fn send_media(
            &self,
            _i: InstanceId,
            _to: &str,
            _t: MediaType,
            _p: &str,
            _c: Option<&str>,
        ) -> DomainResult<String> {
            Ok("msg-2".into())
        }
    }
    struct OkRate;
    #[async_trait]
    impl RateLimitPort for OkRate {
        async fn check_and_record(&self, _id: InstanceId) -> DomainResult<()> {
            Ok(())
        }
        async fn get_status(&self, _id: InstanceId) -> MessageStatus {
            MessageStatus::Sent
        }
    }
    struct FailRate;
    #[async_trait]
    impl RateLimitPort for FailRate {
        async fn check_and_record(&self, _id: InstanceId) -> DomainResult<()> {
            Err(crate::domain::shared::error::DomainError::RateLimited { operation: "send".into(), retry_after_seconds: 5 })
        }
        async fn get_status(&self, _id: InstanceId) -> MessageStatus {
            MessageStatus::Pending
        }
    }

    #[tokio::test]
    async fn test_send_text_ok() {
        let svc = SendService::new(
            Arc::new(E164Validator),
            Arc::new(OkBrowser),
            Arc::new(OkRate),
        );
        let cmd = SendMessageCommand {
            instance: uuid::Uuid::nil(),
            to: "+1234567890".into(),
            text: Some("hello".into()),
            media_type: MediaType::None,
            media_path: None,
        };
        assert_eq!(svc.send(cmd).await.unwrap(), "msg-1");
    }

    #[tokio::test]
    async fn test_send_invalid_phone() {
        let svc = SendService::new(
            Arc::new(E164Validator),
            Arc::new(OkBrowser),
            Arc::new(OkRate),
        );
        let cmd = SendMessageCommand {
            instance: uuid::Uuid::nil(),
            to: "123".into(),
            text: Some("hi".into()),
            media_type: MediaType::None,
            media_path: None,
        };
        assert!(svc.send(cmd).await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limited_blocks_send() {
        let svc = SendService::new(Arc::new(E164Validator), Arc::new(OkBrowser), Arc::new(FailRate));
        let cmd = SendMessageCommand {
            instance: uuid::Uuid::nil(),
            to: "+1234567890".into(),
            text: Some("hi".into()),
            media_type: MediaType::None,
            media_path: None,
        };
        let err = svc.send(cmd).await.unwrap_err();
        assert!(matches!(err, crate::domain::shared::error::DomainError::RateLimited { .. }));
    }

    #[tokio::test]
    async fn test_media_not_allowed() {
        let policy = crate::application::messaging::policy::SendPolicy {
            allowed_media: vec![MediaType::None],
            ..Default::default()
        };
        let svc = SendService::new(Arc::new(E164Validator), Arc::new(OkBrowser), Arc::new(OkRate)).with_policy(policy);
        let cmd = SendMessageCommand {
            instance: uuid::Uuid::nil(),
            to: "+1234567890".into(),
            text: None,
            media_type: MediaType::Image,
            media_path: Some("/tmp/x.jpg".into()),
        };
        assert!(svc.send(cmd).await.is_err());
    }

    #[tokio::test]
    async fn test_browser_not_called_on_rate_limit() {
        struct CountingBrowser { count: std::sync::Arc<tokio::sync::Mutex<usize>> }
        #[async_trait]
        impl BrowserSendPort for CountingBrowser {
            async fn send_text(&self, _i: InstanceId, _to: &str, _t: &str) -> DomainResult<String> {
                *self.count.lock().await += 1;
                Ok("x".into())
            }
            async fn send_media(&self, _i: InstanceId, _to: &str, _t: MediaType, _p: &str, _c: Option<&str>) -> DomainResult<String> { Ok("x".into()) }
        }
        let counter = std::sync::Arc::new(tokio::sync::Mutex::new(0usize));
        let svc = SendService::new(Arc::new(E164Validator), Arc::new(CountingBrowser { count: counter.clone() }), Arc::new(FailRate));
        let cmd = SendMessageCommand {
            instance: uuid::Uuid::nil(),
            to: "+1234567890".into(),
            text: Some("hi".into()),
            media_type: MediaType::None,
            media_path: None,
        };
        let _ = svc.send(cmd).await;
        assert_eq!(*counter.lock().await, 0);
    }
}
