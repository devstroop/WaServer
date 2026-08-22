//! Instance Lifecycle — ports for browser/DB, extracted from `InstanceService` (1208 LOC)
//!
//! Defines `LifecyclePorts` trait so `application` does not depend on `chromiumoxide`/`rusqlite`.

use crate::domain::instance::{InstanceId, InstanceStatus};
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum LifecycleError {
    #[error("already warming up")]
    AlreadyWarming,
    #[error("browser error: {0}")]
    Browser(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("invalid state: {0}")]
    InvalidState(String),
}

/// Ports that `InstanceManager` / `InstanceService` must implement via `infrastructure`
#[async_trait]
pub trait LifecyclePorts: Send + Sync {
    async fn start_browser(&self, id: InstanceId) -> Result<(), LifecycleError>;
    async fn stop_browser(&self, id: InstanceId) -> Result<(), LifecycleError>;
    async fn get_status(&self, id: InstanceId) -> InstanceStatus;
    async fn set_status(&self, id: InstanceId, status: InstanceStatus) -> Result<(), LifecycleError>;
}
