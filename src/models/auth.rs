use serde::{Deserialize, Serialize};
#[cfg(feature = "api")]
use utoipa::ToSchema;

/// Response for authentication status
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct AuthStatusResponse {
    /// Whether the user is authenticated
    pub authorized: bool,
    /// Sender ID if authenticated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<String>,
}

/// Response containing QR code data
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct QrCodeResponse {
    /// Base64 encoded QR code image
    pub qrcode: String,
}

/// Request for phone login
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct PhoneLoginRequest {
    /// Phone number with country code (e.g., "+1234567890")
    pub phone: String,
}

/// Response for phone authentication
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct PhoneAuthResponse {
    /// Authentication code to enter on phone
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Generic success response
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct SuccessResponse {
    /// Success message
    pub message: String,
}

// Note: ErrorResponse is defined in chat.rs to avoid duplication
