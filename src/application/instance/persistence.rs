//! Persistence port for instance — so `application` does not depend on `rusqlite`/`tokio::fs` directly
//! Will be implemented by `infrastructure/persistence` in `feat/foundation/platform` follow-up.

use crate::domain::instance::{InstanceConfig, InstanceId, InstanceMetadata};
use async_trait::async_trait;

#[async_trait]
pub trait InstanceStore: Send + Sync {
    async fn load_metadata(&self, id: InstanceId) -> Option<InstanceMetadata>;
    async fn save_metadata(&self, metadata: &InstanceMetadata) -> anyhow::Result<()>;
    async fn load_config(&self, id: InstanceId) -> Option<InstanceConfig>;
    async fn save_config(&self, id: InstanceId, config: &InstanceConfig) -> anyhow::Result<()>;
}
