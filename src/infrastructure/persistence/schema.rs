//! Database Schema Definitions
//!
//! SQLite schema for core tables: instance, user, access_token, instance_owner.
//! Called once at startup to ensure tables and indexes exist.

use anyhow::Result;
use rusqlite::Connection;
use tracing::info;

/// Apply all schema definitions to the database.
pub fn apply(conn: &Connection) -> Result<()> {
    conn.execute_batch(INSTANCE_SCHEMA)?;
    conn.execute_batch(USER_SCHEMA)?;
    migrate_user_table(conn)?;
    conn.execute_batch(ACCESS_TOKEN_SCHEMA)?;
    conn.execute_batch(INSTANCE_OWNER_SCHEMA)?;
    info!("Database schema applied");
    Ok(())
}

/// Migrate user table to add new columns if they don't exist.
fn migrate_user_table(conn: &Connection) -> Result<()> {
    // Check if email column exists
    let has_email: bool = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('user') WHERE name='email'")?
        .query_row([], |row| row.get::<_, i64>(0))
        .map(|count| count > 0)?;

    if !has_email {
        info!("Migrating user table: adding email column");
        conn.execute("ALTER TABLE user ADD COLUMN email TEXT", [])?;
    }

    // Check if password_hash column exists (for migration from api_key to password auth)
    let has_password_hash: bool = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('user') WHERE name='password_hash'")?
        .query_row([], |row| row.get::<_, i64>(0))
        .map(|count| count > 0)?;

    if !has_password_hash {
        info!("Migrating user table: adding password_hash column");
        conn.execute(
            "ALTER TABLE user ADD COLUMN password_hash TEXT NOT NULL DEFAULT ''",
            [],
        )?;
    }

    // Create indexes after columns exist
    conn.execute_batch(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_user_email ON user(email) WHERE email IS NOT NULL;",
    )?;

    Ok(())
}

/// Instance table schema.
const INSTANCE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS instance (
    id                TEXT PRIMARY KEY,
    phone_number      TEXT NOT NULL CHECK(length(phone_number) >= 7 AND length(phone_number) <= 15),
    instance_name      TEXT NOT NULL DEFAULT 'unknown',
    data_dir          TEXT NOT NULL,
    idle_timeout INTEGER NOT NULL DEFAULT 300,
    status            TEXT NOT NULL DEFAULT 'sleeping',
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_instance_phone ON instance(phone_number);
"#;

/// User table schema for RBAC.
/// Roles: 'admin', 'user'
/// Users authenticate via password (API).
const USER_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS user (
    id            TEXT PRIMARY KEY,
    username      TEXT UNIQUE NOT NULL,
    email         TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    role          TEXT NOT NULL DEFAULT 'user' CHECK(role IN ('admin', 'user')),
    is_active     INTEGER NOT NULL DEFAULT 1,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_username ON user(username);
"#;

/// Access token table for API authentication.
/// Users can have multiple tokens for different integrations.
const ACCESS_TOKEN_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS access_token (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL,
    name        TEXT NOT NULL DEFAULT 'default',
    token_hash  TEXT UNIQUE NOT NULL,
    expires_at  TEXT,
    last_used   TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_access_token_user ON access_token(user_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_access_token_hash ON access_token(token_hash);
"#;

/// Instance ownership table for user-instance relationships.
/// Permissions: 'owner', 'operator', 'viewer'
const INSTANCE_OWNER_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS instance_owner (
    user_id     TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    permission  TEXT NOT NULL DEFAULT 'owner' CHECK(permission IN ('owner', 'operator', 'viewer')),
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, instance_id),
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE,
    FOREIGN KEY (instance_id) REFERENCES instance(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_instance_owner_user ON instance_owner(user_id);
CREATE INDEX IF NOT EXISTS idx_instance_owner_instance ON instance_owner(instance_id);
"#;
