//! SQLite Database Service
//!
//! Handles message persistence, queue management, and conversation history.
//! Acts as a cache layer for WhatsApp DOM data.
//! Clears data on logout or when a different account logs in.
//!
//! ## Schema Design (based on WhatsApp Web DOM analysis)
//!
//! Messages use standard sender/recipient model:
//! - sender: who sent the message ("me" for outgoing, phone/JID for incoming)
//! - recipient: destination ("me" for incoming 1:1, phone/group JID for outgoing/groups)
//! - is_group: whether this is a group message
//!
//! Message types:
//! - Text: Plain text messages
//! - Image: With optional caption, stored locally
//! - Document: Filename, extension, size, stored locally
//! - Video: With optional caption, stored locally
//! - Voice: Duration, stored locally
//! - Quoted: Reply to another message
//!
//! Key attributes:
//! - Sender/Recipient: Standard messaging model (like email/whatsmeow)
//! - Status: Pending, Sent, Delivered, Read, Failed
//! - Timestamp: Message time
//! - Media: File path for attachments, media type, size, caption

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use tracing::{debug, info, warn};

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

// NOTE: QueueItem is removed - we use Message with sender='me' and status='pending'/'processing'
// This simplifies the schema and allows listing all messages in one query.
// Queue = SELECT * FROM messages WHERE sender = 'me' AND status IN ('pending', 'processing')

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

/// Batch size for bulk operations (inspired by whatsmeow)
pub const CONTACT_BATCH_SIZE: usize = 300;

/// Database service for message persistence
pub struct DatabaseService {
    conn: Mutex<Connection>,
    db_path: String,
}

impl DatabaseService {
    /// Create a new database service
    pub fn new(data_dir: &str) -> Result<Self> {
        let db_path = Path::new(data_dir).join("messages.db");
        let db_path_str = db_path.to_string_lossy().to_string();

        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        let service = Self {
            conn: Mutex::new(conn),
            db_path: db_path_str,
        };

        // Migrate first (add missing columns to existing tables)
        service.migrate_schema()?;
        // Then create indexes (after columns exist)
        service.init_schema()?;
        info!("Database initialized at: {}", service.db_path);

        Ok(service)
    }

