//! Domain models for the WAS (WhatsApp Server) library
//!
//! These are pure business objects for the library API.
//! Database models are in services/database.rs - use conversions when needed.

use serde::{Deserialize, Serialize};

// Re-export database types as the canonical source for Message-related types
// This avoids duplicate definitions and ensures consistency
pub use crate::services::database::{MediaType, Message, MessageStatus, NewMessage, SELF_JID};

/// Authentication status for the library API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub phone_number: Option<String>,
    pub session_id: Option<String>,
    pub authenticated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// QR code data for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCode {
    /// Base64 encoded PNG image data
    pub data: String,
    /// When this QR code expires
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// URL to the QR code image (same as data for base64)
    pub image_url: String,
    /// How often to refresh in seconds
    pub refresh_interval_seconds: u32,
}

/// Result of phone number authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneAuthResult {
    pub success: bool,
    pub verification_code: Option<String>,
    pub message: String,
    pub next_retry_in_seconds: Option<u32>,
}

/// Result of sending a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
    pub retry_after_seconds: Option<u32>,
}

/// Engine status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatus {
    pub is_ready: bool,
    pub browser_connected: bool,
    pub whatsapp_loaded: bool,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
}

/// File attachment for sending media
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    pub file_path: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub caption: Option<String>,
}
