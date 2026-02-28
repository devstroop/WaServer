//! Database Service
//!
//! Manages an embedded SQLite database (file-based).
//! Provides typed helpers for instance CRUD operations.

use anyhow::{anyhow, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::info;

use super::schema;

/// Persistent instance record stored in SQLite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceRecord {
    pub id: String,
    pub phone_number: String,
    pub instance_name: String,
    pub data_dir: String,
    pub idle_timeout: u64,
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
    // Instance CRUD
    // ========================================================================

    /// Insert a new account. The record ID is the instance UUID (string).
    pub fn create_instance(
        &self,
        id: &str,
        phone_number: &str,
        instance_name: &str,
        data_dir: &str,
        idle_timeout: u64,
    ) -> Result<InstanceRecord> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        // Check uniqueness of phone_number explicitly for a clear error message
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM instance WHERE phone_number = ?1 LIMIT 1",
                rusqlite::params![phone_number],
                |row| row.get(0),
            )
            .ok();

        if existing.is_some() {
            return Err(anyhow!("Phone number '{}' already exists", phone_number));
        }

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO instance (id, phone_number, instance_name, data_dir, idle_timeout, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'sleeping', ?6, ?7)",
            rusqlite::params![id, phone_number, instance_name, data_dir, idle_timeout as i64, &now, &now],
        )
        .map_err(|e| anyhow!("Failed to create instance: {}", e))?;

        Ok(InstanceRecord {
            id: id.to_string(),
            phone_number: phone_number.to_string(),
            instance_name: instance_name.to_string(),
            data_dir: data_dir.to_string(),
            idle_timeout,
            status: "sleeping".to_string(),
            created_at: Some(now.clone()),
            updated_at: Some(now),
        })
    }

    /// Get an instance by its UUID string id.
    pub fn get_instance(&self, id: &str) -> Result<Option<InstanceRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT id, phone_number, instance_name, data_dir, idle_timeout, status, created_at, updated_at FROM instance WHERE id = ?1")
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let record = stmt
            .query_row(rusqlite::params![id], |row| {
                Ok(InstanceRecord {
                    id: row.get(0)?,
                    phone_number: row.get(1)?,
                    instance_name: row.get(2)?,
                    data_dir: row.get(3)?,
                    idle_timeout: row.get::<_, i64>(4)? as u64,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .ok();

        Ok(record)
    }

    /// Find an instance by phone number.
    pub fn get_instance_by_phone(&self, phone_number: &str) -> Result<Option<InstanceRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT id, phone_number, instance_name, data_dir, idle_timeout, status, created_at, updated_at FROM instance WHERE phone_number = ?1 LIMIT 1")
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let record = stmt
            .query_row(rusqlite::params![phone_number], |row| {
                Ok(InstanceRecord {
                    id: row.get(0)?,
                    phone_number: row.get(1)?,
                    instance_name: row.get(2)?,
                    data_dir: row.get(3)?,
                    idle_timeout: row.get::<_, i64>(4)? as u64,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .ok();

        Ok(record)
    }

    /// List all instances.
    pub fn list_instances(&self) -> Result<Vec<InstanceRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT id, phone_number, instance_name, data_dir, idle_timeout, status, created_at, updated_at FROM instance")
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let records = stmt
            .query_map([], |row| {
                Ok(InstanceRecord {
                    id: row.get(0)?,
                    phone_number: row.get(1)?,
                    instance_name: row.get(2)?,
                    data_dir: row.get(3)?,
                    idle_timeout: row.get::<_, i64>(4)? as u64,
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| anyhow!("Failed to list instances: {}", e))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("Failed to collect instances: {}", e))?;

        Ok(records)
    }

    /// Update the status field of an instance.
    pub fn update_status(&self, id: &str, status: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE instance SET status = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![status, &now, id],
        )
        .map_err(|e| anyhow!("Failed to update status: {}", e))?;

        Ok(())
    }

    /// Update instance_name for an instance.
    pub fn update_instance_name(&self, id: &str, instance_name: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE instance SET instance_name = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![instance_name, &now, id],
        )
        .map_err(|e| anyhow!("Failed to update instance_name: {}", e))?;

        Ok(())
    }

    /// Delete an instance record.
    pub fn delete_instance(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        conn.execute("DELETE FROM instance WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| anyhow!("Failed to delete instance: {}", e))?;

        Ok(())
    }
}
