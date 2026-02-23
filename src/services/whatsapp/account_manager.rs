//! Account Manager Service
//!
//! Manages multiple WhatsApp accounts with create/get/list/delete operations.
//! Account IDs are UUIDs, phone numbers are unique per account.

use super::account::WhatsAppAccount;
use crate::{
    config::AppConfig,
    models::account::{
        phone_to_dir_name, validate_phone_number, AccountConfig, AccountId, AccountListResponse,
        AccountMetadata, CreateAccountRequest, CreateAccountResponse,
    },
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Account metadata filename
const METADATA_FILE: &str = "account.json";

/// Manages multiple WhatsApp accounts
pub struct AccountManager {
    /// Active account instances (keyed by UUID)
    accounts: Arc<RwLock<HashMap<AccountId, Arc<WhatsAppAccount>>>>,
    /// Phone number to UUID mapping for lookups
    phone_to_id: Arc<RwLock<HashMap<String, AccountId>>>,
    /// Base directory for all account data
    base_dir: PathBuf,
    /// App configuration
    config: Arc<AppConfig>,
}

impl AccountManager {
    /// Create a new AccountManager
    pub fn new(config: Arc<AppConfig>) -> Self {
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

        // Check if phone number is already used
        {
            let phone_map = self.phone_to_id.read().await;
            if phone_map.contains_key(&phone_number) {
                return Err(anyhow!(
                    "Account for phone '{}' already exists",
                    phone_number
                ));
            }
        }

        // Use phone digits as directory name
        let dir_name = phone_to_dir_name(&phone_number);
        let account_dir = self.base_dir.join(&dir_name);

        // Check if directory already exists (account was previously created)
        if account_dir.exists() {
            // Check for metadata file
            let metadata_path = account_dir.join(METADATA_FILE);
            if metadata_path.exists() {
                return Err(anyhow!(
                    "Account directory for phone '{}' already exists. Use discover_accounts() to load it.",
                    phone_number
                ));
            }
        }

        // Generate new UUID for this account
        let account_id = Uuid::new_v4();

        // Create account config
        let account_config = AccountConfig {
            id: account_id,
            phone_number: phone_number.clone(),
            display_name: request.display_name.clone(),
            data_dir: account_dir.clone(),
            browser: request.browser.clone().unwrap_or_default(),
            auto_start: request.auto_start.unwrap_or(false),
        };

        // Create the account
        let account = Arc::new(WhatsAppAccount::new(account_config, self.config.clone()).await?);

        // Store in memory
        {
            let mut accounts = self.accounts.write().await;
            accounts.insert(account_id, account);
        }
        {
            let mut phone_map = self.phone_to_id.write().await;
            phone_map.insert(phone_number.clone(), account_id);
        }

        info!(
            "Created account '{}' for phone '{}'",
            account_id, phone_number
        );

        Ok(CreateAccountResponse {
            id: account_id,
            phone_number,
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
        let phone_number = account.phone_number.clone();

        // Remove from both maps
        {
            let mut accounts = self.accounts.write().await;
            accounts.remove(&account_id);
        }
        {
            let mut phone_map = self.phone_to_id.write().await;
            phone_map.remove(&phone_number);
        }

        // Stop the account first
        if let Err(e) = account.stop().await {
            warn!(
                "Error stopping account '{}' during deletion: {}",
                account_id, e
            );
        }

        if delete_data {
            let dir_name = phone_to_dir_name(&phone_number);
            let account_dir = self.base_dir.join(&dir_name);
            if account_dir.exists() {
                tokio::fs::remove_dir_all(&account_dir).await?;
                info!(
                    "Deleted account '{}' (phone: {}) and all data",
                    account_id, phone_number
                );
            }
        } else {
            info!(
                "Deleted account '{}' (phone: {}) (data preserved)",
                account_id, phone_number
            );
        }

        Ok(account_id)
    }

    /// Start an account's browser
    pub async fn start_account(&self, id: &str) -> Result<()> {
        let account = self.get_account_or_error(id).await?;
        account.start().await
    }

    /// Stop an account's browser
    pub async fn stop_account(&self, id: &str) -> Result<()> {
        let account = self.get_account_or_error(id).await?;
        account.stop().await
    }

    /// Discover existing accounts from filesystem
    pub async fn discover_accounts(&self) -> Result<Vec<AccountId>> {
        let mut discovered = Vec::new();

        // Ensure base directory exists
        if !self.base_dir.exists() {
            tokio::fs::create_dir_all(&self.base_dir).await?;
            return Ok(discovered);
        }

        let mut entries = tokio::fs::read_dir(&self.base_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let dir_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            // Check for metadata file
            let metadata_path = path.join(METADATA_FILE);
            if !metadata_path.exists() {
                debug!("Skipping directory '{}' - no metadata file", dir_name);
                continue;
            }

            // Load metadata to get account ID and phone number
            let content = match tokio::fs::read_to_string(&metadata_path).await {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to read metadata in '{}': {}", dir_name, e);
                    continue;
                }
            };

            let metadata: AccountMetadata = match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(e) => {
                    error!("Failed to parse metadata in '{}': {}", dir_name, e);
                    continue;
                }
            };

            let account_id = metadata.id;
            let phone_number = metadata.phone_number.clone();

            // Skip if already loaded
            if self.accounts.read().await.contains_key(&account_id) {
                continue;
            }

            // Load the account
            match self
                .load_account_from_dir(account_id, &phone_number, &path, &metadata)
                .await
            {
                Ok(()) => {
                    discovered.push(account_id);
                }
                Err(e) => {
                    error!("Failed to load account from '{}': {}", dir_name, e);
                }
            }
        }

        if !discovered.is_empty() {
            info!("Discovered {} existing accounts", discovered.len());
        }

        Ok(discovered)
    }

    /// Load an account from its data directory
    async fn load_account_from_dir(
        &self,
        account_id: AccountId,
        phone_number: &str,
        data_dir: &PathBuf,
        metadata: &AccountMetadata,
    ) -> Result<()> {
        // Create config from metadata
        let account_config = AccountConfig {
            id: account_id,
            phone_number: phone_number.to_string(),
            display_name: metadata.display_name.clone(),
            data_dir: data_dir.clone(),
            browser: Default::default(),
            auto_start: false,
        };

        // Create account instance
        let account = Arc::new(WhatsAppAccount::new(account_config, self.config.clone()).await?);

        // Store in both maps
        {
            let mut accounts = self.accounts.write().await;
            accounts.insert(account_id, account);
        }
        {
            let mut phone_map = self.phone_to_id.write().await;
            phone_map.insert(phone_number.to_string(), account_id);
        }

        info!(
            "Loaded account '{}' (phone: {}) from {:?}",
            account_id, phone_number, data_dir
        );
        Ok(())
    }

    /// Get account count
    pub async fn count(&self) -> usize {
        self.accounts.read().await.len()
    }

    /// Check if an account exists (by UUID or phone)
    pub async fn exists(&self, id: &str) -> bool {
        self.get_account(id).await.is_some()
    }

    /// Auto-start accounts that have auto_start enabled
    pub async fn auto_start_accounts(&self) -> Vec<(AccountId, Result<()>)> {
        let accounts = self.accounts.read().await;
        let results = Vec::new();

        for (id, account) in accounts.iter() {
            let info = account.info().await;
            // TODO: Check config for auto_start flag
            // For now, skip auto-start
            debug!("Account '{}' auto_start check (disabled for now)", id);
            let _ = info;
        }

        results
    }

    /// Shutdown all accounts
    pub async fn shutdown_all(&self) -> Vec<(AccountId, Result<()>)> {
        let accounts = self.accounts.read().await;
        let mut results = Vec::new();

        for (id, account) in accounts.iter() {
            let result = account.stop().await;
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
