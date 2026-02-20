use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// =============================================================================
// Authenticated User Context
// =============================================================================

/// Represents how a request was authenticated
/// 
/// This allows handlers to differentiate between:
/// - **Secret**: Static config-based secret token (external scripts, CI/CD pipelines)
/// - **LocalUser**: JWT-based user authentication (web UI, MCP, requires username/password)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthenticatedUser {
    /// Authenticated via static secret key from config `[auth].secret_key`
    /// Used for: External scripts, CI/CD pipelines, simple integrations
    Secret,
    
    /// Authenticated via JWT token from user login
    /// Contains the username of the authenticated user
    /// Used for: Web UI, MCP clients, user-specific access control
    LocalUser {
        /// The username extracted from the JWT token
        username: String,
    },
}

impl AuthenticatedUser {
    /// Check if authenticated via static secret token
    pub fn is_secret(&self) -> bool {
        matches!(self, AuthenticatedUser::Secret)
    }
    
    /// Check if authenticated via local user JWT
    pub fn is_local_user(&self) -> bool {
        matches!(self, AuthenticatedUser::LocalUser { .. })
    }
    
    /// Get username if authenticated via local user
    pub fn username(&self) -> Option<&str> {
        match self {
            AuthenticatedUser::LocalUser { username } => Some(username),
            AuthenticatedUser::Secret => None,
        }
    }
    
    /// Get a display name for logging
    pub fn display_name(&self) -> String {
        match self {
            AuthenticatedUser::Secret => "secret".to_string(),
            AuthenticatedUser::LocalUser { username } => format!("user:{}", username),
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

// =============================================================================
// Local Authentication Models
// =============================================================================

/// Request for local user login
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

/// Response for successful login
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    /// Access token (JWT)
    pub access_token: String,
    /// Refresh token for getting new access tokens
    pub refresh_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Access token expiry in seconds
    pub expires_in: i64,
    /// Username of the logged in user
    pub username: String,
}

/// Request for token refresh
#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    /// Refresh token
    pub refresh_token: String,
}

/// Response for token refresh
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RefreshTokenResponse {
    /// New access token (JWT)
    pub access_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Access token expiry in seconds
    pub expires_in: i64,
}

// =============================================================================
// Initial Setup Models
// =============================================================================

/// Request for initial admin setup
#[derive(Debug, Deserialize, ToSchema)]
pub struct SetupRequest {
    /// One-time setup token (displayed in server console on first run)
    pub setup_token: String,
    /// Username for the admin account (min 3 characters)
    pub username: String,
    /// Password for the admin account (min 8 characters)
    pub password: String,
}

/// Response for setup status check
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SetupStatusResponse {
    /// Whether initial setup is required (no admin user exists)
    pub needs_setup: bool,
    /// Message for the user
    pub message: String,
}

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (username)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Token type: "access" or "refresh"
    pub token_type: String,
}

/// Local auth status response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LocalAuthStatusResponse {
    /// Whether user is logged in (has valid token)
    pub logged_in: bool,
    /// Username if logged in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

// Note: ErrorResponse is defined in chat.rs to avoid duplication
