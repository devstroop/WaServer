use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::user::{InstancePermission, UserRole};

// =============================================================================
// Authenticated User Context
// =============================================================================

/// Represents how a request was authenticated.
/// Supports both static secret key (superadmin) and user API keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthenticatedUser {
    /// Authenticated via static secret key from config `[auth].secret_key`
    /// Has full superadmin access to everything.
    Secret,
    /// Authenticated via user API key from database.
    User {
        id: String,
        username: String,
        role: UserRole,
    },
}

impl AuthenticatedUser {
    /// Get a display name for logging
    pub fn display_name(&self) -> String {
        match self {
            AuthenticatedUser::Secret => "superadmin".to_string(),
            AuthenticatedUser::User { username, .. } => username.clone(),
        }
    }

    /// Check if this user has admin privileges (secret or admin role)
    pub fn is_admin(&self) -> bool {
        match self {
            AuthenticatedUser::Secret => true,
            AuthenticatedUser::User { role, .. } => role.is_admin(),
        }
    }

    /// Check if this user is the superadmin (secret key)
    pub fn is_superadmin(&self) -> bool {
        matches!(self, AuthenticatedUser::Secret)
    }

    /// Get user ID if authenticated as a user
    pub fn user_id(&self) -> Option<&str> {
        match self {
            AuthenticatedUser::Secret => None,
            AuthenticatedUser::User { id, .. } => Some(id),
        }
    }

    /// Get user role
    pub fn role(&self) -> Option<&UserRole> {
        match self {
            AuthenticatedUser::Secret => None,
            AuthenticatedUser::User { role, .. } => Some(role),
        }
    }

    /// Check if user can access an instance with given permission requirement.
    /// Superadmin and admin users can access everything.
    pub fn can_access_instance(
        &self,
        permission: Option<InstancePermission>,
        required: InstancePermission,
    ) -> bool {
        // Superadmin can do anything
        if self.is_superadmin() {
            return true;
        }

        // Admin role can do anything
        if self.is_admin() {
            return true;
        }

        // Check specific permission
        match permission {
            Some(perm) => match required {
                InstancePermission::Viewer => true, // Any permission grants view access
                InstancePermission::Operator => perm.can_send(),
                InstancePermission::Owner => perm.can_modify(),
            },
            None => false, // No permission = no access
        }
    }
}

impl std::fmt::Display for AuthenticatedUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Response for authentication status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthStatusResponse {
    /// Whether the user is authenticated
    pub authenticated: bool,
    /// Status reason: "authenticated", "not_authenticated", "checking"
    pub status: String,
    /// Sender ID if authenticated (phone number)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
}

/// Response containing QR code data
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct QrCodeResponse {
    /// Base64 encoded QR code image
    pub qrcode: String,
}

/// Request for phone login
#[derive(Debug, Deserialize, ToSchema)]
pub struct PhoneLoginRequest {
    /// Phone number with country code (e.g., "+1234567890")
    pub phone: String,
}

/// Response for phone authentication
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PhoneAuthResponse {
    /// Authentication code to enter on phone
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Generic success response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SuccessResponse {
    /// Success message
    pub message: String,
}
