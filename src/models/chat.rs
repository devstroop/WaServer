use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// Shared chat models
// ============================================================================

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
