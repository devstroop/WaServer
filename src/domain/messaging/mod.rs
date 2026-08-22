//! Messaging Domain — Message, Media, Status
//!
//! Pure domain, no `axum`/`tokio`. Mirrors `models/message.rs` but lives in domain.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const SELF_JID: &str = "me";

pub fn is_self(jid: &str) -> bool {
    jid == SELF_JID || jid == "me" || jid.is_empty()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    Pending,
    Processing,
    Sent,
    Delivered,
    Read,
    Failed,
    Received,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    None,
    Image,
    Video,
    Document,
    Voice,
    Sticker,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub sender: String,
    pub recipient: String,
    pub sender_name: Option<String>,
    pub text: Option<String>,
    pub is_group: bool,
    pub status: MessageStatus,
    pub media_type: MediaType,
    pub media_path: Option<String>,
    pub media_filename: Option<String>,
    pub media_extension: Option<String>,
    pub media_size: Option<i64>,
    pub media_duration: Option<i32>,
    pub quoted_message_id: Option<String>,
    pub error: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub priority: i32,
    pub message_timestamp: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

impl Message {
    pub fn is_outgoing(&self) -> bool {
        is_self(&self.sender)
    }
    pub fn is_incoming(&self) -> bool {
        !self.is_outgoing()
    }
    pub fn chat_jid(&self) -> &str {
        if self.is_outgoing() || self.is_group {
            &self.recipient
        } else {
            &self.sender
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewMessage {
    pub sender: String,
    pub recipient: String,
    pub sender_name: Option<String>,
    pub text: Option<String>,
    pub is_group: bool,
    pub status: MessageStatus,
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
    pub fn outgoing_media(recipient: &str, media_type: MediaType, media_path: &str, caption: Option<&str>) -> Self {
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
    pub fn incoming(sender: &str, recipient: &str, text: Option<&str>, is_group: bool, timestamp: Option<DateTime<Utc>>) -> Self {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub phone: String,
    pub name: Option<String>,
    pub is_business: bool,
    pub last_seen: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub phone: Option<String>,
    pub name: String,
    pub last_message: Option<String>,
    pub last_message_time: Option<String>,
    pub unread_count: i32,
    pub is_group: bool,
    pub is_muted: bool,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub avatar_url: Option<String>,
    pub cached_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatSettings {
    pub chat_id: String,
    pub muted_until: Option<DateTime<Utc>>,
    pub pinned: bool,
    pub archived: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueueStatus {
    pub pending_count: i64,
    pub processing_count: i64,
    pub failed_count: i64,
    pub total_sent_today: i64,
}
