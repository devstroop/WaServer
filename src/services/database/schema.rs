//! Database Schema Definitions
//!
//! SurrealDB schema for the `account` table.
//! Called once at startup to ensure tables and indexes exist.

use anyhow::Result;
use surrealdb::{engine::local::Db, Surreal};
use tracing::info;

/// Apply all schema definitions to the database.
pub async fn apply(db: &Surreal<Db>) -> Result<()> {
    db.query(ACCOUNT_SCHEMA).await?.check()?;
    info!("Database schema applied");
    Ok(())
}

/// Account table schema.
///
/// Fields:
///   - phone_number : string (unique, mandatory)
///   - display_name : string (default "unknown")
///   - data_dir     : string
///   - auto_start   : bool
///   - status       : string  (stopped | starting | running | error)
///   - created_at   : datetime
///   - updated_at   : datetime
const ACCOUNT_SCHEMA: &str = r#"
DEFINE TABLE IF NOT EXISTS account SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS phone_number ON account TYPE string
    ASSERT string::len($value) >= 7 AND string::len($value) <= 15;
DEFINE FIELD IF NOT EXISTS display_name ON account TYPE string
    DEFAULT "unknown";
DEFINE FIELD IF NOT EXISTS data_dir     ON account TYPE string;
DEFINE FIELD IF NOT EXISTS auto_start   ON account TYPE bool
    DEFAULT false;
DEFINE FIELD IF NOT EXISTS status       ON account TYPE string
    DEFAULT "stopped";
DEFINE FIELD IF NOT EXISTS created_at   ON account TYPE datetime
    DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at   ON account TYPE datetime
    DEFAULT time::now();

DEFINE INDEX IF NOT EXISTS idx_account_phone ON account FIELDS phone_number UNIQUE;
"#;
