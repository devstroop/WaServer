//! Account Manager Service
//!
//! Manages multiple WhatsApp accounts with create/get/list/delete operations.
//! Account IDs are phone numbers in E.164 format.

use crate::{
    config::AppConfig,
    models::account::{
        validate_phone_number, phone_to_dir_name, AccountConfig, AccountId, AccountListResponse,
        AccountMetadata, CreateAccountRequest, CreateAccountResponse,
    },
};
use super::account::WhatsAppAccount;
use anyhow::{anyhow, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Account metadata filename
const METADATA_FILE: &str = "account.json";

/// Manages multiple WhatsApp accounts
pub struct AccountManager {
    /// Active account instances
    accounts: Arc<RwLock<HashMap<AccountId, Arc<WhatsAppAccount>>>>,
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
                PathBuf::from(home)
                    .join(".was")
                    .join("accounts")
            });

        info!("AccountManager initialized with base_dir: {:?}", base_dir);

        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            base_dir,
            config,
        }
    }

    /// Get base directory for accounts
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// Create a new account
    pub async fn create_account(&self, request: CreateAccountRequest) -> Result<CreateAccountResponse> {
        // Validate and normalize phone number
        let phone_number = validate_phone_number(&request.phone_number)
            .map_err(|e| anyhow!("Invalid phone number: {}", e))?;

        let accounts = self.accounts.read().await;
        if accounts.contains_key(&phone_number) {
            return Err(anyhow!("Account for phone '{}' already exists", phone_number));
        }
        drop(accounts);

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

        // Create account config
        let account_config = AccountConfig {
            id: phone_number.clone(),
            display_name: request.display_name.clone(),
            data_dir: account_dir.clone(),
            browser: request.browser.clone().unwrap_or_default(),
            auto_start: request.auto_start.unwrap_or(false),
        };

        // Create the account
        let account = Arc::new(WhatsAppAccount::new(account_config, self.config.clone()).await?);

        // Store in memory
        let mut accounts = self.accounts.write().await;
        accounts.insert(phone_number.clone(), account);

        info!("Created account for phone '{}'", phone_number);

        Ok(CreateAccountResponse {
            id: phone_number,
            status: "created".to_string(),
            data_directory: account_dir.to_string_lossy().to_string(),
            created_at: Utc::now().to_rfc3339(),
        })
    }

    /// Get an existing account
    pub async fn get_account(&self, id: &str) -> Option<Arc<WhatsAppAccount>> {
        // Try to normalize phone number for lookup
        let phone_id = validate_phone_number(id)
            .unwrap_or_else(|_| id.to_string());
        self.accounts.read().await.get(&phone_id).cloned()
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
    pub async fn delete_account(&self, id: &str, delete_data: bool) -> Result<()> {
        // Normalize phone number for lookup
        let phone_id = validate_phone_number(id)
            .unwrap_or_else(|_| id.to_string());
            
        let mut accounts = self.accounts.write().await;

        if let Some(account) = accounts.remove(&phone_id) {
            // Stop the account first
            if let Err(e) = account.stop().await {
                warn!("Error stopping account '{}' during deletion: {}", phone_id, e);
            }

            if delete_data {
                let dir_name = phone_to_dir_name(&phone_id);
                let account_dir = self.base_dir.join(&dir_name);
                if account_dir.exists() {
                    tokio::fs::remove_dir_all(&account_dir).await?;
                    info!("Deleted account '{}' and all data", phone_id);
                }
            } else {
                info!("Deleted account '{}' (data preserved)", phone_id);
            }

            Ok(())
        } else {
            Err(anyhow!("Account '{}' not found", id))
        }
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

            // Load metadata to get phone number (account ID)
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

            let account_id = metadata.id.clone();

            // Skip if already loaded
            if self.accounts.read().await.contains_key(&account_id) {
                continue;
            }

            // Load the account
            match self.load_account_from_dir(&account_id, &path).await {
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
    async fn load_account_from_dir(&self, phone_id: &str, data_dir: &PathBuf) -> Result<()> {
        // Read metadata
        let metadata_path = data_dir.join(METADATA_FILE);
        let content = tokio::fs::read_to_string(&metadata_path).await?;
        let metadata: AccountMetadata = serde_json::from_str(&content)?;

        // Create config from metadata
        let account_config = AccountConfig {
            id: phone_id.to_string(),
            display_name: metadata.display_name,
            data_dir: data_dir.clone(),
            browser: Default::default(),
            auto_start: false,
        };

        // Create account instance
        let account = Arc::new(WhatsAppAccount::new(account_config, self.config.clone()).await?);

        // Store
        let mut accounts = self.accounts.write().await;
        accounts.insert(phone_id.to_string(), account);

        info!("Loaded account '{}' from {:?}", phone_id, data_dir);
        Ok(())
    }

    /// Get account count
    pub async fn count(&self) -> usize {
        self.accounts.read().await.len()
    }

    /// Check if an account exists
    pub async fn exists(&self, id: &str) -> bool {
        let phone_id = validate_phone_number(id)
            .unwrap_or_else(|_| id.to_string());
        self.accounts.read().await.contains_key(&phone_id)
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
            results.push((id.clone(), result));
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
