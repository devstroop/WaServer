//! Messaging Ports — so `application` has no `chromiumoxide`/`rusqlite`

use crate::domain::instance::InstanceId;
use crate::domain::messaging::{MediaType, MessageStatus};
use crate::domain::shared::error::DomainResult;
use async_trait::async_trait;

#[async_trait]
pub trait BrowserSendPort: Send + Sync {
    async fn send_text(&self, instance: InstanceId, to: &str, text: &str) -> DomainResult<String>;
    async fn send_media(
        &self,
        instance: InstanceId,
        to: &str,
        media_type: MediaType,
        path: &str,
        caption: Option<&str>,
    ) -> DomainResult<String>;
}

#[async_trait]
pub trait RateLimitPort: Send + Sync {
    async fn check_and_record(&self, instance: InstanceId) -> DomainResult<()>;
    async fn get_status(&self, instance: InstanceId) -> MessageStatus;
}
