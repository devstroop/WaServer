use crate::models::message as db;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// Request/Response Models
// ============================================================================

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

// ============================================================================
// Chat List Models (DOM-based)
// ============================================================================

/// A chat/conversation from the WhatsApp sidebar
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChatInfo {
    /// Chat ID (phone@c.us or group@g.us)
    pub id: String,
    /// Contact or group name
    pub name: String,
    /// Last message preview
    pub last_message: Option<String>,
    /// Last message sender (for group chats)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_sender: Option<String>,
    /// Last message timestamp (human readable like "10:30 AM", "Yesterday", "1/15/26")
    pub timestamp: Option<String>,
    /// Number of unread messages
    pub unread_count: u32,
    /// Whether this is a group chat
    pub is_group: bool,
    /// Profile picture URL (if available)
    pub avatar_url: Option<String>,
    /// Whether the chat is pinned to top
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pinned: Option<bool>,
    /// Whether notifications are muted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_muted: Option<bool>,
}

/// Response for listing chats
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatListResponse {
    /// List of chats
    pub chats: Vec<ChatInfo>,
    /// Total number of chats found
    pub total: usize,
}

/// Response for listing messages
#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
#[derive(Debug, Deserialize, ToSchema)]
pub struct MessageQueryParams {
    /// Maximum number of messages to retrieve
    pub limit: Option<u32>,
    /// Filter by status (pending, sent, delivered, failed, etc.)
    pub status: Option<String>,
    /// Load older messages (scroll up)
    pub load_more: Option<bool>,
}

/// Multipart form schema for Swagger documentation
#[derive(Debug, ToSchema)]
#[schema(title = "SendMessageForm")]
pub struct SendMessageMultipartForm {
    /// Recipient phone number (required)
    #[schema(example = "1234567890")]
    pub phone: String,

    /// Message text (optional if file is provided)
    #[schema(example = "Hello from WAS!")]
    pub text: Option<String>,

    /// File attachment (optional if text is provided)  
    #[schema(format = "binary")]
    pub file: Option<String>,
}

/// Simplified message info for DOM-scraped messages (before DB storage)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

/// Full message details (from database) - API response DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Message {
    /// Unique message ID
    pub id: String,
    /// Sender JID/phone
    pub sender: String,
    /// Recipient JID/phone
    pub recipient: String,
    /// Sender display name
    pub sender_name: Option<String>,
    /// Message text content
    pub text: Option<String>,
    /// Whether this is a group message
    pub is_group: bool,
    /// Message status
    pub status: String,
    /// Media type (none, image, video, document, voice)
    pub media_type: String,
    /// Local file path for media
    pub media_path: Option<String>,
    /// Original filename for documents
    pub media_filename: Option<String>,
    /// File extension/type
    pub media_extension: Option<String>,
    /// File size in bytes
    pub media_size: Option<i64>,
    /// Duration in seconds (for voice/video)
    pub media_duration: Option<i32>,
    /// Quoted message ID (for replies)
    pub quoted_message_id: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// WhatsApp timestamp
    pub message_timestamp: Option<DateTime<Utc>>,
    /// When we created this record
    pub created_at: DateTime<Utc>,
    /// When the message was processed
    pub processed_at: Option<DateTime<Utc>>,
}

impl From<db::Message> for Message {
    fn from(msg: db::Message) -> Self {
        Self {
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
            message_timestamp: msg.message_timestamp,
            created_at: msg.created_at,
            processed_at: msg.processed_at,
        }
    }
}

// ============================================================================
// Typing Indicator Models
// ============================================================================

/// Typing state enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TypingState {
    /// User is typing
    Composing,
    /// User stopped typing
    Paused,
}

/// Request to send typing indicator
#[derive(Debug, Deserialize, ToSchema)]
pub struct TypingRequest {
    /// Chat ID (phone number or group ID)
    pub chat_id: String,
    /// Typing state
    pub state: TypingState,
}

/// Response for typing indicator
#[derive(Debug, Serialize, ToSchema)]
pub struct TypingResponse {
    /// Success status
    pub success: bool,
    /// Chat ID where typing indicator was sent
    pub chat_id: String,
    /// The typing state that was set
    pub state: TypingState,
}

// ============================================================================
// Presence/Online Status Models
// ============================================================================

/// Online presence status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PresenceStatus {
    /// User is online
    Online,
    /// User is offline
    Offline,
    /// Status unknown/unavailable
    Unknown,
}

