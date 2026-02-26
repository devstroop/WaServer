//! Database Service
//!
//! Manages an embedded SurrealDB instance (file-based via SurrealKV).
//! Provides typed helpers for account CRUD operations.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use surrealdb::{
    engine::local::{Db, SurrealKv},
    Surreal,
};
use tracing::info;

use super::schema;

/// Persistent account record stored in SurrealDB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRecord {
    pub id: Option<surrealdb::sql::Thing>,
    pub phone_number: String,
    pub display_name: String,
    pub data_dir: String,
    pub auto_start: bool,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Wraps an embedded SurrealDB connection.
#[derive(Clone)]
pub struct Database {
    db: Surreal<Db>,
}

impl Database {
    /// Open (or create) a file-based SurrealDB at the given directory.
    pub async fn open(data_dir: &Path) -> Result<Self> {
        let db_path = data_dir.join("surreal");
        tokio::fs::create_dir_all(&db_path).await?;

        let db = Surreal::new::<SurrealKv>(db_path.to_str().unwrap())
            .await
            .map_err(|e| anyhow!("Failed to open SurrealDB: {}", e))?;

        db.use_ns("was")
            .use_db("was")
            .await
            .map_err(|e| anyhow!("Failed to select namespace/database: {}", e))?;

        info!("SurrealDB opened at {:?}", db_path);

        let instance = Self { db };
        schema::apply(&instance.db).await?;
        Ok(instance)
    }

    // ========================================================================
    // Account CRUD
    // ========================================================================

    /// Insert a new account. The record ID is the account UUID (string).
    pub async fn create_account(
        &self,
        id: &str,
        phone_number: &str,
        display_name: &str,
        data_dir: &str,
        auto_start: bool,
    ) -> Result<AccountRecord> {
        // Check uniqueness of phone_number explicitly for a clear error message
        let existing: Vec<AccountRecord> = self
            .db
            .query("SELECT * FROM account WHERE phone_number = $phone LIMIT 1")
            .bind(("phone", phone_number.to_string()))
            .await?
            .take(0)?;

        if !existing.is_empty() {
            return Err(anyhow!("Phone number '{}' already exists", phone_number));
        }

        let record: Option<AccountRecord> = self
            .db
            .create(("account", id))
            .content(serde_json::json!({
                "phone_number": phone_number,
                "display_name": display_name,
                "data_dir": data_dir,
                "auto_start": auto_start,
                "status": "stopped",
            }))
            .await
            .map_err(|e| anyhow!("Failed to create account: {}", e))?;

        record.ok_or_else(|| anyhow!("Account creation returned no record"))
    }

    /// Get an account by its UUID string id.
    pub async fn get_account(&self, id: &str) -> Result<Option<AccountRecord>> {
        let record: Option<AccountRecord> = self
            .db
            .select(("account", id))
            .await
            .map_err(|e| anyhow!("Failed to get account: {}", e))?;
        Ok(record)
    }

    /// Find an account by phone number.
    pub async fn get_account_by_phone(&self, phone_number: &str) -> Result<Option<AccountRecord>> {
        let mut results: Vec<AccountRecord> = self
            .db
            .query("SELECT * FROM account WHERE phone_number = $phone LIMIT 1")
            .bind(("phone", phone_number.to_string()))
            .await?
            .take(0)?;
        Ok(results.pop())
    }

    /// List all accounts.
    pub async fn list_accounts(&self) -> Result<Vec<AccountRecord>> {
        let records: Vec<AccountRecord> = self
            .db
            .select("account")
            .await
            .map_err(|e| anyhow!("Failed to list accounts: {}", e))?;
        Ok(records)
    }

    /// Update the status field of an account.
    pub async fn update_status(&self, id: &str, status: &str) -> Result<()> {
        let _: Option<AccountRecord> = self
            .db
            .update(("account", id))
            .merge(serde_json::json!({
                "status": status,
                "updated_at": chrono::Utc::now().to_rfc3339(),
            }))
            .await
            .map_err(|e| anyhow!("Failed to update status: {}", e))?;
        Ok(())
    }

    /// Update display_name for an account.
    pub async fn update_display_name(&self, id: &str, display_name: &str) -> Result<()> {
        let _: Option<AccountRecord> = self
            .db
            .update(("account", id))
            .merge(serde_json::json!({
                "display_name": display_name,
                "updated_at": chrono::Utc::now().to_rfc3339(),
            }))
            .await
            .map_err(|e| anyhow!("Failed to update display_name: {}", e))?;
        Ok(())
    }

    /// Delete an account record.
    pub async fn delete_account(&self, id: &str) -> Result<()> {
        let _: Option<AccountRecord> = self
            .db
            .delete(("account", id))
            .await
            .map_err(|e| anyhow!("Failed to delete account: {}", e))?;
        Ok(())
    }
}
