//! Token repository — extracted from `src/infrastructure/persistence/service.rs:410..550` (#9)
//! Implements `TokenStore` port for `Database`. Keeps `handlers/api/users.rs:461` DB-free.

use crate::application::auth::{AccessToken, TokenStore, UserRecord as AppUserRecord};
use crate::domain::identity::InstancePermission;
use crate::infrastructure::persistence::service::Database;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

pub struct SqliteTokenStore(pub Database);

#[async_trait]
impl TokenStore for SqliteTokenStore {
    async fn find_by_hash(&self, hash: &str) -> Option<(AppUserRecord, AccessToken)> {
        self.0
            .get_user_by_access_token(hash)
            .ok()
            .flatten()
            .map(|(u, t)| {
                let user = AppUserRecord {
                    id: u.id,
                    username: u.username,
                    role: u.role,
                    is_active: u.is_active,
                };
                let token = AccessToken {
                    id: t.id,
                    user_id: t.user_id,
                    name: t.name,
                    hash: t.token_hash,
                    expires_at: t.expires_at.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .ok()
                            .map(|d| d.with_timezone(&Utc))
                    }),
                };
                (user, token)
            })
    }
    async fn check_permission(
        &self,
        user_id: &str,
        instance_id: &str,
    ) -> Option<InstancePermission> {
        self.0
            .get_instance_permission(user_id, instance_id)
            .ok()
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_store_exists() {
        let _ = std::any::type_name::<SqliteTokenStore>();
    }
}
