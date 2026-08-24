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
use crate::models::user::{
    AccessTokenRecord, InstanceOwnerRecord, InstancePermission, UserRecord, UserRole,
};

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

    /// Update idle_timeout for an instance (part of #6 — config updates without full row rewrite).
    pub fn update_idle_timeout(&self, id: &str, idle_timeout: u64) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE instance SET idle_timeout = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![idle_timeout as i64, &now, id],
        )
        .map_err(|e| anyhow!("Failed to update idle_timeout: {}", e))?;

        Ok(())
    }

    /// Delete an instance record.
    pub fn delete_instance(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        conn.execute("DELETE FROM instance WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| anyhow!("Failed to delete instance: {}", e))?;

        Ok(())
    }

    // ========================================================================
    // User CRUD
    // ========================================================================

    /// Create a new user with password.
    pub fn create_user(
        &self,
        id: &str,
        username: &str,
        email: Option<&str>,
        password_hash: &str,
        role: UserRole,
    ) -> Result<UserRecord> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        // Check uniqueness of username
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM user WHERE username = ?1 LIMIT 1",
                rusqlite::params![username],
                |row| row.get(0),
            )
            .ok();

        if existing.is_some() {
            return Err(anyhow!("Username '{}' already exists", username));
        }

        // Check uniqueness of email if provided
        if let Some(email_val) = email {
            let existing_email: Option<String> = conn
                .query_row(
                    "SELECT id FROM user WHERE email = ?1 LIMIT 1",
                    rusqlite::params![email_val],
                    |row| row.get(0),
                )
                .ok();

            if existing_email.is_some() {
                return Err(anyhow!("Email '{}' already exists", email_val));
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO user (id, username, email, password_hash, role, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
            rusqlite::params![id, username, email, password_hash, role.to_string(), &now, &now],
        )
        .map_err(|e| anyhow!("Failed to create user: {}", e))?;

        Ok(UserRecord {
            id: id.to_string(),
            username: username.to_string(),
            email: email.map(|s| s.to_string()),
            password_hash: password_hash.to_string(),
            role,
            is_active: true,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        })
    }

    /// Get a user by ID.
    pub fn get_user(&self, id: &str) -> Result<Option<UserRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT id, username, email, password_hash, role, is_active, created_at, updated_at FROM user WHERE id = ?1")
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let record = stmt
            .query_row(rusqlite::params![id], |row| {
                let role_str: String = row.get(4)?;
                Ok(UserRecord {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    role: role_str.parse().unwrap_or(UserRole::User),
                    is_active: row.get::<_, i64>(5)? != 0,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .ok();

        Ok(record)
    }

    /// Get a user by username.
    pub fn get_user_by_username(&self, username: &str) -> Result<Option<UserRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT id, username, email, password_hash, role, is_active, created_at, updated_at FROM user WHERE username = ?1 LIMIT 1")
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let record = stmt
            .query_row(rusqlite::params![username], |row| {
                let role_str: String = row.get(4)?;
                Ok(UserRecord {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    role: role_str.parse().unwrap_or(UserRole::User),
                    is_active: row.get::<_, i64>(5)? != 0,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .ok();

        Ok(record)
    }

    /// Get a user by email.
    pub fn get_user_by_email(&self, email: &str) -> Result<Option<UserRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT id, username, email, password_hash, role, is_active, created_at, updated_at FROM user WHERE email = ?1 AND is_active = 1 LIMIT 1")
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let record = stmt
            .query_row(rusqlite::params![email], |row| {
                let role_str: String = row.get(4)?;
                Ok(UserRecord {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    role: role_str.parse().unwrap_or(UserRole::User),
                    is_active: row.get::<_, i64>(5)? != 0,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .ok();

        Ok(record)
    }

    /// List all users.
    pub fn list_users(&self) -> Result<Vec<UserRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT id, username, email, password_hash, role, is_active, created_at, updated_at FROM user ORDER BY created_at DESC")
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let records = stmt
            .query_map([], |row| {
                let role_str: String = row.get(4)?;
                Ok(UserRecord {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    role: role_str.parse().unwrap_or(UserRole::User),
                    is_active: row.get::<_, i64>(5)? != 0,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| anyhow!("Failed to list users: {}", e))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("Failed to collect users: {}", e))?;

        Ok(records)
    }

    /// Update user fields.
    pub fn update_user(
        &self,
        id: &str,
        username: Option<&str>,
        email: Option<Option<&str>>,
        password_hash: Option<&str>,
        role: Option<UserRole>,
        is_active: Option<bool>,
    ) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let now = chrono::Utc::now().to_rfc3339();

        // Build dynamic update query
        let mut updates = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        updates.push("updated_at = ?".to_string());
        params.push(Box::new(now.clone()));

        if let Some(u) = username {
            updates.push("username = ?".to_string());
            params.push(Box::new(u.to_string()));
        }
        if let Some(e) = email {
            updates.push("email = ?".to_string());
            params.push(Box::new(e.map(|s| s.to_string())));
        }
        if let Some(p) = password_hash {
            updates.push("password_hash = ?".to_string());
            params.push(Box::new(p.to_string()));
        }
        if let Some(r) = role {
            updates.push("role = ?".to_string());
            params.push(Box::new(r.to_string()));
        }
        if let Some(a) = is_active {
            updates.push("is_active = ?".to_string());
            params.push(Box::new(a as i64));
        }

        params.push(Box::new(id.to_string()));

        let query = format!("UPDATE user SET {} WHERE id = ?", updates.join(", "));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        conn.execute(&query, params_refs.as_slice())
            .map_err(|e| anyhow!("Failed to update user: {}", e))?;

        Ok(())
    }

    /// Delete a user.
    pub fn delete_user(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        conn.execute("DELETE FROM user WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| anyhow!("Failed to delete user: {}", e))?;

        Ok(())
    }

    // ========================================================================
    // Access Token CRUD
    // ========================================================================

    /// Create a new access token for a user.
    pub fn create_access_token(
        &self,
        id: &str,
        user_id: &str,
        name: &str,
        token_hash: &str,
        expires_at: Option<&str>,
    ) -> Result<AccessTokenRecord> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO access_token (id, user_id, name, token_hash, expires_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, user_id, name, token_hash, expires_at, &now],
        )
        .map_err(|e| anyhow!("Failed to create access token: {}", e))?;

        Ok(AccessTokenRecord {
            id: id.to_string(),
            user_id: user_id.to_string(),
            name: name.to_string(),
            token_hash: token_hash.to_string(),
            expires_at: expires_at.map(|s| s.to_string()),
            last_used: None,
            created_at: Some(now),
        })
    }

    /// Get user by access token hash (for API authentication).
    pub fn get_user_by_access_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<(UserRecord, AccessTokenRecord)>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare(
                r#"SELECT u.id, u.username, u.email, u.password_hash, u.role, u.is_active, u.created_at, u.updated_at,
                          t.id, t.user_id, t.name, t.token_hash, t.expires_at, t.last_used, t.created_at
                   FROM access_token t
                   JOIN user u ON t.user_id = u.id
                   WHERE t.token_hash = ?1 AND u.is_active = 1
                     AND (t.expires_at IS NULL OR t.expires_at > datetime('now'))
                   LIMIT 1"#,
            )
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let result = stmt
            .query_row(rusqlite::params![token_hash], |row| {
                let role_str: String = row.get(4)?;
                let user = UserRecord {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    role: role_str.parse().unwrap_or(UserRole::User),
                    is_active: row.get::<_, i64>(5)? != 0,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                };
                let token = AccessTokenRecord {
                    id: row.get(8)?,
                    user_id: row.get(9)?,
                    name: row.get(10)?,
                    token_hash: row.get(11)?,
                    expires_at: row.get(12)?,
                    last_used: row.get(13)?,
                    created_at: row.get(14)?,
                };
                Ok((user, token))
            })
            .ok();

        // Update last_used timestamp
        if let Some((_, ref token)) = result {
            let now = chrono::Utc::now().to_rfc3339();
            let _ = conn.execute(
                "UPDATE access_token SET last_used = ?1 WHERE id = ?2",
                rusqlite::params![&now, &token.id],
            );
        }

        Ok(result)
    }

    /// List all access tokens for a user.
    pub fn list_user_access_tokens(&self, user_id: &str) -> Result<Vec<AccessTokenRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT id, user_id, name, token_hash, expires_at, last_used, created_at FROM access_token WHERE user_id = ?1 ORDER BY created_at DESC")
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let records = stmt
            .query_map(rusqlite::params![user_id], |row| {
                Ok(AccessTokenRecord {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    name: row.get(2)?,
                    token_hash: row.get(3)?,
                    expires_at: row.get(4)?,
                    last_used: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| anyhow!("Failed to list access tokens: {}", e))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("Failed to collect access tokens: {}", e))?;

        Ok(records)
    }

    /// Delete every "Web Session" token for a user (logout-all, #42).
    /// Returns the number of sessions revoked.
    pub fn delete_user_web_sessions(&self, user_id: &str) -> Result<usize> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let n = conn
            .execute(
                "DELETE FROM access_token WHERE user_id = ?1 AND name = 'Web Session'",
                rusqlite::params![user_id],
            )
            .map_err(|e| anyhow!("Failed to delete web sessions: {}", e))?;
        Ok(n)
    }

    /// Delete an access token.
    pub fn delete_access_token(&self, id: &str, user_id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        conn.execute(
            "DELETE FROM access_token WHERE id = ?1 AND user_id = ?2",
            rusqlite::params![id, user_id],
        )
        .map_err(|e| anyhow!("Failed to delete access token: {}", e))?;

        Ok(())
    }

    // ========================================================================
    // Instance Ownership CRUD
    // ========================================================================

    /// Assign instance permission to a user.
    pub fn assign_instance_to_user(
        &self,
        user_id: &str,
        instance_id: &str,
        permission: InstancePermission,
    ) -> Result<InstanceOwnerRecord> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT OR REPLACE INTO instance_owner (user_id, instance_id, permission, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![user_id, instance_id, permission.to_string(), &now],
        )
        .map_err(|e| anyhow!("Failed to assign instance: {}", e))?;

        Ok(InstanceOwnerRecord {
            user_id: user_id.to_string(),
            instance_id: instance_id.to_string(),
            permission,
            created_at: Some(now),
        })
    }

    /// Get user's permission for an instance.
    pub fn get_instance_permission(
        &self,
        user_id: &str,
        instance_id: &str,
    ) -> Result<Option<InstancePermission>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let permission: Option<String> = conn
            .query_row(
                "SELECT permission FROM instance_owner WHERE user_id = ?1 AND instance_id = ?2",
                rusqlite::params![user_id, instance_id],
                |row| row.get(0),
            )
            .ok();

        Ok(permission.and_then(|p| p.parse().ok()))
    }

    /// List all instances a user has access to.
    pub fn list_user_instances(&self, user_id: &str) -> Result<Vec<InstanceOwnerRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT user_id, instance_id, permission, created_at FROM instance_owner WHERE user_id = ?1")
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let records = stmt
            .query_map(rusqlite::params![user_id], |row| {
                let perm_str: String = row.get(2)?;
                Ok(InstanceOwnerRecord {
                    user_id: row.get(0)?,
                    instance_id: row.get(1)?,
                    permission: perm_str.parse().unwrap_or(InstancePermission::Viewer),
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| anyhow!("Failed to list user instances: {}", e))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("Failed to collect user instances: {}", e))?;

        Ok(records)
    }

    /// List all users with access to an instance.
    pub fn list_instance_users(&self, instance_id: &str) -> Result<Vec<InstanceOwnerRecord>> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT user_id, instance_id, permission, created_at FROM instance_owner WHERE instance_id = ?1")
            .map_err(|e| anyhow!("Failed to prepare query: {}", e))?;

        let records = stmt
            .query_map(rusqlite::params![instance_id], |row| {
                let perm_str: String = row.get(2)?;
                Ok(InstanceOwnerRecord {
                    user_id: row.get(0)?,
                    instance_id: row.get(1)?,
                    permission: perm_str.parse().unwrap_or(InstancePermission::Viewer),
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| anyhow!("Failed to list instance users: {}", e))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("Failed to collect instance users: {}", e))?;

        Ok(records)
    }

    /// Remove user's access to an instance.
    pub fn remove_instance_from_user(&self, user_id: &str, instance_id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow!("Lock error: {}", e))?;

        conn.execute(
            "DELETE FROM instance_owner WHERE user_id = ?1 AND instance_id = ?2",
            rusqlite::params![user_id, instance_id],
        )
        .map_err(|e| anyhow!("Failed to remove instance access: {}", e))?;

        Ok(())
    }
}
