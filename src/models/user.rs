//! User and RBAC Models
//!
//! Models for user management, roles, access tokens, and instance permissions.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Re-export canonical domain identity types
pub use crate::domain::identity::{InstancePermission, UserRole};

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

/// Request to register a new user (API).
#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterUserRequest {
    /// Unique username
    pub username: String,
    /// Optional email
    pub email: Option<String>,
    /// Password (will be hashed)
    pub password: String,
}

/// Request to login (API).
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
    /// Session token for API
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
