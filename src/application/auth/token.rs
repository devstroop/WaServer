//! Access Token application logic — extracted from `services/auth/auth.rs` and `handlers/api/users.rs`

use crate::domain::identity::{InstancePermission, UserRole};
use chrono::{DateTime, Utc};

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("not found")]
    NotFound,
    #[error("expired")]
    Expired,
    #[error("invalid")]
    Invalid(String),
}

#[derive(Debug, Clone)]
pub struct AccessToken {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub hash: String,
    pub expires_at: Option<DateTime<Utc>>,
}

impl AccessToken {
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires_at {
            Utc::now() > exp
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: String,
    pub username: String,
    pub role: UserRole,
    pub is_active: bool,
}

impl UserRecord {
    pub fn can_access(
        &self,
        permission: Option<InstancePermission>,
        required: InstancePermission,
    ) -> bool {
        if self.role.is_admin() {
            return true;
        }
        match permission {
            Some(p) => matches!(
                (p, required),
                (InstancePermission::Owner, _)
                    | (InstancePermission::Operator, InstancePermission::Viewer)
                    | (InstancePermission::Operator, InstancePermission::Operator)
            ),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_token_expiry() {
        let t = AccessToken {
            id: "1".into(),
            user_id: "u1".into(),
            name: "test".into(),
            hash: "h".into(),
            expires_at: Some(Utc::now() - chrono::Duration::hours(1)),
        };
        assert!(t.is_expired());
    }
    #[test]
    fn test_user_can_access() {
        let admin = UserRecord {
            id: "1".into(),
            username: "admin".into(),
            role: UserRole::Admin,
            is_active: true,
        };
        assert!(admin.can_access(None, InstancePermission::Viewer));
    }
}
