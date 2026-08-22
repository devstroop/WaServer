//! Identity DTOs — users, tokens, RBAC (part of #9 #11)
//! Mirrors `models/user.rs:328` but versioned under `interfaces`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateUserRequestDto {
    pub username: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserInfoDto {
    pub id: String,
    pub username: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAccessTokenRequestDto {
    pub name: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccessTokenInfoDto {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub expires_at: Option<String>,
    pub last_used: Option<String>,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_user_dto_serde() {
        let dto = UserInfoDto {
            id: "1".into(),
            username: "alice".into(),
            role: "user".into(),
            is_active: true,
            created_at: "now".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("alice"));
    }
}
