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
    async fn set_status(
        &self,
        id: InstanceId,
        status: InstanceStatus,
    ) -> Result<(), LifecycleError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lifecycle_error_display() {
        assert_eq!(
            LifecycleError::AlreadyWarming.to_string(),
            "already warming up"
        );
        assert_eq!(
            LifecycleError::Browser("dead".into()).to_string(),
            "browser error: dead"
        );
        assert_eq!(
            LifecycleError::Storage("disk full".into()).to_string(),
            "storage error: disk full"
        );
    }

    #[tokio::test]
    async fn test_lifecycle_ports_mock() {
        struct Mock;
        #[async_trait]
        impl LifecyclePorts for Mock {
            async fn start_browser(&self, _id: InstanceId) -> Result<(), LifecycleError> {
                Ok(())
            }
            async fn stop_browser(&self, _id: InstanceId) -> Result<(), LifecycleError> {
                Ok(())
            }
            async fn get_status(&self, _id: InstanceId) -> InstanceStatus {
                InstanceStatus::Sleeping
            }
            async fn set_status(
                &self,
                _id: InstanceId,
                _status: InstanceStatus,
            ) -> Result<(), LifecycleError> {
                Ok(())
            }
        }
        let mock = Mock;
        let id = uuid::Uuid::nil();
        assert!(mock.start_browser(id).await.is_ok());
        assert_eq!(mock.get_status(id).await, InstanceStatus::Sleeping);
    }
}
