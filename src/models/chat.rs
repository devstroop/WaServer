use serde::{Deserialize, Serialize};
#[cfg(feature = "api")]
use utoipa::ToSchema;

// ============================================================================
// Request/Response Models
// ============================================================================

/// Request for sending a message
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct SendMessageRequest {
    /// Recipient phone number
    pub phone: String,
    /// Message text (optional if sending file)
    pub text: Option<String>,
}

/// Response for sending a message
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct SendMessageResponse {
    /// Status message
    pub status: String,
    /// Message ID for tracking
    pub message_id: String,
}

/// Error response
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
}

// ============================================================================
// Chat List Models (DOM-based)
// ============================================================================

/// A chat/conversation from the WhatsApp sidebar
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct ChatInfo {
    /// Chat ID (phone@c.us or group ID)
    pub id: String,
    /// Contact or group name
    pub name: String,
    /// Last message preview
    pub last_message: Option<String>,
    /// Last message timestamp (human readable)
    pub timestamp: Option<String>,
    /// Number of unread messages
    pub unread_count: u32,
    /// Whether this is a group chat
    pub is_group: bool,
    /// Profile picture URL (if available)
    pub avatar_url: Option<String>,
}

/// Response for listing chats
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct ChatListResponse {
    /// List of chats
    pub chats: Vec<ChatInfo>,
    /// Total number of chats found
    pub total: usize,
}

// ============================================================================
// Unified Message Model - Contains ALL message information with status
// NO separate queue - everything is a Message with status tracking
// ============================================================================

/// A complete message with all information and status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct Message {
    /// Unique message ID
    pub id: String,
    /// Sender (phone number or "me" for outgoing)
    pub sender: String,
    /// Recipient (phone number or group JID)
    pub recipient: String,
    /// Sender display name
    pub sender_name: Option<String>,
    /// Message text content
    pub text: Option<String>,
    /// Whether this is a group message
    pub is_group: bool,
    /// Message status: pending, processing, sent, delivered, read, failed, received
    pub status: String,
    /// Media type: none, image, video, document, voice, sticker
    pub media_type: String,
    /// Local file path for media
    pub media_path: Option<String>,
    /// Original filename for documents
    pub media_filename: Option<String>,
    /// File extension (e.g., "PDF", "TOML")
    pub media_extension: Option<String>,
    /// File size in bytes
    pub media_size: Option<i64>,
    /// Duration in seconds (for voice/video)
    pub media_duration: Option<i32>,
    /// Quoted message ID (for replies)
    pub quoted_message_id: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Retry count
    pub retry_count: i32,
    /// WhatsApp timestamp
    pub message_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// When we created this record
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the message was processed/sent
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Response for listing messages
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct MessageListResponse {
    /// Chat ID
    pub chat_id: String,
    /// Chat name
    pub chat_name: Option<String>,
    /// List of messages (from DOM or database)
    pub messages: Vec<MessageInfo>,
    /// Total messages retrieved
    pub total: usize,
    /// Whether more messages are available
    pub has_more: bool,
}

/// Query params for message listing
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct MessageQueryParams {
    /// Maximum number of messages to retrieve
    pub limit: Option<u32>,
    /// Filter by status (pending, sent, delivered, failed, etc.)
    pub status: Option<String>,
    /// Load older messages (scroll up)
    pub load_more: Option<bool>,
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
    #[schema(example = "Hello from WhatsApp Engine!")]
    pub text: Option<String>,

    /// File attachment (optional if text is provided)  
    #[schema(format = "binary")]
    pub file: Option<String>,
}

// ============================================================================
// Conversions from Database Models
// ============================================================================

impl From<crate::services::database::Message> for Message {
    fn from(msg: crate::services::database::Message) -> Self {
        Message {
            id: msg.id,
            sender: msg.sender,
            recipient: msg.recipient,
            sender_name: msg.sender_name,
            text: msg.text,
            is_group: msg.is_group,
            status: msg.status.to_string(),
            media_type: msg.media_type.to_string(),
            media_path: msg.media_path,
            media_filename: msg.media_filename,
            media_extension: msg.media_extension,
            media_size: msg.media_size,
            media_duration: msg.media_duration,
            quoted_message_id: msg.quoted_message_id,
            error: msg.error,
            retry_count: msg.retry_count,
            message_timestamp: msg.message_timestamp,
            created_at: msg.created_at,
            processed_at: msg.processed_at,
        }
    }
}

/// Simplified message info for DOM-scraped messages (before DB storage)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(ToSchema))]
pub struct MessageInfo {
    /// Message ID
    pub id: String,
    /// Whether message was sent by me
    pub from_me: bool,
    /// Sender name (for groups)
    pub sender: Option<String>,
    /// Message text content
    pub text: Option<String>,
    /// Message type (chat, image, video, document, etc.)
    pub message_type: String,
    /// Timestamp (human readable)
    pub timestamp: Option<String>,
    /// Unix timestamp (for sorting)
    pub timestamp_unix: Option<i64>,
    /// Message status
    pub status: Option<String>,
    /// Media URL or file info
    pub media_info: Option<String>,
}
