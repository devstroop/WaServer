use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

// ============================================================================
// Request/Response Models (send-only)
// ============================================================================

/// Query parameters for sending a message
#[derive(Debug, Deserialize, IntoParams)]
pub struct SendMessageParams {
    /// Recipient phone number (e.g., 919876543210)
    pub phone: String,
    /// Message text or caption for file attachment (optional if sending file only)
    #[serde(default)]
    pub text: Option<String>,
}

/// Request body for file upload (multipart/form-data).
/// Use this to upload file attachments.
#[derive(Debug, ToSchema)]
pub struct SendMessageRequest {
    /// File attachment (image, video, document, etc.)
    #[schema(format = Binary, nullable = true)]
    pub file: Option<String>,
}

/// Response for sending a message
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendMessageResponse {
    /// Status message
    pub status: String,
    /// Message ID for tracking
    pub message_id: String,
}

/// Standardized error response returned by all API endpoints
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    /// Error code (e.g. "not_found", "browser_not_running")
    pub error: String,
    /// Human-readable error description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ErrorResponse {
    /// Create an error response with just an error string
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: None,
        }
    }

    /// Create an error response with a code and human-readable message
    pub fn with_message(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: Some(message.into()),
        }
    }
}
