//! Send Service — application use-case extracted from `services/whatsapp/chat.rs:910` and `handlers/api/chat.rs:285`
//! Thin, testable, no browser/DB deps — depends on ports.

use crate::application::messaging::policy::ValidatePhone;
use crate::application::messaging::ports::{BrowserSendPort, RateLimitPort};
use crate::domain::instance::InstanceId;
use crate::domain::messaging::MediaType;
use std::sync::Arc;

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
        }
    }

    pub async fn send(&self, cmd: SendMessageCommand) -> Result<String, String> {
        let to = self.validator.validate(&cmd.to)?;
        self.rate.check_and_record(cmd.instance).await?;
        if let Some(text) = cmd.text {
            self.browser.send_text(cmd.instance, &to, &text).await
        } else if let Some(path) = cmd.media_path {
            self.browser
                .send_media(cmd.instance, &to, cmd.media_type, &path, None)
                .await
        } else {
            Err("no content".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::messaging::policy::E164Validator;
    use crate::domain::instance::InstanceId;
    use crate::domain::messaging::{MediaType, MessageStatus};
    use async_trait::async_trait;

    struct OkBrowser;
    #[async_trait]
    impl BrowserSendPort for OkBrowser {
        async fn send_text(&self, _i: InstanceId, _to: &str, _t: &str) -> Result<String, String> {
            Ok("msg-1".into())
        }
        async fn send_media(
            &self,
            _i: InstanceId,
            _to: &str,
            _t: MediaType,
            _p: &str,
            _c: Option<&str>,
        ) -> Result<String, String> {
            Ok("msg-2".into())
        }
    }
    struct OkRate;
    #[async_trait]
    impl RateLimitPort for OkRate {
        async fn check_and_record(&self, _id: InstanceId) -> Result<(), String> {
            Ok(())
        }
        async fn get_status(&self, _id: InstanceId) -> MessageStatus {
            MessageStatus::Sent
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
}
