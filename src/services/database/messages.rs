//! Message Operations
//!
//! CRUD operations for messages (insert, get, update, delete).

use anyhow::Result;
use chrono::Utc;
use rusqlite::params;
use tracing::debug;

use crate::models::message::{is_self, MediaType, Message, MessageStatus, NewMessage, SELF_JID};

use super::service::DatabaseService;

impl DatabaseService {
    /// Insert a new message (returns message ID)
    pub fn insert_message(&self, msg: &NewMessage) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let msg_ts = msg.message_timestamp.map(|dt| dt.to_rfc3339());

        // For backward compatibility with old schema
        let phone = if is_self(&msg.sender) {
            &msg.recipient
        } else {
            &msg.sender
        };

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
                None::<String>,
                0,
                msg_ts,
                now,
                phone,
                direction,
                msg.sender_name,
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
            tracing::info!("Deleted {} messages older than {} days", deleted, days);
        }

        Ok(deleted as i64)
    }
}
