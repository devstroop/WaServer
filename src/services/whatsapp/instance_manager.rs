//! Instance Manager Service
//!
//! Manages multiple WhatsApp instances with create/get/list/delete operations.
//! Instance IDs are UUIDs, phone numbers are unique per instance.

use super::instance::InstanceService;
use crate::{
    config::AppConfig,
    models::instance::{
        validate_phone_number, CreateInstanceRequest, CreateInstanceResponse, InstanceId,
        InstanceListResponse, InstanceSetupConfig,
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

/// Manages multiple WhatsApp instances
///
/// Facade over `application::instance::InstanceRegistry` (#5) — the registry owns
/// metadata/config/phone-index state; this struct keeps `Arc<InstanceService>`
/// handles (browser lifecycle) and delegates persistence to `SqliteInstanceStore`.
pub struct InstanceManager {
    /// Active instance services (keyed by UUID)
    instances: Arc<RwLock<HashMap<InstanceId, Arc<InstanceService>>>>,
    /// Phone number to UUID mapping for lookups
    phone_to_id: Arc<RwLock<HashMap<String, InstanceId>>>,
    /// Application registry — single source for metadata + config + phone index (#5)
    pub registry: Arc<crate::application::instance::InstanceRegistry>,
    /// SQLite store adapter implementing InstanceStore port (#5)
    pub store: Arc<crate::infrastructure::persistence::SqliteInstanceStore>,
    /// Base directory for all account data
    base_dir: PathBuf,
    /// App configuration
    config: Arc<AppConfig>,
    /// Persistent database (SQLite)
    db: Database,
    /// Per-instance metrics registry (#6 observability)
    pub observability: Arc<crate::shared::observability::instance_metrics::InstanceMetricsRegistry>,
    /// Shared send rate limiter — one instance for the whole process (#7)
    pub rate_limiter: Arc<dyn crate::application::messaging::ports::RateLimitPort + Send + Sync>,
}

impl InstanceManager {
    /// Create a new InstanceManager
    pub fn new(config: Arc<AppConfig>, db: Database) -> Self {
        // Use configured base directory or default to ~/.was/accounts
        let base_dir = config
            .instances
            .as_ref()
            .and_then(|ac| ac.base_directory.clone())
            .unwrap_or_else(|| {
                // Get home directory from environment
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home).join(".was").join("accounts")
            });

        info!("InstanceManager initialized with base_dir: {:?}", base_dir);

        let registry = Arc::new(crate::application::instance::InstanceRegistry::new());
        let rate_limiter = Arc::new(
            crate::infrastructure::messaging::InMemoryRateLimiter::configured(
                Arc::new(crate::infrastructure::messaging::RegistryRateLimits(
                    registry.clone(),
                )),
                60,
                1000,
            ),
        );

        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            phone_to_id: Arc::new(RwLock::new(HashMap::new())),
            registry,
            store: Arc::new(crate::infrastructure::persistence::SqliteInstanceStore(
                db.clone(),
            )),
            base_dir,
            config,
            db,
            observability: Arc::new(
                crate::shared::observability::instance_metrics::InstanceMetricsRegistry::new(),
            ),
            rate_limiter,
        }
    }

    /// Get base directory for accounts
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// Create a new account
    ///
    /// Delegates persistence to `SqliteInstanceStore::create_instance_row` and
    /// state to `InstanceRegistry::register` (#5). Phone conflict → typed error.
    pub async fn create_instance(
        &self,
        request: CreateInstanceRequest,
    ) -> Result<CreateInstanceResponse> {
        // Validate and normalize phone number
        let phone_number = validate_phone_number(&request.phone_number)
            .map_err(|e| anyhow!("Invalid phone number: {}", e))?;

        // Check phone uniqueness via store (DB is source of truth)
        if self.db.get_instance_by_phone(&phone_number)?.is_some() {
            return Err(anyhow!("Phone number '{}' already exists", phone_number));
        }

        // Generate new UUID for this instance
        let instance_id = Uuid::new_v4();

        // Use UUID as directory name
        let instance_dir = self.base_dir.join(instance_id.to_string());
        let instance_name = request.instance_name.clone();
        let idle_timeout = request.idle_timeout.unwrap_or(300);

        // Persist to database via store adapter (#5)
        self.store
            .create_instance_row(
                instance_id,
                &phone_number,
                &instance_name,
                instance_dir.clone(),
                idle_timeout,
            )
            .await?;

        // Create account config
        let setup_config = InstanceSetupConfig {
            id: instance_id,
            phone_number: Some(phone_number.clone()),
            instance_name: Some(instance_name.clone()),
            data_dir: instance_dir.clone(),
            browser: request.browser.clone().unwrap_or_default(),
        };

        // Create the instance
        let account = Arc::new(InstanceService::new(setup_config, self.config.clone()).await?);

        // Store in memory
        {
            let mut accounts = self.instances.write().await;
            accounts.insert(instance_id, account);
        }
        {
            let mut phone_map = self.phone_to_id.write().await;
            phone_map.insert(phone_number.clone(), instance_id);
        }

        // Register in application registry (metadata + config + phone index) (#5)
        let mut metadata = crate::domain::instance::InstanceMetadata::new(
            instance_id,
            Some(phone_number.clone()),
            Some(instance_name.clone()),
        );
        metadata.phone_number = Some(phone_number.clone());
        let config = crate::domain::instance::InstanceConfig {
            instance_id: Some(instance_id),
            instance_name: Some(instance_name.clone()),
            idle_timeout,
            browser: Default::default(),
            rate_limits: Default::default(),
        };
        if let Err(e) = self.registry.register(metadata, config).await {
            warn!("Registry register failed for '{}': {}", instance_id, e);
        }
        // Track warmup metric for new instances (#6)
        let _ = self.observability.for_instance(instance_id).await;

        info!(
            "Created account '{}' (phone: {})",
            instance_id, phone_number
        );

        Ok(CreateInstanceResponse {
            id: instance_id,
            phone_number,
            instance_name,
            status: "created".to_string(),
            data_directory: instance_dir.to_string_lossy().to_string(),
            created_at: Utc::now().to_rfc3339(),
        })
    }

    /// Get an existing instance by UUID or phone number
    pub async fn get_instance(&self, id: &str) -> Option<Arc<InstanceService>> {
        // Try to parse as UUID first
        if let Ok(uuid) = Uuid::parse_str(id) {
            return self.instances.read().await.get(&uuid).cloned();
        }

        // Try to look up by phone number
        let phone = validate_phone_number(id).unwrap_or_else(|_| id.to_string());

        let phone_map = self.phone_to_id.read().await;
        if let Some(uuid) = phone_map.get(&phone) {
            return self.instances.read().await.get(uuid).cloned();
        }

        None
    }

    /// Get an instance by UUID
    pub async fn get_instance_by_id(&self, id: InstanceId) -> Option<Arc<InstanceService>> {
        self.instances.read().await.get(&id).cloned()
    }

    /// Get an instance by phone number
    pub async fn get_instance_by_phone(&self, phone: &str) -> Option<Arc<InstanceService>> {
        let phone = validate_phone_number(phone).unwrap_or_else(|_| phone.to_string());

        let phone_map = self.phone_to_id.read().await;
        if let Some(uuid) = phone_map.get(&phone) {
            return self.instances.read().await.get(uuid).cloned();
        }
        None
    }

    /// Register a phone number for an instance (called after WhatsApp authentication)
    pub async fn register_phone(&self, instance_id: InstanceId, phone: &str) -> Result<()> {
        let phone =
            validate_phone_number(phone).map_err(|e| anyhow!("Invalid phone number: {}", e))?;

        // Check for conflict: another account already has this phone
        {
            let phone_map = self.phone_to_id.read().await;
            if let Some(&existing_id) = phone_map.get(&phone) {
                if existing_id != instance_id {
                    return Err(anyhow!(
                        "Phone '{}' is already registered to account '{}'",
                        phone,
                        existing_id
                    ));
                }
                // Already registered to this instance — nothing to do
                return Ok(());
            }
        }

        // Register the mapping
        {
            let mut phone_map = self.phone_to_id.write().await;
            phone_map.insert(phone.clone(), instance_id);
        }

        info!("Registered phone '{}' for account '{}'", phone, instance_id);
        Ok(())
    }

    /// Get an instance, returning an error if not found
    pub async fn get_instance_or_error(&self, id: &str) -> Result<Arc<InstanceService>> {
        self.get_instance(id)
            .await
            .ok_or_else(|| anyhow!("Instance '{}' not found", id))
    }

    /// List all accounts
    pub async fn list_instances(&self) -> InstanceListResponse {
        let accounts = self.instances.read().await;
        let mut account_infos = Vec::with_capacity(accounts.len());

        for account in accounts.values() {
            account_infos.push(account.info().await);
        }

        // Sort by ID for consistent ordering
        account_infos.sort_by_key(|a| a.id);

        InstanceListResponse {
            total: account_infos.len(),
            instances: account_infos,
        }
    }

    /// Delete an instance
    ///
    /// Delegates DB removal to `SqliteInstanceStore::delete_instance_row` and
    /// registry cleanup via `InstanceRegistry::remove` (#5).
    pub async fn delete_instance(&self, id: &str, delete_data: bool) -> Result<InstanceId> {
        // Find the instance (by UUID or phone)
        let account = self
            .get_instance(id)
            .await
            .ok_or_else(|| anyhow!("Instance '{}' not found", id))?;

        let instance_id = account.id;
        let phone_number = account.phone_number().map(|s| s.to_string());

        // Remove from database via store adapter (#5)
        self.store.delete_instance_row(instance_id).await?;

        // Remove from registry (metadata + config + phone index) (#5)
        if self.registry.remove(instance_id).await.is_none() {
            warn!(
                "Registry remove: instance '{}' was not registered",
                instance_id
            );
        }
        // Remove observability counters (#6)
        self.observability.remove_instance(instance_id).await;

        // Remove from accounts map
        {
            let mut accounts = self.instances.write().await;
            accounts.remove(&instance_id);
        }
        // Remove from phone map if phone was set
        if let Some(ref phone) = phone_number {
            let mut phone_map = self.phone_to_id.write().await;
            phone_map.remove(phone);
        }

        // Sleep the instance first
        if let Err(e) = account.sleep().await {
            warn!(
                "Error sleeping account '{}' during deletion: {}",
                instance_id, e
            );
        }

        if delete_data {
            // Directory is UUID-based
            let instance_dir = self.base_dir.join(instance_id.to_string());
            if instance_dir.exists() {
                tokio::fs::remove_dir_all(&instance_dir).await?;
                info!("Deleted account '{}' and all data", instance_id);
            }
        } else {
            info!("Deleted account '{}' (data preserved)", instance_id);
        }

        Ok(instance_id)
    }

    /// Warmup an instance's browser (on-demand)
    pub async fn warmup_account(&self, id: &str) -> Result<()> {
        let account = self.get_instance_or_error(id).await?;
        account.warmup().await
    }

    /// Ensure an instance is warm (auto-warms if sleeping)
    pub async fn ensure_account_warm(&self, id: &str) -> Result<Arc<InstanceService>> {
        let account = self.get_instance_or_error(id).await?;
        account.ensure_warm().await?;
        Ok(account)
    }

    /// Reset an instance — sleep browser and wipe all session data
    pub async fn reset_account(&self, id: &str) -> Result<()> {
        let account = self.get_instance_or_error(id).await?;
        account.reset().await
    }

    /// Discover existing instances from the database
    ///
    /// Returns `(newly_loaded, already_loaded)` — accounts just loaded
    /// and accounts that were already in memory.
    pub async fn discover_instances(&self) -> Result<(Vec<InstanceId>, Vec<InstanceId>)> {
        let mut newly_loaded = Vec::new();
        let mut already_loaded = Vec::new();

        // Ensure base directory exists
        if !self.base_dir.exists() {
            tokio::fs::create_dir_all(&self.base_dir).await?;
        }

        let records = self.db.list_instances()?;

        for record in records {
            let id_str = &record.id;

            let instance_id: InstanceId = match Uuid::parse_str(id_str) {
                Ok(uuid) => uuid,
                Err(e) => {
                    error!("Invalid UUID '{}' in database: {}", id_str, e);
                    continue;
                }
            };

            // Skip if already loaded
            if self.instances.read().await.contains_key(&instance_id) {
                already_loaded.push(instance_id);
                continue;
            }

            let data_dir = PathBuf::from(&record.data_dir);

            let setup_config = InstanceSetupConfig {
                id: instance_id,
                phone_number: Some(record.phone_number.clone()),
                instance_name: Some(record.instance_name.clone()),
                data_dir: data_dir.clone(),
                browser: Default::default(),
            };

            match InstanceService::new(setup_config, self.config.clone()).await {
                Ok(account) => {
                    let account = Arc::new(account);
                    {
                        let mut accounts = self.instances.write().await;
                        accounts.insert(instance_id, account);
                    }
                    {
                        let mut phone_map = self.phone_to_id.write().await;
                        phone_map.insert(record.phone_number.clone(), instance_id);
                    }
                    // Register in application registry (#5)
                    let mut metadata = crate::domain::instance::InstanceMetadata::new(
                        instance_id,
                        Some(record.phone_number.clone()),
                        Some(record.instance_name.clone()),
                    );
                    metadata.phone_number = Some(record.phone_number.clone());
                    if let Err(e) = self.registry.register(metadata, Default::default()).await {
                        warn!("Registry register failed for '{}': {}", instance_id, e);
                    }
                    newly_loaded.push(instance_id);
                    info!(
                        "Loaded account '{}' (phone: {}) from database",
                        instance_id, record.phone_number
                    );
                }
                Err(e) => {
                    // #6: log with context but continue discovery (no silent swallow)
                    error!(
                        "Failed to load account '{}' (phone: {}): {}",
                        instance_id, record.phone_number, e
                    );
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
        self.instances.read().await.len()
    }

    /// Check if an instance exists (by UUID or phone)
    pub async fn exists(&self, id: &str) -> bool {
        self.get_instance(id).await.is_some()
    }

    /// Shutdown all accounts (sleep all active browsers)
    pub async fn shutdown_all(&self) -> Vec<(InstanceId, Result<()>)> {
        let accounts = self.instances.read().await;
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
