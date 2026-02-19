//! Message Models
//!
//! Core message types for WhatsApp communication.
//! Based on WhatsApp Web DOM analysis with standard sender/recipient model.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Special constant for "self" - the logged-in WhatsApp account
/// Used as sender for outgoing messages and recipient for incoming 1:1 messages
pub const SELF_JID: &str = "me";

/// Check if a JID represents the logged-in user
pub fn is_self(jid: &str) -> bool {
    jid == SELF_JID || jid == "me" || jid.is_empty()
}

/// Message status (matches WhatsApp delivery states)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    Pending,    // Queued, waiting to be processed (outgoing only)
    Processing, // Currently being sent (outgoing only)
    Sent,       // Successfully sent (single check)
    Delivered,  // Delivered to recipient (double check)
    Read,       // Read by recipient (blue double check)
    Failed,     // Failed to send
    Received,   // Incoming message received
}

impl std::fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageStatus::Pending => write!(f, "pending"),
            MessageStatus::Processing => write!(f, "processing"),
            MessageStatus::Sent => write!(f, "sent"),
            MessageStatus::Delivered => write!(f, "delivered"),
            MessageStatus::Read => write!(f, "read"),
            MessageStatus::Failed => write!(f, "failed"),
            MessageStatus::Received => write!(f, "received"),
        }
    }
}

impl TryFrom<&str> for MessageStatus {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self> {
        match s {
            "pending" => Ok(MessageStatus::Pending),
            "processing" => Ok(MessageStatus::Processing),
            "sent" => Ok(MessageStatus::Sent),
            "delivered" => Ok(MessageStatus::Delivered),
            "read" => Ok(MessageStatus::Read),
            "failed" => Ok(MessageStatus::Failed),
            "received" => Ok(MessageStatus::Received),
            _ => Err(anyhow::anyhow!("Invalid message status: {}", s)),
        }
    }
}

/// Message media type (from WhatsApp Web DOM)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    None,     // Text-only message
    Image,    // Photo/Picture
    Video,    // Video file
    Document, // PDF, TOML, etc.
    Voice,    // Voice message/audio
    Sticker,  // Sticker
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::None => write!(f, "none"),
            MediaType::Image => write!(f, "image"),
            MediaType::Video => write!(f, "video"),
            MediaType::Document => write!(f, "document"),
            MediaType::Voice => write!(f, "voice"),
            MediaType::Sticker => write!(f, "sticker"),
        }
    }
}

impl TryFrom<&str> for MediaType {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self> {
        match s {
            "none" => Ok(MediaType::None),
            "image" => Ok(MediaType::Image),
            "video" => Ok(MediaType::Video),
            "document" => Ok(MediaType::Document),
            "voice" => Ok(MediaType::Voice),
            "sticker" => Ok(MediaType::Sticker),
            _ => Err(anyhow::anyhow!("Invalid media type: {}", s)),
        }
    }
}

/// Message record - unified for both outgoing (queue) and incoming messages
/// Standard sender/recipient model (like email/whatsmeow):
/// - 1:1 outgoing: sender="me", recipient="contact_phone"
/// - 1:1 incoming: sender="contact_phone", recipient="me"
/// - Group outgoing: sender="me", recipient="group_jid"
/// - Group incoming: sender="member_phone", recipient="group_jid"
///
/// Outgoing queue = messages WHERE sender='me' AND status IN ('pending', 'processing')
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID (UUID) - also serves as transaction ID for queued messages
    pub id: String,
    /// Sender JID/phone ("me" for outgoing, contact phone for incoming)
    pub sender: String,
    /// Recipient JID/phone (contact for 1:1, group JID for groups)
    pub recipient: String,
    /// Sender display name (contact name or group member name)
    pub sender_name: Option<String>,
    /// Message text content (or caption for media)
    pub text: Option<String>,
    /// Whether this is a group message
    pub is_group: bool,
    /// Message status (pending, processing, sent, delivered, read, failed, received)
    pub status: MessageStatus,
    /// Media type (none, image, video, document, voice)
    pub media_type: MediaType,
    /// Local file path for media (stored in data dir)
    pub media_path: Option<String>,
    /// Original filename for documents
    pub media_filename: Option<String>,
    /// File extension/type (e.g., "TOML", "PDF")
    pub media_extension: Option<String>,
    /// File size in bytes
    pub media_size: Option<i64>,
    /// Duration in seconds (for voice/video)
    pub media_duration: Option<i32>,
    /// Quoted message ID (for replies)
    pub quoted_message_id: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Retry count for failed messages
    pub retry_count: i32,
    /// Max retries allowed (for outgoing queue)
    pub max_retries: i32,
    /// Priority for send queue (higher = first)
    pub priority: i32,
    /// WhatsApp timestamp from the message
    pub message_timestamp: Option<DateTime<Utc>>,
    /// When we created this record
    pub created_at: DateTime<Utc>,
    /// When the message was processed
    pub processed_at: Option<DateTime<Utc>>,
}

impl Message {
    /// Check if this is an outgoing message (sent by us)
    pub fn is_outgoing(&self) -> bool {
        is_self(&self.sender)
    }

    /// Check if this is an incoming message
    pub fn is_incoming(&self) -> bool {
        !self.is_outgoing()
    }

    /// Get the "other party" - the contact/group we're chatting with
    /// For outgoing: returns recipient
    /// For incoming 1:1: returns sender
    /// For incoming group: returns recipient (the group)
    pub fn chat_jid(&self) -> &str {
        if self.is_outgoing() || self.is_group {
            &self.recipient
        } else {
            &self.sender
        }
    }
}

