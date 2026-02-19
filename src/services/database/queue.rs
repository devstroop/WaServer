//! Message Queue Operations
//!
//! Outgoing message queue management: enqueue, dequeue, status updates.

use anyhow::Result;
use chrono::Utc;
use rusqlite::params;
use tracing::{debug, info, warn};

use crate::models::message::{MediaType, Message, QueueStatus};

use super::service::DatabaseService;

impl DatabaseService {
    /// Queue a message for sending (returns transaction ID)
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

        let (retry_count, max_retries): (i32, i32) = conn.query_row(
            "SELECT retry_count, COALESCE(max_retries, 3) FROM messages WHERE id = ?1",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        if retry_count + 1 < max_retries {
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
            Ok(true)
        } else {
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE messages SET status = 'failed', processed_at = ?1, error = ?2 WHERE id = ?3",
                params![now, error, id],
            )?;
            warn!(
                "Message {} permanently failed after {} retries: {}",
                id, max_retries, error
            );
            Ok(false)
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
}
