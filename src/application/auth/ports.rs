//! Auth Ports — so `application` has no `rusqlite`

use crate::application::auth::token::{AccessToken, UserRecord};
use crate::domain::identity::InstancePermission;
use async_trait::async_trait;

#[async_trait]
pub trait UserStore: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Option<UserRecord>;
    async fn find_by_username(&self, username: &str) -> Option<UserRecord>;
}

#[async_trait]
pub trait TokenStore: Send + Sync {
    async fn find_by_hash(&self, hash: &str) -> Option<(UserRecord, AccessToken)>;
    async fn check_permission(
        &self,
        user_id: &str,
        instance_id: &str,
    ) -> Option<InstancePermission>;
}
