//! User repository — extracted from `src/infrastructure/persistence/service.rs:110..320` (#9)
//! Handlers must not see `rusqlite`; they depend on `UserStore` port from `application::auth`.
//! This repo implements `UserStore` for `Database`. Currently a thin wrapper around `service.rs`.

use crate::application::auth::{UserRecord as AppUserRecord, UserStore};
use crate::infrastructure::persistence::service::Database;
use async_trait::async_trait;

/// Wrapper that adapts `Database` to `UserStore` (rusqlite-free application)
pub struct SqliteUserStore(pub Database);

#[async_trait]
impl UserStore for SqliteUserStore {
    async fn find_by_id(&self, id: &str) -> Option<AppUserRecord> {
        self.0.get_user(id).ok().flatten().map(|u| AppUserRecord {
            id: u.id,
            username: u.username,
            role: u.role,
            is_active: u.is_active,
        })
    }
    async fn find_by_username(&self, username: &str) -> Option<AppUserRecord> {
        self.0
            .get_user_by_username(username)
            .ok()
            .flatten()
            .map(|u| AppUserRecord {
                id: u.id,
                username: u.username,
                role: u.role,
                is_active: u.is_active,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_store_exists() {
        let _ = std::any::type_name::<SqliteUserStore>();
    }
}
