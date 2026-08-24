//! WhatsApp Instance Service
//!
//! Self-contained WhatsApp instance with isolated resources.
//! Each account has its own browser profile and session data.
//! Instance ID is a UUID, phone_number is the E.164 identifier.

use super::chat::{ChatService, ChatServiceTrait};
use crate::{
    browser::{BrowserService, BrowserServiceConfig},
    config::AppConfig,
    models::auth::AuthStatusResponse,
    models::instance::{
        InstanceBrowserConfig, InstanceConfig, InstanceId, InstanceInfo, InstanceMetadata,
        InstanceRateLimits, InstanceSetupConfig, InstanceStatus, UpdateInstanceConfigRequest,
    },
    services::auth::{AuthService, AuthServiceTrait},
    utils::metrics::{MetricsSnapshot, ServiceMetrics},
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tracing::{debug, error, info, warn};

/// Cached auth status with expiration
pub(super) struct CachedAuthStatus {
    pub(super) status: AuthStatusResponse,
    pub(super) cached_at: Instant,
}

/// Cache TTL for auth status (5 seconds)
pub(super) const AUTH_CACHE_TTL: Duration = Duration::from_secs(5);

/// Instance metadata filename
const METADATA_FILE: &str = "account.json";

/// Instance runtime config filename
const ACCOUNT_CONFIG_FILE: &str = "instance_config.json";

/// Self-contained WhatsApp instance with isolated resources
/// Each account is identified by UUID. Phone/instance_name live in metadata after WhatsApp login.
pub struct InstanceService {
    /// Instance identifier (UUID)
    pub id: InstanceId,
    /// Instance configuration
    pub(super) _config: InstanceSetupConfig,
    /// Global app config
    _app_config: Arc<AppConfig>,
    /// Data directory for this instance
    pub(super) data_dir: PathBuf,
    /// Instance metadata (includes bound phone)
    pub(super) metadata: Arc<RwLock<InstanceMetadata>>,
    /// Instance runtime configuration (API-managed)
    pub(super) instance_config: Arc<RwLock<InstanceConfig>>,
    /// Isolated browser service
    pub(super) browser_service: Arc<BrowserService>,
    /// Auth service
    pub(super) auth_service: Arc<dyn AuthServiceTrait>,
    /// Chat service
    chat_service: Arc<dyn ChatServiceTrait>,
    /// Current account status
    pub(super) status: Arc<RwLock<InstanceStatus>>,
    /// Operation semaphore for mutual exclusion
    operation_semaphore: Arc<Semaphore>,
    /// Service metrics
    pub(super) metrics: ServiceMetrics,
    /// Whether the instance is initialized
    pub(super) initialized: Arc<Mutex<bool>>,
    /// Cached auth status
    pub(super) auth_cache: Arc<Mutex<Option<CachedAuthStatus>>>,
    /// Timestamp of last activity (for idle auto-sleep)
    pub(super) last_activity: Arc<RwLock<Instant>>,
    /// Handle to the idle-sleep background task (cancelled on sleep/warmup)
    pub(super) idle_sleep_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl InstanceService {
    /// Create a new WhatsApp instance with isolated data directory
    pub async fn new(config: InstanceSetupConfig, app_config: Arc<AppConfig>) -> Result<Self> {
        let data_dir = config.data_dir.clone();

        info!("Creating account '{}' at {:?}", config.id, data_dir);

        // Ensure account directories exist
        tokio::fs::create_dir_all(&data_dir).await?;
        tokio::fs::create_dir_all(data_dir.join("chrome-profile")).await?;
        tokio::fs::create_dir_all(data_dir.join("sessions")).await?;
        tokio::fs::create_dir_all(data_dir.join("media")).await?;

        // Load or create account metadata
        let metadata = Self::load_or_create_metadata(&data_dir, &config).await?;

        // Load or create account runtime config
        let instance_config = Self::load_or_create_instance_config(&data_dir, &config).await?;

        // Create browser service with account-specific profile from account config
        let browser_config = BrowserServiceConfig::for_instance(
            &data_dir,
            instance_config.browser.headless,
            instance_config.browser.timeout_ms,
            instance_config.browser.extra_args.clone(),
        );
        let browser_service = Arc::new(BrowserService::new(browser_config));

        // Create auth and chat services
        let auth_service = Arc::new(AuthService::new(
            app_config.clone(),
            browser_service.clone(),
        ));
        let chat_service = Arc::new(ChatService::new(
            app_config.clone(),
            browser_service.clone(),
        ));

        // Register auth observer script — runs automatically after every WhatsApp page load
        browser_service
            .register_page_script(Self::AUTH_OBSERVER_JS)
            .await;

        Ok(Self {
            id: config.id,
            _config: config,
            _app_config: app_config,
            data_dir,
            metadata: Arc::new(RwLock::new(metadata)),
            instance_config: Arc::new(RwLock::new(instance_config)),
            browser_service,
            auth_service: auth_service as Arc<dyn AuthServiceTrait>,
            chat_service: chat_service as Arc<dyn ChatServiceTrait>,
            status: Arc::new(RwLock::new(InstanceStatus::Sleeping)),
            operation_semaphore: Arc::new(Semaphore::new(1)),
            metrics: ServiceMetrics::new(),
            initialized: Arc::new(Mutex::new(false)),
            auth_cache: Arc::new(Mutex::new(None)),
            last_activity: Arc::new(RwLock::new(Instant::now())),
            idle_sleep_handle: Arc::new(Mutex::new(None)),
        })
    }

    /// Load or create account metadata
    async fn load_or_create_metadata(
        data_dir: &Path,
        config: &InstanceSetupConfig,
    ) -> Result<InstanceMetadata> {
        let metadata_path = data_dir.join(METADATA_FILE);

        if metadata_path.exists() {
            let content = tokio::fs::read_to_string(&metadata_path).await?;
            let metadata: InstanceMetadata = serde_json::from_str(&content)
                .map_err(|e| anyhow!("Failed to parse account metadata: {}", e))?;
            debug!("Loaded metadata for account '{}'", config.id);
            Ok(metadata)
        } else {
            let metadata = InstanceMetadata::new(
                config.id,
                config.phone_number.clone(),
                config.instance_name.clone(),
            );
            Self::save_metadata_to_path(&metadata_path, &metadata).await?;
            info!("Created new metadata for account '{}'", config.id);
            Ok(metadata)
        }
    }

    /// Save metadata to disk
    async fn save_metadata(&self) -> Result<()> {
        let metadata = self.metadata.read().await;
        let metadata_path = self.data_dir.join(METADATA_FILE);
        Self::save_metadata_to_path(&metadata_path, &metadata).await
    }

    async fn save_metadata_to_path(path: &Path, metadata: &InstanceMetadata) -> Result<()> {
        let content = serde_json::to_string_pretty(metadata)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Load or create account runtime config
    async fn load_or_create_instance_config(
        data_dir: &Path,
        config: &InstanceSetupConfig,
    ) -> Result<InstanceConfig> {
        let config_path = data_dir.join(ACCOUNT_CONFIG_FILE);

        if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            let instance_config: InstanceConfig = serde_json::from_str(&content)
                .map_err(|e| anyhow!("Failed to parse account config: {}", e))?;
            debug!("Loaded account config for account '{}'", config.id);
            Ok(instance_config)
        } else {
            // Create default config from InstanceSetupConfig
            let instance_config = InstanceConfig {
                instance_id: Some(config.id),
                instance_name: config.instance_name.clone(),
                idle_timeout: 300,
                browser: InstanceBrowserConfig {
                    headless: config.browser.headless.unwrap_or(true),
                    timeout_ms: 30000,
                    extra_args: config.browser.extra_args.clone(),
                },
                rate_limits: InstanceRateLimits::default(),
            };
            Self::save_instance_config_to_path(&config_path, &instance_config).await?;
            info!("Created new account config for account '{}'", config.id);
            Ok(instance_config)
        }
    }

    async fn save_instance_config_to_path(path: &PathBuf, config: &InstanceConfig) -> Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Get the current account configuration
    pub async fn get_config(&self) -> InstanceConfig {
        let config = self.instance_config.read().await;
        let mut result = config.clone();
        // Always include instance_id in response
        result.instance_id = Some(self.id);
        result
    }

    /// Update account configuration with partial updates
    /// Delegates to `application::instance::config_validation::validated_apply_config_update`
    /// (#6) — typed validation errors, `restart_required` derived here.
    pub async fn update_config(
        &self,
        update: UpdateInstanceConfigRequest,
    ) -> Result<InstanceConfig> {
        use crate::application::instance::validated_apply_config_update;

        let mut config = self.instance_config.write().await;

        // Validate + apply via application layer (typed ConfigError)
        let (next, _restart_required) = validated_apply_config_update(&config, update)
            .map_err(|e| anyhow!("Invalid configuration: {}", e))?;

        // Ensure instance_id is set
        let mut next = next;
        next.instance_id = Some(self.id);

        // Save to disk
        let config_path = self.data_dir.join(ACCOUNT_CONFIG_FILE);
        Self::save_instance_config_to_path(&config_path, &next).await?;

        *config = next.clone();

        info!("Updated account config for account '{}'", self.id);
        Ok(next)
    }

    /// Update config returning typed error + restart flag (per #6 — used by handler for 400 vs 500)
    pub async fn update_config_typed(
        &self,
        update: UpdateInstanceConfigRequest,
    ) -> Result<(InstanceConfig, bool), crate::application::instance::ConfigError> {
        use crate::application::instance::validated_apply_config_update;

        let mut config = self.instance_config.write().await;
        let (next, restart_required) = validated_apply_config_update(&config, update)?;

        let mut next = next;
        next.instance_id = Some(self.id);

        let config_path = self.data_dir.join(ACCOUNT_CONFIG_FILE);
        Self::save_instance_config_to_path(&config_path, &next)
            .await
            .map_err(|_| {
                crate::application::instance::ConfigError::InvalidBrowserTimeout(
                    next.browser.timeout_ms,
                )
            })?;

        *config = next.clone();
        Ok((next, restart_required))
    }

    /// Get current account status
    pub async fn status(&self) -> InstanceStatus {
        self.status.read().await.clone()
    }

    /// Get phone number from metadata (None if not yet authenticated with WhatsApp)
    pub fn phone_number(&self) -> Option<String> {
        self.metadata
            .try_read()
            .ok()
            .and_then(|m| m.phone_number.clone())
    }

    /// Get account info
    pub async fn info(&self) -> InstanceInfo {
        let metadata = self.metadata.read().await;
        let status = self.status.read().await.clone();
        let browser_running = self.browser_service.is_running().await;

        // Check WhatsApp auth if browser is running
        let authorized = if browser_running {
            self.check_auth_status_directly().await.unwrap_or(false)
        } else {
            false
        };

        InstanceInfo {
            id: self.id,
            phone_number: metadata.phone_number.clone(),
            instance_name: metadata.instance_name.clone(),
            status,
            authorized,
            created_at: metadata.created_at,
        }
    }

    /// Called when WhatsApp Web authentication completes.
    /// Sets the phone number and display name from the authenticated session.
    pub async fn on_whatsapp_authenticated(&self, phone: &str) -> Result<()> {
        let auth_phone = crate::models::instance::validate_phone_number(phone)
            .map_err(|e| anyhow!("Invalid authenticated phone: {}", e))?;

        // Check if another phone was already bound
        let current_phone = self.metadata.read().await.phone_number.clone();
        if let Some(ref existing) = current_phone {
            if *existing != auth_phone {
                return Err(anyhow!(
                    "Instance is already bound to phone {}. Cannot rebind to {}.",
                    existing,
                    auth_phone
                ));
            }
        }

        // Update metadata
        let mut metadata = self.metadata.write().await;
        if metadata.phone_number.is_none() {
            metadata.phone_number = Some(auth_phone.clone());
        }
        if metadata.first_linked_at.is_none() {
            metadata.first_linked_at = Some(Utc::now());
        }
        drop(metadata);
        self.save_metadata().await?;

        info!(
            "Instance '{}' authenticated with phone '{}'",
            self.id, auth_phone
        );
        Ok(())
    }

    // === Service Access ===

    /// Get reference to auth service
    pub fn auth_service(&self) -> &Arc<dyn AuthServiceTrait> {
        &self.auth_service
    }

    /// Get reference to chat service
    pub fn chat_service(&self) -> &Arc<dyn ChatServiceTrait> {
        &self.chat_service
    }

    /// Get reference to browser service
    pub fn browser_service(&self) -> &Arc<BrowserService> {
        &self.browser_service
    }

    // === Operations ===

    /// Check if the instance is currently busy
    pub async fn is_busy(&self) -> bool {
        self.operation_semaphore.available_permits() == 0
    }

    /// Default operation timeout (30 seconds)
    const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_secs(30);

    /// Execute an operation with exclusive access and timeout protection
    ///
    /// This ensures the API layer never hangs indefinitely waiting for browser operations.
    /// If the operation takes longer than the timeout, it returns an error immediately.
    pub async fn execute_with_busy_flag<F, T>(&self, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        self.execute_with_timeout(operation, Self::DEFAULT_OPERATION_TIMEOUT)
            .await
    }

    /// Execute an operation with custom timeout
    pub async fn execute_with_timeout<F, T>(&self, operation: F, timeout: Duration) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        let permit = match self.operation_semaphore.try_acquire() {
            Ok(permit) => permit,
            Err(_) => {
                return Err(anyhow!(
                    "Instance '{}' is busy with another operation",
                    self.id
                ));
            }
        };

        debug!(
            "Instance '{}' - operation started (timeout: {:?})",
            self.id, timeout
        );

        // Wrap the operation with a timeout to prevent indefinite hangs
        let result = match tokio::time::timeout(timeout, operation).await {
            Ok(result) => result,
            Err(_) => {
                error!(
                    "Instance '{}' - operation timed out after {:?}",
                    self.id, timeout
                );
                Err(anyhow!(
                    "Operation timed out after {:?}. Browser may be unresponsive.",
                    timeout
                ))
            }
        };

        drop(permit);
        debug!("Instance '{}' - operation completed", self.id);

        result
    }

    /// Quick browser health check with short timeout (2 seconds)
    ///
    /// This is used to verify the browser is responsive before starting operations.
    /// Returns false if browser is dead/unresponsive, allowing fast failure.
    pub async fn is_browser_responsive(&self) -> bool {
        if !self.browser_service.is_running().await {
            return false;
        }

        // Quick ping with 2 second timeout
        let check = async {
            match self.browser_service.get_whatsapp_page().await {
                Ok(page) => {
                    // Try a simple operation to verify page is responsive
                    page.evaluate("1 + 1").await.is_ok()
                }
                Err(_) => false,
            }
        };

        match tokio::time::timeout(Duration::from_secs(2), check).await {
            Ok(result) => result,
            Err(_) => {
                warn!("Instance '{}' - browser health check timed out", self.id);
                false
            }
        }
    }

    /// Check if account is initialized
    pub async fn is_initialized(&self) -> bool {
        *self.initialized.lock().await
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        if !self.is_initialized().await {
            return Err(anyhow!("Instance '{}' not initialized", self.id));
        }

        match self.browser_service.get_whatsapp_page().await {
            Ok(_) => Ok(()),
            Err(e) => {
                self.metrics.increment_error_count();
                Err(anyhow!(
                    "Browser unhealthy for account '{}': {}",
                    self.id,
                    e
                ))
            }
        }
    }

    /// Get metrics snapshot
    pub fn get_metrics(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Track message sent
    pub fn track_message_sent(&self) {
        self.metrics.increment_messages_sent();
    }

    /// Track error
    pub fn track_error(&self) {
        self.metrics.increment_error_count();
    }

    /// Pre-check to dismiss dialogs
    pub async fn pre_check(&self) -> Result<()> {
        let page = self.browser_service.get_whatsapp_page().await?;

        if let Ok(_dialog) = page.find_element("[role='dialog']").await {
            if let Ok(backdrop) = page
                .find_element("div[data-animate-modal-backdrop='true']")
                .await
            {
                debug!("Instance '{}' - dismissing dialog", self.id);
                backdrop.click().await?;

                tokio::time::timeout(std::time::Duration::from_millis(10000), async {
                    while page.find_element("[role='dialog']").await.is_ok() {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                })
                .await
                .map_err(|_| anyhow!("Timeout waiting for dialog to disappear"))?;
            }
        }

        Ok(())
    }

    /// Wait for loading to complete
    pub async fn wait_til_loading(&self) -> Result<()> {
        let page = self.browser_service.get_whatsapp_page().await?;

        tokio::time::timeout(std::time::Duration::from_millis(10000), async {
            while page.find_element("progress[max='100']").await.is_ok() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        })
        .await
        .map_err(|_| anyhow!("Timeout waiting for loading to complete"))?;

        Ok(())
    }
}
