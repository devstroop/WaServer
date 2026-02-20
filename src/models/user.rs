//! User Models
//!
//! Types for user accounts, ownership, and access control.
//! This module provides the foundation for RBAC (Role-Based Access Control).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Unique identifier for a user (UUID)
pub type UserId = Uuid;

// =============================================================================
// User Entity
// =============================================================================

/// User account stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier (UUID)
    pub id: UserId,
    /// Login username (unique, case-insensitive)
    pub username: String,
    /// bcrypt hashed password (never serialized to API)
    #[serde(skip_serializing)]
    pub password_hash: String,
    /// Optional email for notifications
    pub email: Option<String>,
    /// Display name (defaults to username)
    pub display_name: Option<String>,
    /// Account active status
    pub is_active: bool,
    /// System administrator flag
    pub is_admin: bool,
    /// When account was created
    pub created_at: DateTime<Utc>,
    /// Last profile update
    pub updated_at: DateTime<Utc>,
    /// Last successful login
    pub last_login_at: Option<DateTime<Utc>>,
}

impl User {
    /// Create a new user with required fields
    pub fn new(username: &str, password_hash: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username: username.to_string(),
            password_hash: password_hash.to_string(),
            email: None,
            display_name: None,
            is_active: true,
            is_admin: false,
            created_at: now,
            updated_at: now,
            last_login_at: None,
        }
    }

    /// Create the first admin user
    pub fn new_admin(username: &str, password_hash: &str) -> Self {
        let mut user = Self::new(username, password_hash);
        user.is_admin = true;
        user
    }

    /// Get display name or fallback to username
    pub fn display_name_or_username(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.username)
    }
}

// =============================================================================
// API Response Types
// =============================================================================

/// User info returned by API (excludes sensitive fields)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserInfo {
    /// Unique user identifier (UUID)
    pub id: UserId,
    /// Username
    pub username: String,
    /// Email address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Account active status
    pub is_active: bool,
    /// System administrator
    pub is_admin: bool,
    /// When account was created
    pub created_at: DateTime<Utc>,
    /// Last login timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_at: Option<DateTime<Utc>>,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            is_active: user.is_active,
            is_admin: user.is_admin,
            created_at: user.created_at,
            last_login_at: user.last_login_at,
        }
    }
}

impl From<&User> for UserInfo {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            is_active: user.is_active,
            is_admin: user.is_admin,
            created_at: user.created_at,
            last_login_at: user.last_login_at,
        }
    }
}

/// List of users response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserListResponse {
    /// List of users
    pub users: Vec<UserInfo>,
    /// Total count
    pub total: usize,
}

// =============================================================================
// API Request Types
// =============================================================================

/// Request to create a new user
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    /// Username (min 3 characters, alphanumeric + underscore)
    pub username: String,
    /// Password (min 8 characters)
    pub password: String,
    /// Optional email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Optional display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Make this user an admin (requires admin privileges)
    #[serde(default)]
    pub is_admin: bool,
}

/// Request to update user profile
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    /// New email address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// New display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Active status (admin only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    /// Admin status (admin only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,
}

/// Query parameters for user list
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ListUsersQuery {
    /// Filter by active status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Filter by admin status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin: Option<bool>,
    /// Search by username or email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    /// Limit results
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Offset for pagination
    #[serde(default)]
    pub offset: usize,
}

fn default_limit() -> usize {
    50
}

// =============================================================================
// Instance Ownership
// =============================================================================

/// Instance ownership record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceOwnership {
    /// Instance ID
    pub instance_id: String,
    /// Owner user ID
    pub owner_id: UserId,
    /// Display name for the instance
    pub display_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Active status
    pub is_active: bool,
    /// When instance was created
    pub created_at: DateTime<Utc>,
    /// Last update
    pub updated_at: DateTime<Utc>,
}

/// Instance access grant for sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceAccess {
    /// Instance ID
    pub instance_id: String,
    /// User ID who has access
    pub user_id: UserId,
    /// Can view messages and chats
    pub can_read: bool,
    /// Can send messages
    pub can_send: bool,
    /// Can modify instance settings
    pub can_manage: bool,
    /// User who granted access
    pub granted_by: UserId,
    /// When access was granted
    pub granted_at: DateTime<Utc>,
    /// Optional expiry
    pub expires_at: Option<DateTime<Utc>>,
}

/// Instance info with ownership details (API response)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceWithOwner {
    /// Instance ID
    pub instance_id: String,
    /// Owner info
    pub owner: UserInfo,
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Whether current user is owner
    pub is_owner: bool,
    /// Current user's permissions on this instance
    pub permissions: InstancePermissions,
    /// When instance was created
    pub created_at: DateTime<Utc>,
}

