use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request for sending a message
#[derive(Debug, Deserialize, ToSchema)]
pub struct SendMessageRequest {
    /// Recipient phone number
    pub phone: String,
    /// Message text (optional if sending file)
    pub text: Option<String>,
}

/// Response for sending a message
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SendMessageResponse {
    /// Status message
    pub status: String,
}

/// Chat message metadata
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MessageMetadata {
    /// Message ID
    pub id: String,
    /// Timestamp when message was sent
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Whether message was delivered
    pub delivered: bool,
}
