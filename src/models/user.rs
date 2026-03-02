//! User and RBAC Models
//!
//! Models for user management, roles, access tokens, and instance permissions.

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
    pub email: Option<String>,
    /// Password hash (not returned in API responses)
    #[serde(skip_serializing)]
    pub password_hash: String,
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
    pub email: Option<String>,
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
            email: record.email,
            role: record.role,
            is_active: record.is_active,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

// =============================================================================
// Access Token Records
// =============================================================================

/// Access token record for API authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenRecord {
    pub id: String,
    pub user_id: String,
    pub name: String,
    /// Token hash (not returned in API responses)
    #[serde(skip_serializing)]
    pub token_hash: String,
    pub expires_at: Option<String>,
    pub last_used: Option<String>,
    pub created_at: Option<String>,
}

/// Access token info returned in API responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccessTokenInfo {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub expires_at: Option<String>,
    pub last_used: Option<String>,
    pub created_at: Option<String>,
}

impl From<AccessTokenRecord> for AccessTokenInfo {
    fn from(record: AccessTokenRecord) -> Self {
        Self {
            id: record.id,
            user_id: record.user_id,
            name: record.name,
            expires_at: record.expires_at,
            last_used: record.last_used,
            created_at: record.created_at,
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

/// Request to register a new user (web UI).
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterUserRequest {
    /// Unique username
    pub username: String,
    /// Optional email
    pub email: Option<String>,
    /// Password (will be hashed)
    pub password: String,
}

/// Request to login (web UI).
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// Username or email
    pub username: String,
    /// Password
    pub password: String,
}

/// Login response with session token.
#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub user: UserInfo,
    /// Session token for web UI
    pub token: String,
    /// Token expiration time
    pub expires_at: String,
}

/// Request to create a new user (admin).
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    /// Unique username
    pub username: String,
    /// Optional email
    pub email: Option<String>,
    /// Password (will be hashed)
    pub password: String,
    /// User role (admin or user)
    #[serde(default = "default_user_role")]
    pub role: UserRole,
}

fn default_user_role() -> UserRole {
    UserRole::User
}

/// Response after creating a user.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateUserResponse {
    pub user: UserInfo,
}

/// Request to update a user.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    /// New username (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// New email (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// New password (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// New role (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<UserRole>,
    /// Active status (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// Request to create an access token.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAccessTokenRequest {
    /// Token name/label
    #[serde(default = "default_token_name")]
    pub name: String,
    /// Expiration in days (optional, null = never expires)
    pub expires_in_days: Option<u32>,
}

fn default_token_name() -> String {
    "default".to_string()
}

/// Response after creating an access token (includes token only once).
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateAccessTokenResponse {
    pub token_info: AccessTokenInfo,
    /// Access token (shown only on creation, store securely!)
    pub access_token: String,
}

/// List access tokens response.
#[derive(Debug, Serialize, ToSchema)]
pub struct ListAccessTokensResponse {
    pub tokens: Vec<AccessTokenInfo>,
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
