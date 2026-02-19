//! Session Models
//!
//! Data structures for WhatsApp session persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Session data structure for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub session_id: String,
    pub phone_number: Option<String>,
    pub authenticated_at: DateTime<Utc>,
    pub browser_cookies: Option<String>,   // JSON serialized cookies
    pub local_storage: Option<String>,     // JSON serialized local storage
    pub session_storage: Option<String>,   // JSON serialized session storage
}