/// Permissions a user has on an instance
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstancePermissions {
    /// Can view messages and chats
    pub can_read: bool,
    /// Can send messages
    pub can_send: bool,
    /// Can modify instance settings
    pub can_manage: bool,
    /// Can delete the instance
    pub can_delete: bool,
    /// Can share access with other users
    pub can_share: bool,
}

impl InstancePermissions {
    /// Full permissions (for owner)
    pub fn owner() -> Self {
        Self {
            can_read: true,
            can_send: true,
            can_manage: true,
            can_delete: true,
            can_share: true,
        }
    }

    /// Admin permissions
    pub fn admin() -> Self {
        Self::owner()
    }

    /// Read-only permissions
    pub fn read_only() -> Self {
        Self {
            can_read: true,
            can_send: false,
            can_manage: false,
            can_delete: false,
            can_share: false,
        }
    }

    /// No access
    pub fn none() -> Self {
        Self {
            can_read: false,
            can_send: false,
            can_manage: false,
            can_delete: false,
            can_share: false,
        }
    }

    /// From instance access record
    pub fn from_access(access: &InstanceAccess) -> Self {
        Self {
            can_read: access.can_read,
            can_send: access.can_send,
            can_manage: access.can_manage,
            can_delete: false, // Only owner can delete
            can_share: false,  // Only owner can share
        }
    }
}

// =============================================================================
// Request Types for Instance Access
// =============================================================================

/// Request to grant instance access to another user
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct GrantInstanceAccessRequest {
    /// User ID to grant access to
    pub user_id: UserId,
    /// Can view messages and chats
    #[serde(default = "default_true")]
    pub can_read: bool,
    /// Can send messages
    #[serde(default)]
    pub can_send: bool,
    /// Can modify instance settings
    #[serde(default)]
    pub can_manage: bool,
    /// Optional expiry (RFC 3339 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

fn default_true() -> bool {
    true
}

/// Request to update instance access
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateInstanceAccessRequest {
    /// Can view messages and chats
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_read: Option<bool>,
    /// Can send messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_send: Option<bool>,
    /// Can modify instance settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_manage: Option<bool>,
    /// Optional expiry (RFC 3339 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Instance access info for API response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceAccessInfo {
    /// User who has access
    pub user: UserInfo,
    /// Permissions
    pub permissions: InstancePermissions,
    /// Who granted access
    pub granted_by: UserInfo,
    /// When access was granted
    pub granted_at: DateTime<Utc>,
    /// Optional expiry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// List of users with access to an instance
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceAccessListResponse {
    /// Instance ID
    pub instance_id: String,
    /// Owner
    pub owner: UserInfo,
    /// Users with shared access
    pub shared_with: Vec<InstanceAccessInfo>,
}

// =============================================================================
// Validation
// =============================================================================

/// Validate username format
pub fn validate_username(username: &str) -> Result<(), String> {
    let username = username.trim();
    
    if username.len() < 3 {
        return Err("Username must be at least 3 characters".to_string());
    }
    
    if username.len() > 32 {
        return Err("Username cannot exceed 32 characters".to_string());
    }
    
    if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Username can only contain letters, numbers, and underscores".to_string());
    }
    
    if username.starts_with('_') || username.ends_with('_') {
        return Err("Username cannot start or end with underscore".to_string());
    }
    
    Ok(())
}

/// Validate email format (basic check)
pub fn validate_email(email: &str) -> Result<(), String> {
    let email = email.trim();
    
    if email.is_empty() {
        return Err("Email cannot be empty".to_string());
    }
    
    if !email.contains('@') || !email.contains('.') {
        return Err("Invalid email format".to_string());
    }
    
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err("Invalid email format".to_string());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        // Valid usernames
        assert!(validate_username("admin").is_ok());
        assert!(validate_username("user_123").is_ok());
        assert!(validate_username("John_Doe").is_ok());
        
        // Invalid usernames
        assert!(validate_username("ab").is_err()); // Too short
        assert!(validate_username("a".repeat(33).as_str()).is_err()); // Too long
        assert!(validate_username("user@name").is_err()); // Invalid char
        assert!(validate_username("_user").is_err()); // Starts with _
        assert!(validate_username("user_").is_err()); // Ends with _
    }

    #[test]
    fn test_validate_email() {
        // Valid emails
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("test.user@domain.org").is_ok());
        
        // Invalid emails
        assert!(validate_email("").is_err());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@domain.com").is_err());
        assert!(validate_email("user@").is_err());
    }
}
