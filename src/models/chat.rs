use serde::{Deserialize, Serialize};
#[cfg(feature = "api")]
use utoipa::ToSchema;

/// Request for sending a message
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct SendMessageRequest {
    /// Recipient phone number
    pub phone: String,
    /// Message text (optional if sending file)
    pub text: Option<String>,
}

/// Multipart form schema for Swagger documentation
#[cfg(feature = "api")]
#[derive(Debug, ToSchema)]
#[schema(title = "SendMessageForm")]
pub struct SendMessageMultipartForm {
    /// Recipient phone number (required)
    #[schema(example = "1234567890")]
    pub phone: String,

    /// Message text (optional if file is provided)
    #[schema(example = "Hello from Rust WhatsApp Engine!")]
    pub text: Option<String>,

    /// File attachment (optional if text is provided)  
    #[schema(format = "binary")]
    pub file: Option<String>,
}

/// Response for sending a message
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct SendMessageResponse {
    /// Status message
    pub status: String,
}

/// Chat message metadata
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct MessageMetadata {
    /// Message ID
    pub id: String,
    /// Timestamp when message was sent
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Whether message was delivered
    pub delivered: bool,
}
