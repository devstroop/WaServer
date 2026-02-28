//! Account Manager Service
//!
//! Manages multiple WhatsApp accounts with create/get/list/delete operations.
//! Account IDs are UUIDs, phone numbers are unique per account.

use super::account::WhatsAppAccount;
use crate::{
    config::AppConfig,
    models::account::{
        validate_phone_number, AccountId, AccountListResponse, AccountSetupConfig,
        CreateAccountRequest, CreateAccountResponse,
    },
    services::database::Database,
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Manages multiple WhatsApp accounts
pub struct AccountManager {
    /// Active account accounts (keyed by UUID)
    accounts: Arc<RwLock<HashMap<AccountId, Arc<WhatsAppAccount>>>>,
    /// Phone number to UUID mapping for lookups
    phone_to_id: Arc<RwLock<HashMap<String, AccountId>>>,
    /// Base directory for all account data
    base_dir: PathBuf,
    /// App configuration
    config: Arc<AppConfig>,
    /// Persistent database (SQLite)
    db: Database,
}

impl AccountManager {
    /// Create a new AccountManager
    pub fn new(config: Arc<AppConfig>, db: Database) -> Self {
        // Use configured base directory or default to ~/.was/accounts
        let base_dir = config
            .accounts
            .as_ref()
            .and_then(|ac| ac.base_directory.clone())
            .unwrap_or_else(|| {
                // Get home directory from environment
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home).join(".was").join("accounts")
            });

        info!("AccountManager initialized with base_dir: {:?}", base_dir);

        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            phone_to_id: Arc::new(RwLock::new(HashMap::new())),
            base_dir,
            config,
            db,
        }
    }

    /// Get base directory for accounts
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// Create a new account
    pub async fn create_account(
        &self,
        request: CreateAccountRequest,
    ) -> Result<CreateAccountResponse> {
        // Validate and normalize phone number
        let phone_number = validate_phone_number(&request.phone_number)
            .map_err(|e| anyhow!("Invalid phone number: {}", e))?;

        // Check phone uniqueness via database
        if self.db.get_account_by_phone(&phone_number)?.is_some() {
            return Err(anyhow!("Phone number '{}' already exists", phone_number));
        }

        // Generate new UUID for this account
        let account_id = Uuid::new_v4();

        // Use UUID as directory name
        let account_dir = self.base_dir.join(account_id.to_string());
        let account_name = request.account_name.clone();
        let idle_timeout = request.idle_timeout.unwrap_or(300);

        // Persist to database first
        self.db.create_account(
            &account_id.to_string(),
            &phone_number,
            &account_name,
            &account_dir.to_string_lossy(),
            idle_timeout,
        )?;

        // Create account config
        let setup_config = AccountSetupConfig {
            id: account_id,
            phone_number: Some(phone_number.clone()),
            account_name: Some(account_name.clone()),
            data_dir: account_dir.clone(),
            browser: request.browser.clone().unwrap_or_default(),
        };

        // Create the account
        let account = Arc::new(WhatsAppAccount::new(setup_config, self.config.clone()).await?);

        // Store in memory
        {
            let mut accounts = self.accounts.write().await;
            accounts.insert(account_id, account);
        }
        {
            let mut phone_map = self.phone_to_id.write().await;
            phone_map.insert(phone_number.clone(), account_id);
        }

        info!("Created account '{}' (phone: {})", account_id, phone_number);

        Ok(CreateAccountResponse {
            id: account_id,
            phone_number,
            account_name,
            status: "created".to_string(),
            data_directory: account_dir.to_string_lossy().to_string(),
            created_at: Utc::now().to_rfc3339(),
        })
    }

    /// Get an existing account by UUID or phone number
    pub async fn get_account(&self, id: &str) -> Option<Arc<WhatsAppAccount>> {
        // Try to parse as UUID first
        if let Ok(uuid) = Uuid::parse_str(id) {
            return self.accounts.read().await.get(&uuid).cloned();
        }

        // Try to look up by phone number
        let phone = validate_phone_number(id).unwrap_or_else(|_| id.to_string());

        let phone_map = self.phone_to_id.read().await;
        if let Some(uuid) = phone_map.get(&phone) {
            return self.accounts.read().await.get(uuid).cloned();
        }

        None
    }

    /// Get an account by UUID
    pub async fn get_account_by_id(&self, id: AccountId) -> Option<Arc<WhatsAppAccount>> {
        self.accounts.read().await.get(&id).cloned()
    }

    /// Get an account by phone number
    pub async fn get_account_by_phone(&self, phone: &str) -> Option<Arc<WhatsAppAccount>> {
        let phone = validate_phone_number(phone).unwrap_or_else(|_| phone.to_string());

        let phone_map = self.phone_to_id.read().await;
        if let Some(uuid) = phone_map.get(&phone) {
            return self.accounts.read().await.get(uuid).cloned();
        }
        None
    }

    /// Register a phone number for an account (called after WhatsApp authentication)
    pub async fn register_phone(&self, account_id: AccountId, phone: &str) -> Result<()> {
        let phone =
            validate_phone_number(phone).map_err(|e| anyhow!("Invalid phone number: {}", e))?;

        // Check for conflict: another account already has this phone
        {
            let phone_map = self.phone_to_id.read().await;
            if let Some(&existing_id) = phone_map.get(&phone) {
                if existing_id != account_id {
                    return Err(anyhow!(
                        "Phone '{}' is already registered to account '{}'",
                        phone,
                        existing_id
                    ));
                }
                // Already registered to this account — nothing to do
                return Ok(());
            }
        }

        // Register the mapping
        {
            let mut phone_map = self.phone_to_id.write().await;
            phone_map.insert(phone.clone(), account_id);
        }

        info!("Registered phone '{}' for account '{}'", phone, account_id);
        Ok(())
    }

    /// Get an account, returning an error if not found
    pub async fn get_account_or_error(&self, id: &str) -> Result<Arc<WhatsAppAccount>> {
        self.get_account(id)
            .await
            .ok_or_else(|| anyhow!("Account '{}' not found", id))
    }

    /// List all accounts
    pub async fn list_accounts(&self) -> AccountListResponse {
        let accounts = self.accounts.read().await;
        let mut account_infos = Vec::with_capacity(accounts.len());

        for account in accounts.values() {
            account_infos.push(account.info().await);
        }

        // Sort by ID for consistent ordering
        account_infos.sort_by(|a, b| a.id.cmp(&b.id));

        AccountListResponse {
            total: account_infos.len(),
            accounts: account_infos,
        }
    }

    /// Delete an account
    pub async fn delete_account(&self, id: &str, delete_data: bool) -> Result<AccountId> {
        // Find the account (by UUID or phone)
        let account = self
            .get_account(id)
            .await
            .ok_or_else(|| anyhow!("Account '{}' not found", id))?;

        let account_id = account.id;
        let phone_number = account.phone_number().map(|s| s.to_string());

        // Remove from database
        self.db.delete_account(&account_id.to_string())?;

        // Remove from accounts map
        {
            let mut accounts = self.accounts.write().await;
            accounts.remove(&account_id);
        }
        // Remove from phone map if phone was set
        if let Some(ref phone) = phone_number {
            let mut phone_map = self.phone_to_id.write().await;
            phone_map.remove(phone);
        }

        // Sleep the account first
        if let Err(e) = account.sleep().await {
            warn!(
                "Error sleeping account '{}' during deletion: {}",
                account_id, e
            );
        }

        if delete_data {
            // Directory is UUID-based
            let account_dir = self.base_dir.join(account_id.to_string());
            if account_dir.exists() {
                tokio::fs::remove_dir_all(&account_dir).await?;
                info!("Deleted account '{}' and all data", account_id);
            }
        } else {
            info!("Deleted account '{}' (data preserved)", account_id);
        }

        Ok(account_id)
    }

    /// Warmup an account's browser (on-demand)
    pub async fn warmup_account(&self, id: &str) -> Result<()> {
        let account = self.get_account_or_error(id).await?;
        account.warmup().await
    }

    /// Ensure an account is warm (auto-warms if sleeping)
    pub async fn ensure_account_warm(&self, id: &str) -> Result<Arc<WhatsAppAccount>> {
        let account = self.get_account_or_error(id).await?;
        account.ensure_warm().await?;
        Ok(account)
    }

    /// Reset an account — sleep browser and wipe all session data
    pub async fn reset_account(&self, id: &str) -> Result<()> {
        let account = self.get_account_or_error(id).await?;
        account.reset().await
    }

    /// Discover existing accounts from the database
    ///
    /// Returns `(newly_loaded, already_loaded)` — accounts just loaded
    /// and accounts that were already in memory.
    pub async fn discover_accounts(&self) -> Result<(Vec<AccountId>, Vec<AccountId>)> {
        let mut newly_loaded = Vec::new();
        let mut already_loaded = Vec::new();

        // Ensure base directory exists
        if !self.base_dir.exists() {
            tokio::fs::create_dir_all(&self.base_dir).await?;
        }

        let records = self.db.list_accounts()?;

        for record in records {
            let id_str = &record.id;

            let account_id: AccountId = match Uuid::parse_str(id_str) {
                Ok(uuid) => uuid,
                Err(e) => {
                    error!("Invalid UUID '{}' in database: {}", id_str, e);
                    continue;
                }
            };

            // Skip if already loaded
            if self.accounts.read().await.contains_key(&account_id) {
                already_loaded.push(account_id);
                continue;
            }

            let data_dir = PathBuf::from(&record.data_dir);

            let setup_config = AccountSetupConfig {
                id: account_id,
                phone_number: Some(record.phone_number.clone()),
                account_name: Some(record.account_name.clone()),
                data_dir: data_dir.clone(),
                browser: Default::default(),
            };

            match WhatsAppAccount::new(setup_config, self.config.clone()).await {
                Ok(account) => {
                    let account = Arc::new(account);
                    {
                        let mut accounts = self.accounts.write().await;
                        accounts.insert(account_id, account);
                    }
                    {
                        let mut phone_map = self.phone_to_id.write().await;
                        phone_map.insert(record.phone_number.clone(), account_id);
                    }
                    newly_loaded.push(account_id);
                    info!(
                        "Loaded account '{}' (phone: {}) from database",
                        account_id, record.phone_number
                    );
                }
                Err(e) => {
                    error!("Failed to load account '{}': {}", account_id, e);
                }
            }
        }

        if !newly_loaded.is_empty() {
            info!(
                "Discovered {} new accounts from database",
                newly_loaded.len()
            );
        }

        Ok((newly_loaded, already_loaded))
    }

    /// Get account count
    pub async fn count(&self) -> usize {
        self.accounts.read().await.len()
    }

    /// Check if an account exists (by UUID or phone)
    pub async fn exists(&self, id: &str) -> bool {
        self.get_account(id).await.is_some()
    }

    /// Shutdown all accounts (sleep all active browsers)
    pub async fn shutdown_all(&self) -> Vec<(AccountId, Result<()>)> {
        let accounts = self.accounts.read().await;
        let mut results = Vec::new();

        for (id, account) in accounts.iter() {
            let result = account.sleep().await;
            results.push((*id, result));
        }

        info!("Shutdown {} accounts", results.len());
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_phone_number() {
        // Valid phone numbers
        assert!(validate_phone_number("+1234567890").is_ok());
        assert!(validate_phone_number("1234567890").is_ok());
        assert!(validate_phone_number("+919876543210").is_ok());

        // Invalid phone numbers
        assert!(validate_phone_number("").is_err());
        assert!(validate_phone_number("123456").is_err()); // Too short
    }
}
