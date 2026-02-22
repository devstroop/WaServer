use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// =============================================================================
// Authenticated User Context
// =============================================================================

/// Represents how a request was authenticated.
/// With secret-key-only auth, all requests are authenticated via the static secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthenticatedUser {
    /// Authenticated via static secret key from config `[auth].secret_key`
    Secret,
}

impl AuthenticatedUser {
    /// Check if authenticated via static secret token
    pub fn is_secret(&self) -> bool {
        matches!(self, AuthenticatedUser::Secret)
    }

    /// Check if user is an admin (secret tokens are always admin)
    pub fn is_admin(&self) -> bool {
        true
    }

    /// Get a display name for logging
    pub fn display_name(&self) -> String {
        "secret".to_string()
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
