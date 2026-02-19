//! Chat Settings Operations
//!
//! Per-chat settings: pinned, archived, muted.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::params;
use tracing::debug;

use crate::models::message::ChatSettings;

use super::service::DatabaseService;

impl DatabaseService {
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
}
