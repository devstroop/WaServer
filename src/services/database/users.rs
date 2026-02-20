//! User Database Operations
//!
//! CRUD operations for users, instance ownership, and access control.

use anyhow::Result;
use chrono::Utc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::models::user::{
    InstanceAccess, InstanceOwnership, InstancePermissions, User, UserId,
};

use super::DatabaseService;

impl DatabaseService {
    // =========================================================================
    // Schema Initialization
    // =========================================================================

    /// Initialize user-related tables
    pub fn init_user_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Users table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE COLLATE NOCASE,
                password_hash TEXT NOT NULL,
                email TEXT,
                display_name TEXT,
                is_active INTEGER DEFAULT 1,
                is_admin INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_login_at TEXT
            )",
            [],
        )?;

        // Instances table (ownership)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS instances (
                id TEXT PRIMARY KEY,
                owner_id TEXT NOT NULL,
                display_name TEXT,
                description TEXT,
                is_active INTEGER DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Instance access table (sharing)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS instance_access (
                instance_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                can_read INTEGER DEFAULT 1,
                can_send INTEGER DEFAULT 0,
                can_manage INTEGER DEFAULT 0,
                granted_by TEXT NOT NULL,
                granted_at TEXT NOT NULL,
                expires_at TEXT,
                PRIMARY KEY (instance_id, user_id),
                FOREIGN KEY (instance_id) REFERENCES instances(id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (granted_by) REFERENCES users(id)
            )",
            [],
        )?;

        // Refresh tokens table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS refresh_tokens (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                issued_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                device_name TEXT,
                device_fingerprint TEXT,
                revoked_at TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Password reset tokens table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS password_reset_tokens (
                token TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                used_at TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_users_username ON users(username COLLATE NOCASE)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_instances_owner ON instances(owner_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_instance_access_user ON instance_access(user_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user ON refresh_tokens(user_id)",
            [],
        )?;

        debug!("User schema initialized");
        Ok(())
    }

    // =========================================================================
    // User CRUD Operations
    // =========================================================================

    /// Create a new user
    pub fn create_user(&self, user: &User) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO users (id, username, password_hash, email, display_name, is_active, is_admin, created_at, updated_at, last_login_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                user.id.to_string(),
                user.username,
                user.password_hash,
                user.email,
                user.display_name,
                user.is_active as i32,
                user.is_admin as i32,
                user.created_at.to_rfc3339(),
                user.updated_at.to_rfc3339(),
                user.last_login_at.map(|dt| dt.to_rfc3339()),
            ],
        )?;
        info!("Created user: {} ({})", user.username, user.id);
        Ok(())
    }

    /// Get user by ID
    pub fn get_user_by_id(&self, id: UserId) -> Result<Option<User>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, email, display_name, is_active, is_admin, created_at, updated_at, last_login_at
             FROM users WHERE id = ?1",
        )?;

        let user = stmt
            .query_row([id.to_string()], Self::row_to_user)
            .ok();

        Ok(user)
    }

    /// Get user by username (case-insensitive)
    pub fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, email, display_name, is_active, is_admin, created_at, updated_at, last_login_at
             FROM users WHERE username = ?1 COLLATE NOCASE",
        )?;

        let user = stmt
            .query_row([username], Self::row_to_user)
            .ok();

        Ok(user)
    }

    /// List all users with optional filters
    pub fn list_users(
        &self,
        active: Option<bool>,
        admin: Option<bool>,
        search: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<User>> {
        let conn = self.conn.lock().unwrap();
        
        let mut sql = String::from(
            "SELECT id, username, password_hash, email, display_name, is_active, is_admin, created_at, updated_at, last_login_at
             FROM users WHERE 1=1"
        );
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(active) = active {
            sql.push_str(" AND is_active = ?");
            params.push(Box::new(active as i32));
        }

        if let Some(admin) = admin {
            sql.push_str(" AND is_admin = ?");
            params.push(Box::new(admin as i32));
        }

        if let Some(search) = search {
            sql.push_str(" AND (username LIKE ? OR email LIKE ?)");
            let pattern = format!("%{}%", search);
            params.push(Box::new(pattern.clone()));
            params.push(Box::new(pattern));
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        params.push(Box::new(limit as i64));
        params.push(Box::new(offset as i64));

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let users = stmt
            .query_map(params_refs.as_slice(), Self::row_to_user)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(users)
    }

    /// Count total users
    pub fn count_users(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Update user
    pub fn update_user(&self, user: &User) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET 
                username = ?2, password_hash = ?3, email = ?4, display_name = ?5,
                is_active = ?6, is_admin = ?7, updated_at = ?8, last_login_at = ?9
             WHERE id = ?1",
            rusqlite::params![
                user.id.to_string(),
                user.username,
                user.password_hash,
                user.email,
                user.display_name,
                user.is_active as i32,
                user.is_admin as i32,
                Utc::now().to_rfc3339(),
                user.last_login_at.map(|dt| dt.to_rfc3339()),
            ],
        )?;
        debug!("Updated user: {}", user.id);
        Ok(())
    }

    /// Update user's last login timestamp
    pub fn update_user_last_login(&self, id: UserId) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE users SET last_login_at = ?2, updated_at = ?2 WHERE id = ?1",
            rusqlite::params![id.to_string(), now],
        )?;
        Ok(())
    }

    /// Update user's password hash
    pub fn update_user_password(&self, id: UserId, password_hash: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE users SET password_hash = ?2, updated_at = ?3 WHERE id = ?1",
            rusqlite::params![id.to_string(), password_hash, now],
        )?;
        info!("Updated password for user: {}", id);
        Ok(())
    }

    /// Delete user
    pub fn delete_user(&self, id: UserId) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM users WHERE id = ?1", [id.to_string()])?;
        if rows > 0 {
            info!("Deleted user: {}", id);
        }
        Ok(rows > 0)
    }

    /// Check if any users exist (for initial setup)
    pub fn has_users(&self) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
        Ok(count > 0)
    }

    /// Helper: Convert row to User
    fn row_to_user(row: &rusqlite::Row) -> rusqlite::Result<User> {
        let id_str: String = row.get(0)?;
        let created_str: String = row.get(7)?;
        let updated_str: String = row.get(8)?;
        let last_login_str: Option<String> = row.get(9)?;

        Ok(User {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            username: row.get(1)?,
            password_hash: row.get(2)?,
            email: row.get(3)?,
            display_name: row.get(4)?,
            is_active: row.get::<_, i32>(5)? != 0,
            is_admin: row.get::<_, i32>(6)? != 0,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            last_login_at: last_login_str.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
        })
    }

    // =========================================================================
    // Instance Ownership Operations
    // =========================================================================

    /// Register instance ownership
    pub fn create_instance_ownership(&self, ownership: &InstanceOwnership) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO instances (id, owner_id, display_name, description, is_active, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                ownership.instance_id,
                ownership.owner_id.to_string(),
                ownership.display_name,
                ownership.description,
                ownership.is_active as i32,
                ownership.created_at.to_rfc3339(),
                ownership.updated_at.to_rfc3339(),
            ],
        )?;
        info!("Created instance ownership: {} -> {}", ownership.instance_id, ownership.owner_id);
        Ok(())
    }

    /// Get instance ownership by instance ID
    pub fn get_instance_ownership(&self, instance_id: &str) -> Result<Option<InstanceOwnership>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, owner_id, display_name, description, is_active, created_at, updated_at
             FROM instances WHERE id = ?1",
        )?;

        let ownership = stmt
            .query_row([instance_id], Self::row_to_instance_ownership)
            .ok();

        Ok(ownership)
    }

    /// List instances owned by a user
    pub fn list_instances_by_owner(&self, owner_id: UserId) -> Result<Vec<InstanceOwnership>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, owner_id, display_name, description, is_active, created_at, updated_at
             FROM instances WHERE owner_id = ?1 ORDER BY created_at DESC",
        )?;

        let instances = stmt
            .query_map([owner_id.to_string()], Self::row_to_instance_ownership)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(instances)
    }

    /// List all instances a user has access to (owned + shared)
    pub fn list_accessible_instances(&self, user_id: UserId) -> Result<Vec<(InstanceOwnership, InstancePermissions)>> {
        let conn = self.conn.lock().unwrap();
        
        // First get owned instances
        let mut stmt = conn.prepare(
            "SELECT id, owner_id, display_name, description, is_active, created_at, updated_at
             FROM instances WHERE owner_id = ?1 ORDER BY created_at DESC",
        )?;
        let owned: Vec<(InstanceOwnership, InstancePermissions)> = stmt
            .query_map([user_id.to_string()], Self::row_to_instance_ownership)?
            .filter_map(|r| r.ok())
            .map(|o| (o, InstancePermissions::owner()))
            .collect();

        // Then get shared instances
        let mut stmt = conn.prepare(
            "SELECT i.id, i.owner_id, i.display_name, i.description, i.is_active, i.created_at, i.updated_at,
                    a.can_read, a.can_send, a.can_manage
             FROM instances i
             JOIN instance_access a ON i.id = a.instance_id
             WHERE a.user_id = ?1 AND (a.expires_at IS NULL OR a.expires_at > ?2)
             ORDER BY i.created_at DESC",
        )?;
        let now = Utc::now().to_rfc3339();
        let shared: Vec<(InstanceOwnership, InstancePermissions)> = stmt
            .query_map(rusqlite::params![user_id.to_string(), now], |row| {
                let ownership = Self::row_to_instance_ownership(row)?;
                let perms = InstancePermissions {
                    can_read: row.get::<_, i32>(7)? != 0,
                    can_send: row.get::<_, i32>(8)? != 0,
                    can_manage: row.get::<_, i32>(9)? != 0,
                    can_delete: false,
                    can_share: false,
                };
                Ok((ownership, perms))
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Combine owned and shared
        let mut all = owned;
        all.extend(shared);
        Ok(all)
    }

    /// Update instance ownership
    pub fn update_instance_ownership(&self, ownership: &InstanceOwnership) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE instances SET 
                owner_id = ?2, display_name = ?3, description = ?4, is_active = ?5, updated_at = ?6
             WHERE id = ?1",
            rusqlite::params![
                ownership.instance_id,
                ownership.owner_id.to_string(),
                ownership.display_name,
                ownership.description,
                ownership.is_active as i32,
                Utc::now().to_rfc3339(),
            ],
        )?;
        debug!("Updated instance ownership: {}", ownership.instance_id);
        Ok(())
    }

    /// Delete instance ownership
    pub fn delete_instance_ownership(&self, instance_id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM instances WHERE id = ?1", [instance_id])?;
        if rows > 0 {
            info!("Deleted instance ownership: {}", instance_id);
        }
        Ok(rows > 0)
    }

    /// Transfer instance ownership to another user
    pub fn transfer_instance_ownership(&self, instance_id: &str, new_owner_id: UserId) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE instances SET owner_id = ?2, updated_at = ?3 WHERE id = ?1",
            rusqlite::params![instance_id, new_owner_id.to_string(), now],
        )?;
        info!("Transferred instance {} to user {}", instance_id, new_owner_id);
        Ok(())
    }

    /// Helper: Convert row to InstanceOwnership
    fn row_to_instance_ownership(row: &rusqlite::Row) -> rusqlite::Result<InstanceOwnership> {
        let owner_id_str: String = row.get(1)?;
        let created_str: String = row.get(5)?;
        let updated_str: String = row.get(6)?;

        Ok(InstanceOwnership {
            instance_id: row.get(0)?,
            owner_id: Uuid::parse_str(&owner_id_str).unwrap_or_default(),
            display_name: row.get(2)?,
            description: row.get(3)?,
            is_active: row.get::<_, i32>(4)? != 0,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    // =========================================================================
    // Instance Access Operations (Sharing)
    // =========================================================================

    /// Grant access to an instance
    pub fn grant_instance_access(&self, access: &InstanceAccess) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO instance_access 
             (instance_id, user_id, can_read, can_send, can_manage, granted_by, granted_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                access.instance_id,
                access.user_id.to_string(),
                access.can_read as i32,
                access.can_send as i32,
                access.can_manage as i32,
                access.granted_by.to_string(),
                access.granted_at.to_rfc3339(),
                access.expires_at.map(|dt| dt.to_rfc3339()),
            ],
        )?;
        info!(
            "Granted instance access: {} -> user {}",
            access.instance_id, access.user_id
        );
        Ok(())
    }

    /// Get instance access for a user
    pub fn get_instance_access(&self, instance_id: &str, user_id: UserId) -> Result<Option<InstanceAccess>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT instance_id, user_id, can_read, can_send, can_manage, granted_by, granted_at, expires_at
             FROM instance_access WHERE instance_id = ?1 AND user_id = ?2",
        )?;

        let access = stmt
            .query_row(rusqlite::params![instance_id, user_id.to_string()], Self::row_to_instance_access)
            .ok();

        Ok(access)
    }

    /// List all users with access to an instance
    pub fn list_instance_access(&self, instance_id: &str) -> Result<Vec<InstanceAccess>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT instance_id, user_id, can_read, can_send, can_manage, granted_by, granted_at, expires_at
             FROM instance_access WHERE instance_id = ?1 ORDER BY granted_at DESC",
        )?;

        let access_list = stmt
            .query_map([instance_id], Self::row_to_instance_access)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(access_list)
    }

    /// Revoke instance access
    pub fn revoke_instance_access(&self, instance_id: &str, user_id: UserId) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM instance_access WHERE instance_id = ?1 AND user_id = ?2",
            rusqlite::params![instance_id, user_id.to_string()],
        )?;
        if rows > 0 {
            info!("Revoked instance access: {} from user {}", instance_id, user_id);
        }
        Ok(rows > 0)
    }

    /// Check if user has access to instance (owner or shared)
    pub fn check_instance_access(&self, instance_id: &str, user_id: UserId) -> Result<Option<InstancePermissions>> {
        // First check if user is owner
        if let Some(ownership) = self.get_instance_ownership(instance_id)? {
            if ownership.owner_id == user_id {
                return Ok(Some(InstancePermissions::owner()));
            }
        }

        // Then check shared access
        if let Some(access) = self.get_instance_access(instance_id, user_id)? {
            // Check expiry
            if let Some(expires) = access.expires_at {
                if expires < Utc::now() {
                    return Ok(None);
                }
            }
            return Ok(Some(InstancePermissions::from_access(&access)));
        }

        // Check if user is admin (admins have full access)
        if let Some(user) = self.get_user_by_id(user_id)? {
            if user.is_admin {
                return Ok(Some(InstancePermissions::admin()));
            }
        }

        Ok(None)
    }

    /// Helper: Convert row to InstanceAccess
    fn row_to_instance_access(row: &rusqlite::Row) -> rusqlite::Result<InstanceAccess> {
        let user_id_str: String = row.get(1)?;
        let granted_by_str: String = row.get(5)?;
        let granted_at_str: String = row.get(6)?;
        let expires_at_str: Option<String> = row.get(7)?;

        Ok(InstanceAccess {
            instance_id: row.get(0)?,
            user_id: Uuid::parse_str(&user_id_str).unwrap_or_default(),
            can_read: row.get::<_, i32>(2)? != 0,
            can_send: row.get::<_, i32>(3)? != 0,
            can_manage: row.get::<_, i32>(4)? != 0,
            granted_by: Uuid::parse_str(&granted_by_str).unwrap_or_default(),
            granted_at: chrono::DateTime::parse_from_rfc3339(&granted_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            expires_at: expires_at_str.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
        })
    }

    // =========================================================================
    // Refresh Token Operations
    // =========================================================================

    /// Store a refresh token
    pub fn store_refresh_token(
        &self,
        token_id: &str,
        user_id: UserId,
        expires_at: chrono::DateTime<Utc>,
        device_name: Option<&str>,
        device_fingerprint: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO refresh_tokens (id, user_id, issued_at, expires_at, device_name, device_fingerprint, revoked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL)",
            rusqlite::params![
                token_id,
                user_id.to_string(),
                now,
                expires_at.to_rfc3339(),
                device_name,
                device_fingerprint,
            ],
        )?;
        debug!("Stored refresh token for user: {}", user_id);
        Ok(())
    }

    /// Check if a refresh token is valid (not revoked, not expired)
    pub fn is_refresh_token_valid(&self, token_id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        let valid: bool = conn
            .query_row(
                "SELECT 1 FROM refresh_tokens WHERE id = ?1 AND revoked_at IS NULL AND expires_at > ?2",
                rusqlite::params![token_id, now],
                |_| Ok(true),
            )
            .unwrap_or(false);
        Ok(valid)
    }

    /// Revoke a refresh token
    pub fn revoke_refresh_token(&self, token_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE refresh_tokens SET revoked_at = ?2 WHERE id = ?1",
            rusqlite::params![token_id, now],
        )?;
        debug!("Revoked refresh token: {}", token_id);
        Ok(())
    }

    /// Revoke all refresh tokens for a user (logout from all devices)
    pub fn revoke_all_user_tokens(&self, user_id: UserId) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        let rows = conn.execute(
            "UPDATE refresh_tokens SET revoked_at = ?2 WHERE user_id = ?1 AND revoked_at IS NULL",
            rusqlite::params![user_id.to_string(), now],
        )?;
        if rows > 0 {
            info!("Revoked {} refresh tokens for user: {}", rows, user_id);
        }
        Ok(rows)
    }

    /// Clean up expired refresh tokens
    pub fn cleanup_expired_tokens(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        let rows = conn.execute(
            "DELETE FROM refresh_tokens WHERE expires_at < ?1 OR revoked_at IS NOT NULL",
            [now],
        )?;
        if rows > 0 {
            debug!("Cleaned up {} expired/revoked refresh tokens", rows);
        }
        Ok(rows)
    }

    // =========================================================================
    // Password Reset Token Operations
    // =========================================================================

    /// Store a password reset token
    pub fn store_password_reset_token(
        &self,
        token: &str,
        user_id: UserId,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // First delete any existing tokens for this user
        conn.execute(
            "DELETE FROM password_reset_tokens WHERE user_id = ?1",
            [user_id.to_string()],
        )?;
        // Then insert the new token
        conn.execute(
            "INSERT INTO password_reset_tokens (token, user_id, expires_at, used_at) VALUES (?1, ?2, ?3, NULL)",
            rusqlite::params![token, user_id.to_string(), expires_at.to_rfc3339()],
        )?;
        debug!("Stored password reset token for user: {}", user_id);
        Ok(())
    }

    /// Validate and get user ID for a password reset token
    pub fn validate_password_reset_token(&self, token: &str) -> Result<Option<UserId>> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        let user_id: Option<String> = conn
            .query_row(
                "SELECT user_id FROM password_reset_tokens WHERE token = ?1 AND used_at IS NULL AND expires_at > ?2",
                rusqlite::params![token, now],
                |row| row.get(0),
            )
            .ok();
        
        Ok(user_id.and_then(|s| Uuid::parse_str(&s).ok()))
    }

    /// Mark password reset token as used
    pub fn use_password_reset_token(&self, token: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE password_reset_tokens SET used_at = ?2 WHERE token = ?1",
            rusqlite::params![token, now],
        )?;
        debug!("Marked password reset token as used");
        Ok(())
    }

    /// Clean up expired password reset tokens
    pub fn cleanup_expired_reset_tokens(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        let rows = conn.execute(
            "DELETE FROM password_reset_tokens WHERE expires_at < ?1 OR used_at IS NOT NULL",
            [now],
        )?;
        if rows > 0 {
            debug!("Cleaned up {} expired password reset tokens", rows);
        }
        Ok(rows)
    }
}
