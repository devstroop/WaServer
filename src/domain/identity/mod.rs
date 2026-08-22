//! Identity Domain — UserRole, InstancePermission, User
//!
//! Pure domain, no DB/HTTP. `UserRecord`/`AccessTokenRecord` stay in infra persistence;
//! domain holds only value objects and the `User` entity.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
}

impl UserRole {
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::User => write!(f, "user"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(UserRole::Admin),
            "user" => Ok(UserRole::User),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum InstancePermission {
    Owner,
    Operator,
    Viewer,
}

impl InstancePermission {
    pub fn can_send(&self) -> bool {
        matches!(
            self,
            InstancePermission::Owner | InstancePermission::Operator
        )
    }
    pub fn can_modify(&self) -> bool {
        matches!(self, InstancePermission::Owner)
    }
    pub fn can_delete(&self) -> bool {
        matches!(self, InstancePermission::Owner)
    }
}

impl std::fmt::Display for InstancePermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstancePermission::Owner => write!(f, "owner"),
            InstancePermission::Operator => write!(f, "operator"),
            InstancePermission::Viewer => write!(f, "viewer"),
        }
    }
}

impl std::str::FromStr for InstancePermission {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(InstancePermission::Owner),
            "operator" => Ok(InstancePermission::Operator),
            "viewer" => Ok(InstancePermission::Viewer),
            _ => Err(format!("Invalid permission: {}", s)),
        }
    }
}

/// Pure domain User entity (no password_hash exposure)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub role: UserRole,
    pub is_active: bool,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }
}