/// Presence info for a contact
#[derive(Debug, Serialize, ToSchema)]
pub struct PresenceInfo {
    /// Chat/Contact ID
    pub chat_id: String,
    /// Presence status
    pub status: PresenceStatus,
    /// Last seen timestamp (if available)
    pub last_seen: Option<String>,
    /// Whether "last seen" is hidden by privacy settings
    pub last_seen_hidden: bool,
}

/// Request to subscribe to presence updates
#[derive(Debug, Deserialize, ToSchema)]
pub struct PresenceSubscribeRequest {
    /// Chat ID to subscribe to
    pub chat_id: String,
}

// ============================================================================
// Group Management Models
// ============================================================================

/// Group participant info
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GroupParticipant {
    /// Participant JID
    pub id: String,
    /// Display name
    pub name: Option<String>,
    /// Phone number
    pub phone: Option<String>,
    /// Whether this is an admin
    pub is_admin: bool,
    /// Whether this is the group owner/creator
    pub is_owner: bool,
}

/// Detailed group information
#[derive(Debug, Serialize, ToSchema)]
pub struct GroupInfo {
    /// Group JID
    pub id: String,
    /// Group name
    pub name: String,
    /// Group description
    pub description: Option<String>,
    /// Group profile picture URL
    pub avatar_url: Option<String>,
    /// Group creation timestamp
    pub created_at: Option<String>,
    /// Group creator JID
    pub created_by: Option<String>,
    /// Participant count
    pub participant_count: u32,
    /// List of participants
    pub participants: Vec<GroupParticipant>,
    /// Whether only admins can send messages
    pub is_announce: bool,
    /// Whether only admins can edit group info
    pub is_locked: bool,
    /// Invite link (if available)
    pub invite_link: Option<String>,
}

/// Request to create a group
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateGroupRequest {
    /// Group name (max 25 characters)
    pub name: String,
    /// List of participant phone numbers
    pub participants: Vec<String>,
}

/// Request to update group info
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateGroupRequest {
    /// New group name (optional)
    pub name: Option<String>,
    /// New description (optional)
    pub description: Option<String>,
}

/// Request to manage group participants
#[derive(Debug, Deserialize, ToSchema)]
pub struct GroupParticipantsRequest {
    /// List of phone numbers to add/remove
    pub participants: Vec<String>,
}

// ============================================================================
// Contact Info Models
// ============================================================================

/// Contact profile information
#[derive(Debug, Serialize, ToSchema)]
pub struct ContactInfo {
    /// Contact JID
    pub id: String,
    /// Display name
    pub name: Option<String>,
    /// Push name (name set by the contact themselves)
    pub push_name: Option<String>,
    /// Phone number
    pub phone: Option<String>,
    /// Profile picture URL
    pub avatar_url: Option<String>,
    /// Status/About text
    pub status: Option<String>,
    /// Whether this is a business account
    pub is_business: bool,
    /// Business name (if business account)
    pub business_name: Option<String>,
    /// Business category
    pub business_category: Option<String>,
    /// Whether the contact is blocked
    pub is_blocked: bool,
}

// ============================================================================
// Message Reaction Models
// ============================================================================

/// Request to send a reaction
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReactionRequest {
    /// Message ID to react to
    pub message_id: String,
    /// Emoji reaction (empty string to remove)
    #[schema(example = "👍")]
    pub emoji: String,
}

/// Response for reaction
#[derive(Debug, Serialize, ToSchema)]
pub struct ReactionResponse {
    /// Success status
    pub success: bool,
}

/// Reaction info on a message
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageReaction {
    /// Emoji used
    pub emoji: String,
    /// Who sent the reaction
    pub sender: String,
    /// Sender name
    pub sender_name: Option<String>,
    /// When the reaction was sent
    pub timestamp: Option<String>,
}

// ============================================================================
// Reply/Quote Models
// ============================================================================

/// Request to send a reply/quoted message
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReplyMessageRequest {
    /// Phone number or chat ID
    pub chat_id: String,
    /// Message ID to reply to
    pub quoted_message_id: String,
    /// Reply text
    pub text: String,
}

// ============================================================================
// Read Receipt Models
// ============================================================================

/// Request to mark messages as read
#[derive(Debug, Deserialize, ToSchema)]
pub struct MarkReadRequest {
    /// Chat ID
    pub chat_id: String,
    /// Optional list of specific message IDs to mark read
    /// If empty, marks all messages in chat as read
    pub message_ids: Option<Vec<String>>,
}

/// Response for mark read
#[derive(Debug, Serialize, ToSchema)]
pub struct MarkReadResponse {
    /// Success status
    pub success: bool,
    /// Chat ID where messages were read
    pub chat_id: String,
    /// Number of messages marked as read
    pub messages_read: u32,
}
