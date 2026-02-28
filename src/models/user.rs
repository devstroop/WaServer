//! User and RBAC Models
//!
//! Models for user management, roles, and instance permissions.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// =============================================================================
// Role & Permission Enums
// =============================================================================

/// User roles for access control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Full system access: manage users, all instances, config
    Admin,
    /// Standard user: access only owned/permitted instances
    User,
}

impl UserRole {
    /// Check if this role has admin privileges
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

/// Instance permission levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum InstancePermission {
    /// Full control: send messages, manage config, delete instance
    Owner,
    /// Operational access: send messages, view status (no delete/config)
    Operator,
    /// Read-only: view status, chat history
    Viewer,
}

impl InstancePermission {
    /// Check if permission allows sending messages
    pub fn can_send(&self) -> bool {
        matches!(self, InstancePermission::Owner | InstancePermission::Operator)
    }

    /// Check if permission allows modifying instance config
    pub fn can_modify(&self) -> bool {
        matches!(self, InstancePermission::Owner)
    }

    /// Check if permission allows deleting the instance
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

// =============================================================================
// User Records
// =============================================================================

/// User record stored in database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: String,
    pub username: String,
    /// Hashed API key (not returned in API responses)
    #[serde(skip_serializing)]
    pub api_key: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// User info returned in API responses (without sensitive fields).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<UserRecord> for UserInfo {
    fn from(record: UserRecord) -> Self {
        Self {
            id: record.id,
            username: record.username,
            role: record.role,
            is_active: record.is_active,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

/// Instance ownership record.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceOwnerRecord {
    pub user_id: String,
    pub instance_id: String,
    pub permission: InstancePermission,
    pub created_at: Option<String>,
}

// =============================================================================
// API Request/Response Types
// =============================================================================

/// Request to create a new user.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    /// Unique username
    pub username: String,
    /// User role (admin or user)
    #[serde(default = "default_user_role")]
    pub role: UserRole,
}

fn default_user_role() -> UserRole {
    UserRole::User
}

/// Response after creating a user (includes API key only once).
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateUserResponse {
    pub user: UserInfo,
    /// API key (shown only on creation, store securely!)
    pub api_key: String,
}

/// Request to update a user.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    /// New username (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// New role (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<UserRole>,
    /// Active status (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// Request to assign instance permission to a user.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignInstanceRequest {
    pub user_id: String,
    pub instance_id: String,
    #[serde(default = "default_permission")]
    pub permission: InstancePermission,
}

fn default_permission() -> InstancePermission {
    InstancePermission::Owner
}

/// Response for regenerating API key.
#[derive(Debug, Serialize, ToSchema)]
pub struct RegenerateApiKeyResponse {
    /// New API key (store securely!)
    pub api_key: String,
}

/// List users response.
#[derive(Debug, Serialize, ToSchema)]
pub struct ListUsersResponse {
    pub users: Vec<UserInfo>,
    pub total: usize,
}

/// User's instance permissions response.
#[derive(Debug, Serialize, ToSchema)]
pub struct UserInstancesResponse {
    pub instances: Vec<InstanceOwnerRecord>,
}