/// New message input (for inserting)
#[derive(Debug, Clone)]
pub struct NewMessage {
    /// Sender JID ("me" for outgoing)
    pub sender: String,
    /// Recipient JID (contact phone or group JID)
    pub recipient: String,
    /// Sender display name
    pub sender_name: Option<String>,
    /// Message text
    pub text: Option<String>,
    /// Whether this is a group message
    pub is_group: bool,
    /// Message status
    pub status: MessageStatus,
    /// Media type
    pub media_type: MediaType,
    pub media_path: Option<String>,
    pub media_filename: Option<String>,
    pub media_extension: Option<String>,
    pub media_size: Option<i64>,
    pub media_duration: Option<i32>,
    pub quoted_message_id: Option<String>,
    pub message_timestamp: Option<DateTime<Utc>>,
}

impl NewMessage {
    /// Create an outgoing text message (sender=me, recipient=phone)
    pub fn outgoing_text(recipient: &str, text: &str) -> Self {
        Self {
            sender: SELF_JID.to_string(),
            recipient: recipient.to_string(),
            sender_name: None,
            text: Some(text.to_string()),
            is_group: false,
            status: MessageStatus::Pending,
            media_type: MediaType::None,
            media_path: None,
            media_filename: None,
            media_extension: None,
            media_size: None,
            media_duration: None,
            quoted_message_id: None,
            message_timestamp: None,
        }
    }

    /// Create an outgoing media message
    pub fn outgoing_media(
        recipient: &str,
        media_type: MediaType,
        media_path: &str,
        caption: Option<&str>,
    ) -> Self {
        Self {
            sender: SELF_JID.to_string(),
            recipient: recipient.to_string(),
            sender_name: None,
            text: caption.map(|s| s.to_string()),
            is_group: false,
            status: MessageStatus::Pending,
            media_type,
            media_path: Some(media_path.to_string()),
            media_filename: None,
            media_extension: None,
            media_size: None,
            media_duration: None,
            quoted_message_id: None,
            message_timestamp: None,
        }
    }

    /// Create an incoming message record
    pub fn incoming(
        sender: &str,
        recipient: &str,
        text: Option<&str>,
        is_group: bool,
        timestamp: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            sender_name: None,
            text: text.map(|s| s.to_string()),
            is_group,
            status: MessageStatus::Received,
            media_type: MediaType::None,
            media_path: None,
            media_filename: None,
            media_extension: None,
            media_size: None,
            media_duration: None,
            quoted_message_id: None,
            message_timestamp: timestamp,
        }
    }

    /// Create an incoming text message (sender=contact, recipient=me)
    pub fn incoming_text(sender: &str, sender_name: Option<&str>, text: &str) -> Self {
        Self {
            sender: sender.to_string(),
            recipient: SELF_JID.to_string(),
            sender_name: sender_name.map(|s| s.to_string()),
            text: Some(text.to_string()),
            is_group: false,
            status: MessageStatus::Received,
            media_type: MediaType::None,
            media_path: None,
            media_filename: None,
            media_extension: None,
            media_size: None,
            media_duration: None,
            quoted_message_id: None,
            message_timestamp: None,
        }
    }

    /// Create a group message (incoming from a member)
    pub fn group_incoming(
        group_jid: &str,
        sender: &str,
        sender_name: Option<&str>,
        text: &str,
    ) -> Self {
        Self {
            sender: sender.to_string(),
            recipient: group_jid.to_string(),
            sender_name: sender_name.map(|s| s.to_string()),
            text: Some(text.to_string()),
            is_group: true,
            status: MessageStatus::Received,
            media_type: MediaType::None,
            media_path: None,
            media_filename: None,
            media_extension: None,
            media_size: None,
            media_duration: None,
            quoted_message_id: None,
            message_timestamp: None,
        }
    }
}

/// Contact record (cached from WhatsApp)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub phone: String,
    pub name: Option<String>,
    pub is_business: bool,
    pub last_seen: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// Conversation/Chat record (cached from WhatsApp DOM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Chat ID (phone@c.us or group ID)
    pub id: String,
    /// Phone number (if individual chat)
    pub phone: Option<String>,
    /// Contact/group name
    pub name: String,
    /// Last message preview
    pub last_message: Option<String>,
    /// Last message timestamp (human readable from DOM)
    pub last_message_time: Option<String>,
    /// Unread message count
    pub unread_count: i32,
    /// Is this a group chat
    pub is_group: bool,
    /// Is chat muted
    pub is_muted: bool,
    /// Is chat pinned
    pub is_pinned: bool,
    /// Is chat archived
    pub is_archived: bool,
    /// Avatar URL
    pub avatar_url: Option<String>,
    /// When this was cached
    pub cached_at: DateTime<Utc>,
}

/// Chat settings (inspired by whatsmeow)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatSettings {
    pub chat_id: String,
    pub muted_until: Option<DateTime<Utc>>,
    pub pinned: bool,
    pub archived: bool,
}

/// Queue status summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueueStatus {
    pub pending_count: i64,
    pub processing_count: i64,
    pub failed_count: i64,
    pub total_sent_today: i64,
}

/// Message debug timings (inspired by whatsmeow)
/// Tracks time spent in different phases for debugging/optimization
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageDebugTimings {
    /// Time spent waiting in queue
    pub queue_ms: u64,
    /// Time spent navigating to chat
    pub navigate_ms: u64,
    /// Time spent typing/sending
    pub send_ms: u64,
    /// Time waiting for delivery confirmation
    pub confirm_ms: u64,
    /// Total time from queue to sent
    pub total_ms: u64,
}
