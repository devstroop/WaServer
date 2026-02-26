//! Database Schema Definitions
//!
//! SQLite schema for the `account` table.
//! Called once at startup to ensure tables and indexes exist.

use anyhow::Result;
use rusqlite::Connection;
use tracing::info;

/// Apply all schema definitions to the database.
pub fn apply(conn: &Connection) -> Result<()> {
    conn.execute_batch(ACCOUNT_SCHEMA)?;
    info!("Database schema applied");
    Ok(())
}

/// Account table schema.
const ACCOUNT_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS account (
    id                TEXT PRIMARY KEY,
    phone_number      TEXT NOT NULL CHECK(length(phone_number) >= 7 AND length(phone_number) <= 15),
    account_name      TEXT NOT NULL DEFAULT 'unknown',
    data_dir          TEXT NOT NULL,
    idle_timeout INTEGER NOT NULL DEFAULT 300,
    status            TEXT NOT NULL DEFAULT 'sleeping',
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_account_phone ON account(phone_number);
"#;
