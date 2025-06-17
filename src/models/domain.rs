use serde::{Deserialize, Serialize};

/// Domain models for the WhatsApp Engine library
/// These are pure business objects without HTTP-specific details

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub phone: String,
    pub is_business: bool,
    pub profile_picture_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: String,
    pub name: String,
    pub is_group: bool,
    pub last_message: Option<String>,
    pub unread_count: u32,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message_type: MessageType,
    pub status: MessageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Image,
    Document,
    Audio,
    Video,
    Sticker,
    Location,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageStatus {
    Sending,
    Sent,
    Delivered,
    Read,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub phone_number: Option<String>,
    pub session_id: Option<String>,
    pub authenticated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCode {
    pub data: String, // base64 encoded PNG
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub image_url: String,
    pub refresh_interval_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneAuthResult {
    pub success: bool,
    pub verification_code: Option<String>,
    pub message: String,
    pub next_retry_in_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
    pub retry_after_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatus {
    pub is_ready: bool,
    pub browser_connected: bool,
    pub whatsapp_loaded: bool,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    pub file_path: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub caption: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationStatus {
    pub is_authenticated: bool,
    pub phone_number: Option<String>,
    pub connection_state: String,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}
