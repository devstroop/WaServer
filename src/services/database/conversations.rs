//! Conversation Cache Operations
//!
//! Caching conversations from WhatsApp DOM scraping.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::params;
use tracing::debug;

use crate::models::message::{Conversation, Message};

use super::service::DatabaseService;

impl DatabaseService {
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
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(true),
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

    /// Get all unique conversations (distinct chat JIDs with latest message)
    pub fn get_conversations(
        &self,
        limit: Option<i64>,
    ) -> Result<Vec<(String, Option<String>, Message)>> {
        let conn = self.conn.lock().unwrap();
        let limit = limit.unwrap_or(50);

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
            let contact_name = msg.sender_name.clone();
            conversations.push((chat_jid, contact_name, msg));
        }

        Ok(conversations)
    }
}
