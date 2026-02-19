//! Database Service Core
//!
//! Creates and manages the SQLite connection, schema initialization, and migrations.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;
use tracing::{debug, info, warn};

use crate::models::message::{MediaType, Message, MessageStatus};

/// Batch size for bulk operations (inspired by whatsmeow)
pub const CONTACT_BATCH_SIZE: usize = 300;

/// Database service for message persistence
pub struct DatabaseService {
    pub(super) conn: Mutex<Connection>,
    pub(super) db_path: String,
}

impl DatabaseService {
    /// Create a new database service
    pub fn new(data_dir: &str) -> Result<Self> {
        let db_path = Path::new(data_dir).join("database.db");
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

        // Messages table
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

        // Queue index
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_queue 
             ON messages(sender, status, priority DESC, created_at ASC)
             WHERE sender = 'me' AND status IN ('pending', 'processing')",
            [],
        )
        .ok();

        // Chat history index
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_chat 
             ON messages(recipient, created_at DESC)",
            [],
        )
        .ok();

        // Conversations cache table
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

        // Chat settings table
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

        // Contacts table
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

        // Session table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS session (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_status ON messages(status)",
            [],
        )?;
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

    /// Helper: Convert row to Message
    pub(super) fn row_to_message(row: &rusqlite::Row) -> Result<Message> {
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
}