    /// Create an in-memory database (for testing or fallback)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let service = Self {
            conn: Mutex::new(conn),
            db_path: ":memory:".to_string(),
        };

        service.init_schema()?;
        info!("In-memory database initialized");

        Ok(service)
    }

    /// Migrate schema for existing databases (add missing columns)
    fn migrate_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Check if messages table exists first
        let table_exists: bool = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='messages'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !table_exists {
            // Table doesn't exist yet, init_schema will create it with all columns
            debug!("Messages table doesn't exist, skipping migration");
            return Ok(());
        }

        // Get existing columns using PRAGMA
        let mut existing_columns: Vec<String> = Vec::new();
        {
            let mut stmt = conn.prepare("PRAGMA table_info(messages)")?;
            let column_iter = stmt.query_map([], |row| {
                let name: String = row.get(1)?;
                Ok(name)
            })?;
            for name in column_iter.flatten() {
                existing_columns.push(name);
            }
        }

        debug!("Existing columns in messages table: {:?}", existing_columns);

        // Add missing columns
        let migrations = [
            (
                "priority",
                "ALTER TABLE messages ADD COLUMN priority INTEGER DEFAULT 0",
            ),
            (
                "max_retries",
                "ALTER TABLE messages ADD COLUMN max_retries INTEGER DEFAULT 3",
            ),
            (
                "is_group",
                "ALTER TABLE messages ADD COLUMN is_group INTEGER DEFAULT 0",
            ),
            ("sender", "ALTER TABLE messages ADD COLUMN sender TEXT"),
            (
                "recipient",
                "ALTER TABLE messages ADD COLUMN recipient TEXT",
            ),
            (
                "sender_name",
                "ALTER TABLE messages ADD COLUMN sender_name TEXT",
            ),
        ];

        for (column, sql) in migrations {
            if !existing_columns.contains(&column.to_string()) {
                info!("Migrating database: adding {} column", column);
                if let Err(e) = conn.execute(sql, []) {
                    warn!("Failed to add column {}: {}", column, e);
                }
            }
        }

        // Migrate data from old phone/direction schema to sender/recipient model
        let has_old_schema = existing_columns.contains(&"phone".to_string())
            && existing_columns.contains(&"direction".to_string());
        let has_new_schema = existing_columns.contains(&"sender".to_string())
            && existing_columns.contains(&"recipient".to_string());

        if has_old_schema && has_new_schema {
            // Count rows that need migration
            let needs_migration: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM messages WHERE (sender IS NULL OR sender = '') AND phone IS NOT NULL",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            if needs_migration > 0 {
                info!(
                    "Migrating {} messages from phone/direction to sender/recipient model",
                    needs_migration
                );

                // Migrate outgoing messages: sender='me', recipient=phone
                let outgoing_migrated = conn
                    .execute(
                        "UPDATE messages SET 
                        sender = 'me',
                        recipient = phone,
                        is_group = 0
                     WHERE (sender IS NULL OR sender = '') 
                       AND direction = 'outgoing' 
                       AND phone IS NOT NULL",
                        [],
                    )
                    .unwrap_or(0);

                // Migrate incoming messages: sender=phone, recipient='me', sender_name=contact_name
                let incoming_migrated = conn
                    .execute(
                        "UPDATE messages SET 
                        sender = phone,
                        recipient = 'me',
                        sender_name = contact_name,
                        is_group = 0
                     WHERE (sender IS NULL OR sender = '') 
                       AND direction = 'incoming' 
                       AND phone IS NOT NULL",
                        [],
                    )
                    .unwrap_or(0);

                info!(
                    "Data migration complete: {} outgoing, {} incoming messages converted",
                    outgoing_migrated, incoming_migrated
                );
            } else {
                debug!("No messages need migration (already migrated or empty)");
            }
        }

        Ok(())
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Messages table - unified for all messages (sent, received, queued)
        // Uses sender/recipient model (standard messaging terminology)
        // Outgoing queue = messages WHERE sender='me' AND status IN ('pending', 'processing')
        // phone, direction, contact_name are for backward compatibility with old schemas
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                sender TEXT NOT NULL,
                recipient TEXT NOT NULL,
                sender_name TEXT,
                text TEXT,
                is_group INTEGER DEFAULT 0,
                status TEXT NOT NULL,
                priority INTEGER DEFAULT 0,
                media_type TEXT NOT NULL DEFAULT 'none',
                media_path TEXT,
                media_filename TEXT,
                media_extension TEXT,
                media_size INTEGER,
                media_duration INTEGER,
                quoted_message_id TEXT,
                error TEXT,
                retry_count INTEGER DEFAULT 0,
                max_retries INTEGER DEFAULT 3,
                message_timestamp TEXT,
                created_at TEXT NOT NULL,
                processed_at TEXT,
                phone TEXT,
                direction TEXT,
                contact_name TEXT,
                FOREIGN KEY (quoted_message_id) REFERENCES messages(id)
            )",
            [],
        )?;

        // Create index for queue queries (outgoing pending messages)
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_queue 
             ON messages(sender, status, priority DESC, created_at ASC)
             WHERE sender = 'me' AND status IN ('pending', 'processing')",
            [],
        )
        .ok(); // Ignore if partial index not supported

        // Create index for chat history queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_chat 
             ON messages(recipient, created_at DESC)",
            [],
        )
        .ok();

        // Conversations/Chats cache table (from WhatsApp DOM scraping)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                phone TEXT,
                name TEXT NOT NULL,
                last_message TEXT,
                last_message_time TEXT,
                unread_count INTEGER DEFAULT 0,
                is_group INTEGER DEFAULT 0,
                is_muted INTEGER DEFAULT 0,
                is_pinned INTEGER DEFAULT 0,
                is_archived INTEGER DEFAULT 0,
                avatar_url TEXT,
                cached_at TEXT NOT NULL
            )",
            [],
        )?;

        // Chat settings table (inspired by whatsmeow - stores user preferences)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat_settings (
                chat_id TEXT PRIMARY KEY,
                muted_until TEXT,
                pinned INTEGER DEFAULT 0,
                archived INTEGER DEFAULT 0,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Contacts table (for caching contact names and metadata)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS contacts (
                phone TEXT PRIMARY KEY,
                name TEXT,
                is_business INTEGER DEFAULT 0,
                last_seen TEXT,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Session info table (to track logged-in account)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS session (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Indexes for faster queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_status ON messages(status)",
            [],
        )?;
        // Create indexes for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender)",
            [],
        )
        .ok();
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_recipient ON messages(recipient)",
            [],
        )
        .ok();
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_status ON messages(status)",
            [],
        )
        .ok();
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_media_type ON messages(media_type)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_priority ON messages(priority DESC, created_at ASC)",
            [],
        )?;
        // Composite index for chat lookup (both directions of a conversation)
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_chat_lookup ON messages(sender, recipient, created_at DESC)",
            [],
        ).ok();
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_conversations_cached ON conversations(cached_at)",
            [],
        )?;

        debug!("Database schema initialized");
        Ok(())
    }

    /// Insert a new message (returns message ID)
    pub fn insert_message(&self, msg: &NewMessage) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let msg_ts = msg.message_timestamp.map(|dt| dt.to_rfc3339());

        // For backward compatibility with old schema that has phone NOT NULL
        // phone = recipient for outgoing, sender for incoming
        let phone = if is_self(&msg.sender) {
            &msg.recipient
        } else {
            &msg.sender
        };

        // direction for backward compatibility
        let direction = if is_self(&msg.sender) {
            "outgoing"
        } else {
            "incoming"
        };

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (
                id, sender, recipient, sender_name, text, is_group, status,
                media_type, media_path, media_filename, media_extension, 
                media_size, media_duration, quoted_message_id,
                error, retry_count, message_timestamp, created_at,
                phone, direction, contact_name
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
            params![
                id,
                msg.sender,
                msg.recipient,
                msg.sender_name,
                msg.text,
                msg.is_group as i32,
                msg.status.to_string(),
                msg.media_type.to_string(),
                msg.media_path,
                msg.media_filename,
                msg.media_extension,
                msg.media_size,
                msg.media_duration,
                msg.quoted_message_id,
                None::<String>, // error
                0,              // retry_count
                msg_ts,
                now,
                phone,         // backward compat
                direction,     // backward compat
                msg.sender_name, // contact_name = sender_name
            ],
        )?;

        let direction = if is_self(&msg.sender) {
            "outgoing"
        } else {
            "incoming"
        };
        debug!(
            "Inserted message: {} ({}, {})",
            id, direction, msg.media_type
        );
        Ok(id)
    }

    /// Insert an outgoing text message (convenience method)
    pub fn insert_outgoing_message(
        &self,
        recipient: &str,
        text: &str,
        status: MessageStatus,
    ) -> Result<String> {
        self.insert_message(&NewMessage {
            sender: SELF_JID.to_string(),
            recipient: recipient.to_string(),
            sender_name: None,
            text: Some(text.to_string()),
            is_group: false,
            status,
            media_type: MediaType::None,
            media_path: None,
            media_filename: None,
            media_extension: None,
            media_size: None,
            media_duration: None,
            quoted_message_id: None,
            message_timestamp: None,
        })
    }

    /// Insert an incoming text message (convenience method)
    pub fn insert_incoming_message(
        &self,
        sender: &str,
        sender_name: Option<&str>,
        text: &str,
    ) -> Result<String> {
        self.insert_message(&NewMessage {
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
        })
    }

    /// Insert an outgoing media message (convenience method)
    pub fn insert_outgoing_media(
        &self,
        recipient: &str,
        media_type: MediaType,
        media_path: &str,
        filename: Option<&str>,
        caption: Option<&str>,
        status: MessageStatus,
    ) -> Result<String> {
        // Extract extension from filename
        let extension = filename
            .and_then(|f| f.rsplit('.').next())
            .map(|e| e.to_uppercase());

        self.insert_message(&NewMessage {
            sender: SELF_JID.to_string(),
            recipient: recipient.to_string(),
            sender_name: None,
            text: caption.map(|s| s.to_string()),
            is_group: false,
            status,
            media_type,
            media_path: Some(media_path.to_string()),
            media_filename: filename.map(|s| s.to_string()),
            media_extension: extension,
            media_size: None,
            media_duration: None,
            quoted_message_id: None,
            message_timestamp: None,
        })
    }

    /// Update message status
    pub fn update_status(
        &self,
        id: &str,
        status: MessageStatus,
        error: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE messages SET status = ?1, error = ?2, processed_at = ?3 WHERE id = ?4",
            params![status.to_string(), error, now, id],
        )?;

        debug!("Updated message {} status to {}", id, status);
        Ok(())
    }

    /// Increment retry count
    pub fn increment_retry(&self, id: &str) -> Result<i32> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE messages SET retry_count = retry_count + 1 WHERE id = ?1",
            params![id],
        )?;

        let count: i32 = conn.query_row(
            "SELECT retry_count FROM messages WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        Ok(count)
    }

    /// Get next pending message (for queue processing)
    pub fn get_next_pending(&self) -> Result<Option<Message>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, sender, recipient, sender_name, text, is_group, status,
                    media_type, media_path, media_filename, media_extension,
                    media_size, media_duration, quoted_message_id,
                    error, retry_count, COALESCE(max_retries, 3), COALESCE(priority, 0),
                    message_timestamp, created_at, processed_at
             FROM messages
             WHERE status = 'pending' AND sender = 'me'
             ORDER BY created_at ASC
             LIMIT 1",
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_message(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get message by ID
    pub fn get_message(&self, id: &str) -> Result<Option<Message>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, sender, recipient, sender_name, text, is_group, status,
                    media_type, media_path, media_filename, media_extension,
                    media_size, media_duration, quoted_message_id,
                    error, retry_count, COALESCE(max_retries, 3), COALESCE(priority, 0),
                    message_timestamp, created_at, processed_at
             FROM messages WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_message(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get messages for a chat (by recipient/chat JID)
    /// This returns all messages sent to or received from a contact/group
    pub fn get_messages_for_chat(
        &self,
        chat_jid: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Message>> {
        let conn = self.conn.lock().unwrap();
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let mut stmt = conn.prepare(
            "SELECT id, sender, recipient, sender_name, text, is_group, status,
                    media_type, media_path, media_filename, media_extension,
                    media_size, media_duration, quoted_message_id,
                    error, retry_count, COALESCE(max_retries, 3), COALESCE(priority, 0),
                    message_timestamp, created_at, processed_at
             FROM messages
             WHERE recipient = ?1 OR (sender = ?1 AND recipient = 'me')
             ORDER BY created_at DESC
             LIMIT ?2 OFFSET ?3",
        )?;

        let mut rows = stmt.query(params![chat_jid, limit, offset])?;
        let mut messages = Vec::new();

        while let Some(row) = rows.next()? {
            messages.push(Self::row_to_message(row)?);
        }

        Ok(messages)
    }

    /// Alias for backward compatibility
    pub fn get_messages_by_phone(
        &self,
        phone: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Message>> {
        self.get_messages_for_chat(phone, limit, offset)
    }

    /// Get all messages (with pagination)
    pub fn get_all_messages(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Message>> {
        let conn = self.conn.lock().unwrap();
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let mut stmt = conn.prepare(
            "SELECT id, sender, recipient, sender_name, text, is_group, status,
                    media_type, media_path, media_filename, media_extension,
                    media_size, media_duration, quoted_message_id,
                    error, retry_count, COALESCE(max_retries, 3), COALESCE(priority, 0),
                    message_timestamp, created_at, processed_at
             FROM messages
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let mut rows = stmt.query(params![limit, offset])?;
        let mut messages = Vec::new();

        while let Some(row) = rows.next()? {
            messages.push(Self::row_to_message(row)?);
        }

        Ok(messages)
    }

    /// Get pending message count
    pub fn get_pending_count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE status = 'pending'",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Set session value
    pub fn set_session(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO session (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    /// Get session value
    pub fn get_session(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let result: rusqlite::Result<String> = conn.query_row(
            "SELECT value FROM session WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Check and handle account change (clear DB if different account)
    pub fn handle_login(&self, phone_number: &str) -> Result<bool> {
        let previous = self.get_session("logged_in_phone")?;

        if let Some(prev_phone) = previous {
            if prev_phone != phone_number {
                warn!(
                    "Different account detected (was: {}, now: {}). Clearing database.",
                    prev_phone, phone_number
                );
                self.clear_all()?;
            }
        }

        self.set_session("logged_in_phone", phone_number)?;
        info!("Login recorded for: {}", phone_number);
        Ok(true)
    }

    /// Clear all data (on logout)
    pub fn clear_all(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM messages", [])?;
        conn.execute("DELETE FROM contacts", [])?;
        conn.execute("DELETE FROM session", [])?;
        info!("Database cleared (logout/account change)");
        Ok(())
    }

    /// Delete old messages (retention policy)
    pub fn delete_older_than_days(&self, days: i64) -> Result<i64> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let cutoff_str = cutoff.to_rfc3339();

        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM messages WHERE created_at < ?1",
            params![cutoff_str],
        )?;

        if deleted > 0 {
            info!("Deleted {} messages older than {} days", deleted, days);
        }

        Ok(deleted as i64)
    }

    /// Save or update a contact
    pub fn upsert_contact(&self, phone: &str, name: Option<&str>, is_business: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO contacts (phone, name, is_business, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![phone, name, is_business as i32, now],
        )?;
        Ok(())
    }

    /// Get contact by phone
    pub fn get_contact(&self, phone: &str) -> Result<Option<Contact>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT phone, name, is_business, last_seen, updated_at
             FROM contacts WHERE phone = ?1",
            params![phone],
            |row| {
                let updated_str: String = row.get(4)?;
                let last_seen_str: Option<String> = row.get(3)?;
                Ok(Contact {
                    phone: row.get(0)?,
                    name: row.get(1)?,
                    is_business: row.get::<_, i32>(2)? != 0,
                    last_seen: last_seen_str
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    updated_at: DateTime::parse_from_rfc3339(&updated_str)
                        .unwrap()
                        .with_timezone(&Utc),
                })
            },
        );

        match result {
            Ok(contact) => Ok(Some(contact)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all unique conversations (distinct chat JIDs with latest message)
    /// Returns: Vec<(chat_jid, contact_name, Message)>
    pub fn get_conversations(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<(String, Option<String>, Message)>> {
        let conn = self.conn.lock().unwrap();
        let limit = limit.unwrap_or(50);

        // Get latest message for each chat (recipient for outgoing, sender for incoming)
        // We use COALESCE to get the "other party" regardless of direction
        let mut stmt = conn.prepare(
            "SELECT id, sender, recipient, sender_name, text, is_group, status,
                    media_type, media_path, media_filename, media_extension,
                    media_size, media_duration, quoted_message_id,
                    error, retry_count, COALESCE(max_retries, 3), COALESCE(priority, 0),
                    message_timestamp, created_at, processed_at,
                    CASE WHEN sender = 'me' THEN recipient ELSE sender END as chat_jid
             FROM messages m
             WHERE m.id IN (
                 SELECT id FROM (
                     SELECT id, 
                            CASE WHEN sender = 'me' THEN recipient ELSE sender END as chat,
                            ROW_NUMBER() OVER (PARTITION BY 
                                CASE WHEN sender = 'me' THEN recipient ELSE sender END 
                                ORDER BY created_at DESC) as rn
                     FROM messages
                 ) WHERE rn = 1
             )
             ORDER BY m.created_at DESC
             LIMIT ?1",
        )?;

        let mut rows = stmt.query(params![limit])?;
        let mut conversations = Vec::new();

        while let Some(row) = rows.next()? {
            let msg = Self::row_to_message(row)?;
            let chat_jid: String = row.get(21)?;
            // Use sender_name as contact name for now
            let contact_name = msg.sender_name.clone();
            conversations.push((chat_jid, contact_name, msg));
        }

        Ok(conversations)
    }

    /// Helper: Convert row to Message (unified format)
    /// Expected columns: id, sender, recipient, sender_name, text, is_group, status,
    ///                   media_type, media_path, media_filename, media_extension,
    ///                   media_size, media_duration, quoted_message_id,
    ///                   error, retry_count, max_retries, priority,
    ///                   message_timestamp, created_at, processed_at
    fn row_to_message(row: &rusqlite::Row) -> Result<Message> {
        let status_str: String = row.get(6)?;
        let media_type_str: String = row.get(7)?;
        let created_str: String = row.get(19)?;
        let processed_str: Option<String> = row.get(20)?;
        let msg_ts_str: Option<String> = row.get(18)?;

        Ok(Message {
            id: row.get(0)?,
            sender: row.get(1)?,
            recipient: row.get(2)?,
            sender_name: row.get(3)?,
            text: row.get(4)?,
            is_group: row.get::<_, i32>(5)? != 0,
            status: MessageStatus::try_from(status_str.as_str())?,
            media_type: MediaType::try_from(media_type_str.as_str())?,
            media_path: row.get(8)?,
            media_filename: row.get(9)?,
            media_extension: row.get(10)?,
            media_size: row.get(11)?,
            media_duration: row.get(12)?,
            quoted_message_id: row.get(13)?,
            error: row.get(14)?,
            retry_count: row.get(15)?,
            max_retries: row.get(16)?,
            priority: row.get(17)?,
            message_timestamp: msg_ts_str
                .map(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)))
                .transpose()?,
            created_at: DateTime::parse_from_rfc3339(&created_str)?.with_timezone(&Utc),
            processed_at: processed_str
                .map(|s| DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&Utc)))
                .transpose()?,
        })
    }

    // ========================================================================
    // Queue Management Methods
    // ========================================================================

    /// Queue a message for sending (returns transaction ID)
    /// Creates an outgoing message: sender="me", recipient=phone
    pub fn queue_message(
        &self,
        recipient: &str,
        text: Option<&str>,
        media_type: MediaType,
        media_path: Option<&str>,
        priority: Option<i32>,
    ) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let priority = priority.unwrap_or(0);

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (
                id, sender, recipient, is_group, status, priority,
                media_type, media_path, text, retry_count, max_retries, created_at
            ) VALUES (?1, 'me', ?2, 0, 'pending', ?3, ?4, ?5, ?6, 0, 3, ?7)",
            params![
                id,
                recipient,
                priority,
                media_type.to_string(),
                media_path,
                text,
                now
            ],
        )?;

        info!(
            "Queued message {} to {} (priority: {})",
            id, recipient, priority
        );
        Ok(id)
    }

    /// Get next pending message from queue (highest priority first, then oldest)
    /// Returns a Message (unified model - queue = outgoing pending messages)
    pub fn dequeue_next(&self) -> Result<Option<Message>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, sender, recipient, sender_name, text, is_group, status, media_type, 
                    media_path, media_filename, media_extension, media_size, media_duration,
                    quoted_message_id, error, retry_count, COALESCE(max_retries, 3), 
                    COALESCE(priority, 0), message_timestamp, created_at, processed_at
             FROM messages
             WHERE status = 'pending' AND sender = 'me'
               AND retry_count < COALESCE(max_retries, 3)
             ORDER BY priority DESC, created_at ASC
             LIMIT 1",
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_message(row)?))
        } else {
            Ok(None)
        }
    }

    /// Mark message as processing (lock it)
    pub fn mark_processing(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE messages SET status = 'processing' WHERE id = ?1 AND status = 'pending'",
            params![id],
        )?;
        debug!("Message {} marked as processing", id);
        Ok(())
    }

    /// Mark message as sent
    pub fn mark_sent(&self, id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE messages SET status = 'sent', processed_at = ?1, error = NULL WHERE id = ?2",
            params![now, id],
        )?;
        info!("Message {} sent successfully", id);
        Ok(())
    }

    /// Mark message as failed (with error and retry logic)
    pub fn mark_failed(&self, id: &str, error: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        // Check if we should retry
        let (retry_count, max_retries): (i32, i32) = conn.query_row(
            "SELECT retry_count, COALESCE(max_retries, 3) FROM messages WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        if retry_count + 1 < max_retries {
            // Retry: increment count and set back to pending
            conn.execute(
                "UPDATE messages SET status = 'pending', retry_count = retry_count + 1, error = ?1 
                 WHERE id = ?2",
                params![error, id],
            )?;
            warn!(
                "Message {} failed, will retry ({}/{}): {}",
                id,
                retry_count + 1,
                max_retries,
                error
            );
            Ok(true) // Will retry
        } else {
            // Max retries reached
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE messages SET status = 'failed', processed_at = ?1, error = ?2 WHERE id = ?3",
                params![now, error, id],
            )?;
            warn!(
                "Message {} permanently failed after {} retries: {}",
                id, max_retries, error
            );
            Ok(false) // No more retries
        }
    }

    /// Get queue status summary
    pub fn get_queue_status(&self) -> Result<QueueStatus> {
        let conn = self.conn.lock().unwrap();

        let pending: i64 = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE status = 'pending' AND sender = 'me'",
            [],
            |row| row.get(0),
        )?;

        let processing: i64 = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE status = 'processing' AND sender = 'me'",
            [],
            |row| row.get(0),
        )?;

        let failed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE status = 'failed' AND sender = 'me'",
            [],
            |row| row.get(0),
        )?;

        let today = Utc::now().format("%Y-%m-%d").to_string();
        let sent_today: i64 = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE status = 'sent' AND sender = 'me' 
             AND processed_at LIKE ?1",
            params![format!("{}%", today)],
            |row| row.get(0),
        )?;

        Ok(QueueStatus {
            pending_count: pending,
            processing_count: processing,
            failed_count: failed,
            total_sent_today: sent_today,
        })
    }

    /// Get all pending/queued messages for a recipient
    /// Returns Messages (unified model - queue = outgoing pending messages)
    pub fn get_queue_for_recipient(&self, recipient: &str) -> Result<Vec<Message>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, sender, recipient, sender_name, text, is_group, status, media_type, 
                    media_path, media_filename, media_extension, media_size, media_duration,
                    quoted_message_id, error, retry_count, COALESCE(max_retries, 3), 
                    COALESCE(priority, 0), message_timestamp, created_at, processed_at
             FROM messages
             WHERE recipient = ?1 AND sender = 'me' 
               AND status IN ('pending', 'processing')
             ORDER BY priority DESC, created_at ASC",
        )?;

        let mut rows = stmt.query(params![recipient])?;
        let mut items = Vec::new();

        while let Some(row) = rows.next()? {
            items.push(Self::row_to_message(row)?);
        }

        Ok(items)
    }

    /// Alias for backward compatibility
    pub fn get_queue_for_phone(&self, phone: &str) -> Result<Vec<Message>> {
        self.get_queue_for_recipient(phone)
    }

    /// Reset any stuck "processing" messages back to "pending" (recovery on restart)
    pub fn reset_stuck_processing(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute(
            "UPDATE messages SET status = 'pending' WHERE status = 'processing'",
            [],
        )?;
        if count > 0 {
            info!("Reset {} stuck processing messages to pending", count);
        }
        Ok(count as i64)
    }

    // ========================================================================
    // Conversation Cache Methods
    // ========================================================================

    /// Cache a conversation from DOM scraping
    pub fn cache_conversation(&self, conv: &Conversation) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO conversations 
             (id, phone, name, last_message, last_message_time, unread_count, 
              is_group, is_muted, is_pinned, is_archived, avatar_url, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                conv.id,
                conv.phone,
                conv.name,
                conv.last_message,
                conv.last_message_time,
                conv.unread_count,
                conv.is_group as i32,
                conv.is_muted as i32,
                conv.is_pinned as i32,
                conv.is_archived as i32,
                conv.avatar_url,
                now
            ],
        )?;
        Ok(())
    }

    /// Bulk cache conversations (from DOM scraping)
    pub fn cache_conversations(&self, convs: &[Conversation]) -> Result<()> {
        for conv in convs {
            self.cache_conversation(conv)?;
        }
        debug!("Cached {} conversations", convs.len());
        Ok(())
    }

    /// Get cached conversations
    pub fn get_cached_conversations(&self, limit: Option<i64>) -> Result<Vec<Conversation>> {
        let conn = self.conn.lock().unwrap();
        let limit = limit.unwrap_or(100);

        let mut stmt = conn.prepare(
            "SELECT id, phone, name, last_message, last_message_time, unread_count,
                    is_group, is_muted, is_pinned, is_archived, avatar_url, cached_at
             FROM conversations
             ORDER BY is_pinned DESC, cached_at DESC
             LIMIT ?1",
        )?;

        let mut rows = stmt.query(params![limit])?;
        let mut conversations = Vec::new();

        while let Some(row) = rows.next()? {
            let cached_str: String = row.get(11)?;
            conversations.push(Conversation {
                id: row.get(0)?,
                phone: row.get(1)?,
                name: row.get(2)?,
                last_message: row.get(3)?,
                last_message_time: row.get(4)?,
                unread_count: row.get(5)?,
                is_group: row.get::<_, i32>(6)? != 0,
                is_muted: row.get::<_, i32>(7)? != 0,
                is_pinned: row.get::<_, i32>(8)? != 0,
                is_archived: row.get::<_, i32>(9)? != 0,
                avatar_url: row.get(10)?,
                cached_at: DateTime::parse_from_rfc3339(&cached_str)?.with_timezone(&Utc),
            });
        }

        Ok(conversations)
    }

    /// Check if conversation cache is stale (older than given seconds)
    pub fn is_conversation_cache_stale(&self, max_age_secs: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let result: rusqlite::Result<String> = conn.query_row(
            "SELECT cached_at FROM conversations ORDER BY cached_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        );

        match result {
            Ok(cached_str) => {
                let cached = DateTime::parse_from_rfc3339(&cached_str)?.with_timezone(&Utc);
                let age = Utc::now() - cached;
                Ok(age.num_seconds() > max_age_secs)
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(true), // No cache = stale
            Err(e) => Err(e.into()),
        }
    }

    /// Clear conversation cache
    pub fn clear_conversation_cache(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM conversations", [])?;
        debug!("Conversation cache cleared");
        Ok(())
    }

    // ========================================================================
    // Chat Settings Methods (inspired by whatsmeow)
    // ========================================================================

    /// Set chat as pinned/unpinned
    pub fn set_chat_pinned(&self, chat_id: &str, pinned: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO chat_settings (chat_id, pinned, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(chat_id) DO UPDATE SET pinned = ?2, updated_at = ?3",
            params![chat_id, pinned as i32, now],
        )?;
        debug!("Chat {} pinned={}", chat_id, pinned);
        Ok(())
    }

    /// Set chat as archived/unarchived
    pub fn set_chat_archived(&self, chat_id: &str, archived: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO chat_settings (chat_id, archived, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(chat_id) DO UPDATE SET archived = ?2, updated_at = ?3",
            params![chat_id, archived as i32, now],
        )?;
        debug!("Chat {} archived={}", chat_id, archived);
        Ok(())
    }

    /// Set chat muted until a specific time (None = unmute)
    pub fn set_chat_muted(&self, chat_id: &str, muted_until: Option<DateTime<Utc>>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let muted_str = muted_until.map(|t| t.to_rfc3339());
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO chat_settings (chat_id, muted_until, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(chat_id) DO UPDATE SET muted_until = ?2, updated_at = ?3",
            params![chat_id, muted_str, now],
        )?;
        debug!("Chat {} muted_until={:?}", chat_id, muted_until);
        Ok(())
    }

    /// Get chat settings
    pub fn get_chat_settings(&self, chat_id: &str) -> Result<ChatSettings> {
        let conn = self.conn.lock().unwrap();

        let result: rusqlite::Result<(Option<String>, i32, i32)> = conn.query_row(
            "SELECT muted_until, pinned, archived FROM chat_settings WHERE chat_id = ?1",
            params![chat_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );

        match result {
            Ok((muted_str, pinned, archived)) => Ok(ChatSettings {
                chat_id: chat_id.to_string(),
                muted_until: muted_str
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                pinned: pinned != 0,
                archived: archived != 0,
            }),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(ChatSettings {
                chat_id: chat_id.to_string(),
                ..Default::default()
            }),
            Err(e) => Err(e.into()),
        }
    }

    // ========================================================================
    // Batch Contact Operations (inspired by whatsmeow)
    // ========================================================================

    /// Batch insert/update contacts (more efficient for syncing)
    pub fn put_all_contacts(&self, contacts: &[Contact]) -> Result<usize> {
        if contacts.is_empty() {
            return Ok(0);
        }

        let conn = self.conn.lock().unwrap();
        let mut count = 0;

        // Process in batches for efficiency
        for chunk in contacts.chunks(CONTACT_BATCH_SIZE) {
            for contact in chunk {
                let now = Utc::now().to_rfc3339();
                let last_seen = contact.last_seen.map(|t| t.to_rfc3339());

                conn.execute(
                    "INSERT INTO contacts (phone, name, is_business, last_seen, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(phone) DO UPDATE SET 
                        name = COALESCE(excluded.name, contacts.name),
                        is_business = excluded.is_business,
                        last_seen = COALESCE(excluded.last_seen, contacts.last_seen),
                        updated_at = excluded.updated_at",
                    params![
                        contact.phone,
                        contact.name,
                        contact.is_business as i32,
                        last_seen,
                        now
                    ],
                )?;
                count += 1;
            }
        }

        info!("Batch inserted/updated {} contacts", count);
        Ok(count)
    }

    /// Get all contacts
    pub fn get_all_contacts(&self) -> Result<Vec<Contact>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT phone, name, is_business, last_seen, updated_at FROM contacts ORDER BY name",
        )?;

        let mut contacts = Vec::new();
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            let last_seen_str: Option<String> = row.get(3)?;
            let updated_str: String = row.get(4)?;

            contacts.push(Contact {
                phone: row.get(0)?,
                name: row.get(1)?,
                is_business: row.get::<_, i32>(2)? != 0,
                last_seen: last_seen_str
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                updated_at: DateTime::parse_from_rfc3339(&updated_str)?.with_timezone(&Utc),
            });
        }

        Ok(contacts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_insert_and_get_text_message() {
        let dir = tempdir().unwrap();
        let db = DatabaseService::new(dir.path().to_str().unwrap()).unwrap();

        let id = db
            .insert_outgoing_message("1234567890", "Hello", MessageStatus::Pending)
            .unwrap();

        let msg = db.get_message(&id).unwrap().unwrap();
        assert_eq!(msg.recipient, "1234567890");
        assert_eq!(msg.sender, SELF_JID);
        assert_eq!(msg.text, Some("Hello".to_string()));
        assert_eq!(msg.status, MessageStatus::Pending);
        assert_eq!(msg.media_type, MediaType::None);
        assert!(msg.is_outgoing());
    }

    #[test]
    fn test_insert_media_message() {
        let dir = tempdir().unwrap();
        let db = DatabaseService::new(dir.path().to_str().unwrap()).unwrap();

        let id = db
            .insert_outgoing_media(
                "1234567890",
                MediaType::Document,
                "/path/to/file.pdf",
                Some("document.pdf"),
                Some("Check this file"),
                MessageStatus::Pending,
            )
            .unwrap();

        let msg = db.get_message(&id).unwrap().unwrap();
        assert_eq!(msg.recipient, "1234567890");
        assert_eq!(msg.sender, SELF_JID);
        assert_eq!(msg.text, Some("Check this file".to_string()));
        assert_eq!(msg.media_type, MediaType::Document);
        assert_eq!(msg.media_path, Some("/path/to/file.pdf".to_string()));
        assert_eq!(msg.media_filename, Some("document.pdf".to_string()));
        assert_eq!(msg.media_extension, Some("PDF".to_string()));
    }

    #[test]
    fn test_incoming_message() {
        let dir = tempdir().unwrap();
        let db = DatabaseService::new(dir.path().to_str().unwrap()).unwrap();

        let id = db
            .insert_incoming_message("9876543210", Some("John Doe"), "Hi there!")
            .unwrap();

        let msg = db.get_message(&id).unwrap().unwrap();
        assert_eq!(msg.sender, "9876543210");
        assert_eq!(msg.recipient, SELF_JID);
        assert_eq!(msg.sender_name, Some("John Doe".to_string()));
        assert!(msg.is_incoming());
    }

    #[test]
    fn test_clear_on_account_change() {
        let dir = tempdir().unwrap();
        let db = DatabaseService::new(dir.path().to_str().unwrap()).unwrap();

        // Login with first account
        db.handle_login("111111").unwrap();
        db.insert_outgoing_message("111111", "Test", MessageStatus::Sent)
            .unwrap();

        assert_eq!(db.get_all_messages(None, None).unwrap().len(), 1);

        // Login with different account - should clear
        db.handle_login("222222").unwrap();
        assert_eq!(db.get_all_messages(None, None).unwrap().len(), 0);
    }

    #[test]
    fn test_new_message_helpers() {
        let text_msg = NewMessage::outgoing_text("1234567890", "Hello");
        assert_eq!(text_msg.sender, SELF_JID);
        assert_eq!(text_msg.recipient, "1234567890");
        assert_eq!(text_msg.status, MessageStatus::Pending);
        assert_eq!(text_msg.media_type, MediaType::None);

        let incoming_msg = NewMessage::incoming_text("9876543210", Some("John"), "Hi!");
        assert_eq!(incoming_msg.sender, "9876543210");
        assert_eq!(incoming_msg.recipient, SELF_JID);
        assert_eq!(incoming_msg.status, MessageStatus::Received);

        let media_msg = NewMessage::outgoing_media(
            "1234567890",
            MediaType::Image,
            "/path/to/image.jpg",
            Some("Nice picture"),
        );
        assert_eq!(media_msg.status, MessageStatus::Pending);
        assert_eq!(media_msg.media_type, MediaType::Image);
    }

    #[test]
    fn test_contact_operations() {
        let dir = tempdir().unwrap();
        let db = DatabaseService::new(dir.path().to_str().unwrap()).unwrap();

        // No contact initially
        assert!(db.get_contact("1234567890").unwrap().is_none());

        // Add contact
        db.upsert_contact("1234567890", Some("Test User"), false)
            .unwrap();

        let contact = db.get_contact("1234567890").unwrap().unwrap();
        assert_eq!(contact.name, Some("Test User".to_string()));
        assert!(!contact.is_business);

        // Update contact
        db.upsert_contact("1234567890", Some("Updated Name"), true)
            .unwrap();

        let contact = db.get_contact("1234567890").unwrap().unwrap();
        assert_eq!(contact.name, Some("Updated Name".to_string()));
        assert!(contact.is_business);
    }
}
