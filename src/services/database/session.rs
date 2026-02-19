//! Session Management
//!
//! Session key-value storage, login handling, and data clearing.

use anyhow::Result;
use rusqlite::params;
use tracing::{info, warn};

use super::service::DatabaseService;

impl DatabaseService {
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
}
