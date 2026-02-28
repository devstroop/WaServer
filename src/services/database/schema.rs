//! Database Schema Definitions
//!
//! SQLite schema for core tables: instance, user, instance_owner.
//! Called once at startup to ensure tables and indexes exist.

use anyhow::Result;
use rusqlite::Connection;
use tracing::info;

/// Apply all schema definitions to the database.
pub fn apply(conn: &Connection) -> Result<()> {
    conn.execute_batch(INSTANCE_SCHEMA)?;
    conn.execute_batch(USER_SCHEMA)?;
    conn.execute_batch(INSTANCE_OWNER_SCHEMA)?;
    info!("Database schema applied");
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
const USER_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS user (
    id          TEXT PRIMARY KEY,
    username    TEXT UNIQUE NOT NULL,
    api_key     TEXT UNIQUE NOT NULL,
    role        TEXT NOT NULL DEFAULT 'user' CHECK(role IN ('admin', 'user')),
    is_active   INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_username ON user(username);
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_api_key ON user(api_key);
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
