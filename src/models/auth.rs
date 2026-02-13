use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Response for authentication status
#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
    /// Whether local auth is enabled
    pub local_auth_enabled: bool,
    /// Whether user is logged in (has valid token)
    pub logged_in: bool,
    /// Username if logged in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

// Note: ErrorResponse is defined in chat.rs to avoid duplication
