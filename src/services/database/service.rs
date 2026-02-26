//! Database Service
//!
//! Manages an embedded SQLite database (file-based).
//! Provides typed helpers for account CRUD operations.

use anyhow::{anyhow, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::info;

use super::schema;

/// Persistent account record stored in SQLite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRecord {
    pub id: String,
    pub phone_number: String,
    pub display_name: String,
    pub data_dir: String,
    pub auto_start: bool,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Wraps an embedded SQLite connection.
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Open (or create) a SQLite database at the given directory.
    pub fn open(data_dir: &Path) -> Result<Self> {
        let db_dir = data_dir.join("db");
        std::fs::create_dir_all(&db_dir)?;

        let db_path = db_dir.join("was.db");

        let conn = Connection::open(&db_path)
            .map_err(|e| anyhow!("Failed to open SQLite database: {}", e))?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        info!("SQLite database opened at {:?}", db_path);

        schema::apply(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    // ========================================================================
    // Account CRUD
    // ========================================================================

    /// Insert a new account. The record ID is the account UUID (string).
    pub fn create_account(
        &self,
        id: &str,
        phone_number: &str,
        display_name: &str,
        data_dir: &str,
        auto_start: bool,
    ) -> Result<AccountRecord> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        // Check uniqueness of phone_number explicitly for a clear error message
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM account WHERE phone_number = ?1 LIMIT 1",
                rusqlite::params![phone_number],
                |row| row.get(0),
            )
            .ok();

        if existing.is_some() {
            return Err(anyhow!("Phone number '{}' already exists", phone_number));
        }

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO account (id, phone_number, display_name, data_dir, auto_start, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'stopped', ?6, ?7)",
            rusqlite::params![id, phone_number, display_name, data_dir, auto_start as i32, &now, &now],
        )
        .map_err(|e| anyhow!("Failed to create account: {}", e))?;

        Ok(AccountRecord {
            id: id.to_string(),
            phone_number: phone_number.to_string(),
            display_name: display_name.to_string(),
            data_dir: data_dir.to_string(),
            auto_start,
            status: "stopped".to_string(),
            created_at: Some(now.clone()),
            updated_at: Some(now),
        })
    }

    /// Get an account by its UUID string id.
    pub fn get_account(&self, id: &str) -> Result<Option<AccountRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT id, phone_number, display_name, data_dir, auto_start, status, created_at, updated_at FROM account WHERE id = ?1")
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let record = stmt
            .query_row(rusqlite::params![id], |row| {
                Ok(AccountRecord {
                    id: row.get(0)?,
                    phone_number: row.get(1)?,
                    display_name: row.get(2)?,
                    data_dir: row.get(3)?,
                    auto_start: row.get::<_, i32>(4)? != 0,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .ok();

        Ok(record)
    }

    /// Find an account by phone number.
    pub fn get_account_by_phone(&self, phone_number: &str) -> Result<Option<AccountRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT id, phone_number, display_name, data_dir, auto_start, status, created_at, updated_at FROM account WHERE phone_number = ?1 LIMIT 1")
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let record = stmt
            .query_row(rusqlite::params![phone_number], |row| {
                Ok(AccountRecord {
                    id: row.get(0)?,
                    phone_number: row.get(1)?,
                    display_name: row.get(2)?,
                    data_dir: row.get(3)?,
                    auto_start: row.get::<_, i32>(4)? != 0,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .ok();

        Ok(record)
    }

    /// List all accounts.
    pub fn list_accounts(&self) -> Result<Vec<AccountRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT id, phone_number, display_name, data_dir, auto_start, status, created_at, updated_at FROM account")
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let records = stmt
            .query_map([], |row| {
                Ok(AccountRecord {
                    id: row.get(0)?,
                    phone_number: row.get(1)?,
                    display_name: row.get(2)?,
                    data_dir: row.get(3)?,
                    auto_start: row.get::<_, i32>(4)? != 0,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| anyhow!("Failed to list accounts: {}", e))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("Failed to collect accounts: {}", e))?;

        Ok(records)
    }

    /// Update the status field of an account.
    pub fn update_status(&self, id: &str, status: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE account SET status = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![status, &now, id],
        )
        .map_err(|e| anyhow!("Failed to update status: {}", e))?;

        Ok(())
    }

    /// Update display_name for an account.
    pub fn update_display_name(&self, id: &str, display_name: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE account SET display_name = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![display_name, &now, id],
        )
        .map_err(|e| anyhow!("Failed to update display_name: {}", e))?;

        Ok(())
    }

    /// Delete an account record.
    pub fn delete_account(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        conn.execute("DELETE FROM account WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| anyhow!("Failed to delete account: {}", e))?;

        Ok(())
    }
}
