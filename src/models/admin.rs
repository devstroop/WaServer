//! Admin Models - Users, Roles, and Permissions
//!
//! Data structures for user management and RBAC.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// =============================================================================
// User Models
// =============================================================================

/// User account
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    /// Unique user ID
    pub id: Uuid,
    /// Username (unique)
    pub username: String,
    /// Email address (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Whether the user is active
    pub active: bool,
    /// Assigned roles
    pub roles: Vec<String>,
    /// Created timestamp (ISO 8601)
    pub created_at: String,
    /// Last updated timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Request to create a new user
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    /// Username (unique, min 3 characters)
    pub username: String,
    /// Password (min 8 characters)
    pub password: String,
    /// Email address (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Display name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Roles to assign (optional)
    #[serde(default)]
    pub roles: Vec<String>,
}

/// Request to update a user
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    /// New password (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Email address (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Display name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Whether the user is active (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Roles to assign (replaces existing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
}

// =============================================================================
// Role Models
// =============================================================================

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Role {
    /// Unique role ID
    pub id: String,
    /// Role name (e.g., "admin", "operator", "viewer")
    pub name: String,
    /// Role description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Permissions assigned to this role
    pub permissions: Vec<String>,
    /// Whether this is a system role (cannot be deleted)
    pub system: bool,
    /// Created timestamp (ISO 8601)
    pub created_at: String,
}

/// Request to create a new role
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRoleRequest {
    /// Role ID (unique, lowercase alphanumeric with hyphens)
    pub id: String,
    /// Role name
    pub name: String,
    /// Role description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Permissions to assign
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// Request to update a role
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRoleRequest {
    /// Role name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Role description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Permissions to assign (replaces existing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
}

// =============================================================================
// Permission Models
// =============================================================================

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Permission {
    /// Permission ID (e.g., "accounts:read", "messages:send")
    pub id: String,
    /// Permission name
    pub name: String,
    /// Permission description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Resource this permission applies to
    pub resource: String,
    /// Action (e.g., "read", "write", "delete", "admin")
    pub action: String,
}

// =============================================================================
// Default Permissions
// =============================================================================

impl Permission {
    /// Get all available permissions
    pub fn all() -> Vec<Permission> {
        vec![
            // Account permissions
            Permission {
                id: "accounts:read".to_string(),
                name: "Read Accounts".to_string(),
                description: Some("View account list and details".to_string()),
                resource: "accounts".to_string(),
                action: "read".to_string(),
            },
            Permission {
                id: "accounts:write".to_string(),
                name: "Manage Accounts".to_string(),
                description: Some("Create, update, start, stop accounts".to_string()),
                resource: "accounts".to_string(),
                action: "write".to_string(),
            },
            Permission {
                id: "accounts:delete".to_string(),
                name: "Delete Accounts".to_string(),
                description: Some("Delete accounts and their data".to_string()),
                resource: "accounts".to_string(),
                action: "delete".to_string(),
            },
            // Message permissions
            Permission {
                id: "messages:read".to_string(),
                name: "Read Messages".to_string(),
                description: Some("View chats and messages".to_string()),
                resource: "messages".to_string(),
                action: "read".to_string(),
            },
            Permission {
                id: "messages:send".to_string(),
                name: "Send Messages".to_string(),
                description: Some("Send messages to contacts".to_string()),
                resource: "messages".to_string(),
                action: "write".to_string(),
            },
            // User management permissions
            Permission {
                id: "users:read".to_string(),
                name: "Read Users".to_string(),
                description: Some("View user list and details".to_string()),
                resource: "users".to_string(),
                action: "read".to_string(),
            },
            Permission {
                id: "users:write".to_string(),
                name: "Manage Users".to_string(),
                description: Some("Create and update users".to_string()),
                resource: "users".to_string(),
                action: "write".to_string(),
            },
            Permission {
                id: "users:delete".to_string(),
                name: "Delete Users".to_string(),
                description: Some("Delete user accounts".to_string()),
                resource: "users".to_string(),
                action: "delete".to_string(),
            },
            // Role management permissions
            Permission {
                id: "roles:read".to_string(),
                name: "Read Roles".to_string(),
                description: Some("View roles and permissions".to_string()),
                resource: "roles".to_string(),
                action: "read".to_string(),
            },
            Permission {
                id: "roles:write".to_string(),
                name: "Manage Roles".to_string(),
                description: Some("Create and update roles".to_string()),
                resource: "roles".to_string(),
                action: "write".to_string(),
            },
            // System permissions
            Permission {
                id: "system:admin".to_string(),
                name: "System Admin".to_string(),
                description: Some("Full system administration access".to_string()),
                resource: "system".to_string(),
                action: "admin".to_string(),
            },
        ]
    }
}
