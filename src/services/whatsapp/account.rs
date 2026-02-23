//! WhatsApp Account Service
//!
//! Self-contained WhatsApp account with isolated resources.
//! Each account has its own browser profile, database, and session data.
//! Account ID is a UUID, phone_number is the E.164 identifier.

use super::chat::{ChatService, ChatServiceTrait};
use crate::{
    browser::{BrowserService, BrowserServiceConfig},
    config::AppConfig,
    models::account::{
        AccountSetupConfig, AccountId, AccountInfo, AccountMetadata, AccountStatus,
        AccountBrowserConfig, AccountConfig, AccountRateLimits, AccountWebhookConfig,
        UpdateAccountConfigRequest,
    },
    models::auth::AuthStatusResponse,
    services::{
        auth::{AuthService, AuthServiceTrait},
        database::DatabaseService,
        webhook::{WebhookEvent, WebhookMessageData, WebhookService},
    },
    utils::metrics::{MetricsSnapshot, ServiceMetrics},
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tracing::{debug, error, info, warn};

/// Cached auth status with expiration
struct CachedAuthStatus {
    status: AuthStatusResponse,
    cached_at: Instant,
}

/// Cache TTL for auth status (5 seconds)
const AUTH_CACHE_TTL: Duration = Duration::from_secs(5);

/// Account metadata filename
const METADATA_FILE: &str = "account.json";

/// Account runtime config filename
const ACCOUNT_CONFIG_FILE: &str = "account_config.json";

/// Self-contained WhatsApp account with isolated resources
/// Each account is identified by UUID. Phone/display_name live in metadata after WhatsApp login.
pub struct WhatsAppAccount {
    /// Account identifier (UUID)
    pub id: AccountId,
    /// Account configuration
    _config: AccountSetupConfig,
    /// Global app config
    _app_config: Arc<AppConfig>,
    /// Data directory for this account
    data_dir: PathBuf,
    /// Account metadata (includes bound phone)
    metadata: Arc<RwLock<AccountMetadata>>,
    /// Account runtime configuration (API-managed)
    account_config: Arc<RwLock<AccountConfig>>,
    /// Isolated browser service
    browser_service: Arc<BrowserService>,
    /// Auth service
    auth_service: Arc<dyn AuthServiceTrait>,
    /// Chat service
    chat_service: Arc<dyn ChatServiceTrait>,
    /// Database service
    database: Arc<DatabaseService>,
    /// Webhook service
    webhook_service: Arc<WebhookService>,
    /// Current account status
    status: Arc<RwLock<AccountStatus>>,
    /// Operation semaphore for mutual exclusion
    operation_semaphore: Arc<Semaphore>,
    /// Service metrics
    metrics: ServiceMetrics,
    /// Whether the account is initialized
    initialized: Arc<Mutex<bool>>,
    /// Cached auth status
    auth_cache: Arc<Mutex<Option<CachedAuthStatus>>>,
}

impl WhatsAppAccount {
    /// Create a new WhatsApp account with isolated data directory
    pub async fn new(config: AccountSetupConfig, app_config: Arc<AppConfig>) -> Result<Self> {
        let data_dir = config.data_dir.clone();

        info!(
            "Creating account '{}' at {:?}",
            config.id, data_dir
        );

        // Ensure account directories exist
        tokio::fs::create_dir_all(&data_dir).await?;
        tokio::fs::create_dir_all(data_dir.join("chrome-profile")).await?;
        tokio::fs::create_dir_all(data_dir.join("sessions")).await?;
        tokio::fs::create_dir_all(data_dir.join("media")).await?;

        // Load or create account metadata
        let metadata = Self::load_or_create_metadata(&data_dir, &config).await?;

        // Load or create account runtime config
        let account_config = Self::load_or_create_account_config(&data_dir, &config).await?;

        // Create browser service with account-specific profile from account config
        let browser_config = BrowserServiceConfig::for_account(
            &data_dir,
            account_config.browser.headless,
            account_config.browser.timeout_ms,
            account_config.browser.extra_args.clone(),
        );
        let browser_service = Arc::new(BrowserService::new(browser_config));

        // Create database in account directory
        let database = Arc::new(
            DatabaseService::new(data_dir.to_str().unwrap())
                .map_err(|e| anyhow!("Failed to create database: {}", e))?,
        );

        // Create webhook service
        let webhook_service = WebhookService::new(app_config.webhooks.clone()).start_worker();

        // Create auth and chat services
        let auth_service = Arc::new(AuthService::new(
            app_config.clone(),
            browser_service.clone(),
        ));
        let chat_service = Arc::new(ChatService::with_database(
            app_config.clone(),
            browser_service.clone(),
            database.clone(),
        ));

        Ok(Self {
            id: config.id,
            _config: config,
            _app_config: app_config,
            data_dir,
            metadata: Arc::new(RwLock::new(metadata)),
            account_config: Arc::new(RwLock::new(account_config)),
            browser_service,
            auth_service: auth_service as Arc<dyn AuthServiceTrait>,
            chat_service: chat_service as Arc<dyn ChatServiceTrait>,
            database,
            webhook_service,
            status: Arc::new(RwLock::new(AccountStatus::Stopped)),
            operation_semaphore: Arc::new(Semaphore::new(1)),
            metrics: ServiceMetrics::new(),
            initialized: Arc::new(Mutex::new(false)),
            auth_cache: Arc::new(Mutex::new(None)),
        })
    }

    /// Load or create account metadata
    async fn load_or_create_metadata(
        data_dir: &PathBuf,
        config: &AccountSetupConfig,
    ) -> Result<AccountMetadata> {
        let metadata_path = data_dir.join(METADATA_FILE);

        if metadata_path.exists() {
            let content = tokio::fs::read_to_string(&metadata_path).await?;
            let metadata: AccountMetadata = serde_json::from_str(&content)
                .map_err(|e| anyhow!("Failed to parse account metadata: {}", e))?;
            debug!("Loaded metadata for account '{}'", config.id);
            Ok(metadata)
        } else {
            let metadata = AccountMetadata::new(config.id, config.phone_number.clone(), config.display_name.clone());
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

    async fn save_metadata_to_path(path: &PathBuf, metadata: &AccountMetadata) -> Result<()> {
        let content = serde_json::to_string_pretty(metadata)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Load or create account runtime config
    async fn load_or_create_account_config(
        data_dir: &PathBuf,
        config: &AccountSetupConfig,
    ) -> Result<AccountConfig> {
        let config_path = data_dir.join(ACCOUNT_CONFIG_FILE);

        if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path).await?;
            let account_config: AccountConfig = serde_json::from_str(&content)
                .map_err(|e| anyhow!("Failed to parse account config: {}", e))?;
            debug!("Loaded account config for account '{}'", config.id);
            Ok(account_config)
        } else {
            // Create default config from AccountSetupConfig
            let account_config = AccountConfig {
                account_id: Some(config.id),
                display_name: config.display_name.clone(),
                auto_start: config.auto_start,
                browser: AccountBrowserConfig {
                    headless: config.browser.headless.unwrap_or(true),
                    timeout_ms: 30000,
                    extra_args: config.browser.extra_args.clone(),
                },
                webhooks: AccountWebhookConfig::default(),
                rate_limits: AccountRateLimits::default(),
            };
            Self::save_account_config_to_path(&config_path, &account_config).await?;
            info!("Created new account config for account '{}'", config.id);
            Ok(account_config)
        }
    }

    async fn save_account_config_to_path(path: &PathBuf, config: &AccountConfig) -> Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Get the current account configuration
    pub async fn get_config(&self) -> AccountConfig {
        let config = self.account_config.read().await;
        let mut result = config.clone();
        // Always include account_id in response
        result.account_id = Some(self.id);
        result
    }

    /// Update account configuration with partial updates
    pub async fn update_config(
        &self,
        update: UpdateAccountConfigRequest,
    ) -> Result<AccountConfig> {
        let mut config = self.account_config.write().await;

        // Apply partial updates
        if let Some(display_name) = update.display_name {
            config.display_name = Some(display_name);
        }
        if let Some(auto_start) = update.auto_start {
            config.auto_start = auto_start;
        }
        if let Some(browser) = update.browser {
            if let Some(headless) = browser.headless {
                config.browser.headless = headless;
            }
            if let Some(timeout_ms) = browser.timeout_ms {
                config.browser.timeout_ms = timeout_ms;
            }
            if let Some(extra_args) = browser.extra_args {
                config.browser.extra_args = extra_args;
            }
        }
        if let Some(webhooks) = update.webhooks {
            if let Some(enabled) = webhooks.enabled {
                config.webhooks.enabled = enabled;
            }
            if let Some(endpoints) = webhooks.endpoints {
                config.webhooks.endpoints = endpoints;
            }
            if let Some(timeout_ms) = webhooks.timeout_ms {
                config.webhooks.timeout_ms = timeout_ms;
            }
            if let Some(retry_count) = webhooks.retry_count {
                config.webhooks.retry_count = retry_count;
            }
        }
        if let Some(rate_limits) = update.rate_limits {
            if let Some(messages_per_minute) = rate_limits.messages_per_minute {
                config.rate_limits.messages_per_minute = messages_per_minute;
            }
            if let Some(requests_per_minute) = rate_limits.requests_per_minute {
                config.rate_limits.requests_per_minute = requests_per_minute;
            }
            if let Some(message_cooldown_ms) = rate_limits.message_cooldown_ms {
                config.rate_limits.message_cooldown_ms = message_cooldown_ms;
            }
        }

        // Ensure account_id is set
        config.account_id = Some(self.id);

        // Save to disk
        let config_path = self.data_dir.join(ACCOUNT_CONFIG_FILE);
        Self::save_account_config_to_path(&config_path, &config).await?;

        info!("Updated account config for account '{}'", self.id);
        Ok(config.clone())
    }

    /// Start the account (launch browser, navigate to WhatsApp)
    pub async fn start(&self) -> Result<()> {
        let mut status = self.status.write().await;

        match &*status {
            AccountStatus::Running => {
                return Err(anyhow!("Account '{}' is already running", self.id));
            }
            AccountStatus::Starting => {
                return Err(anyhow!("Account '{}' is already starting", self.id));
            }
            _ => {}
        }

        *status = AccountStatus::Starting;
        drop(status);

        info!("Starting account '{}'", self.id);

        match self.browser_service.initialize().await {
            Ok(()) => {
                let mut status = self.status.write().await;
                *status = AccountStatus::Running;
                let mut initialized = self.initialized.lock().await;
                *initialized = true;
                info!("Account '{}' started successfully", self.id);
                Ok(())
            }
            Err(e) => {
                let mut status = self.status.write().await;
                *status = AccountStatus::Error(e.to_string());
                error!("Failed to start account '{}': {}", self.id, e);
                Err(e)
            }
        }
    }

    /// Stop the account (close browser, cleanup)
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping account '{}'", self.id);

        let mut status = self.status.write().await;
        *status = AccountStatus::Stopped;
        drop(status);

        let mut initialized = self.initialized.lock().await;
        *initialized = false;
        drop(initialized);

        self.browser_service.close().await?;

        info!("Account '{}' stopped", self.id);
        Ok(())
    }

    /// Get current account status
    pub async fn status(&self) -> AccountStatus {
        self.status.read().await.clone()
    }

    /// Get phone number from metadata (None if not yet authenticated with WhatsApp)
    pub fn phone_number(&self) -> Option<String> {
        self.metadata.try_read().ok().and_then(|m| m.phone_number.clone())
    }

    /// Get account info
    pub async fn info(&self) -> AccountInfo {
        let metadata = self.metadata.read().await;
        let status = self.status.read().await.clone();
        let browser_running = self.browser_service.is_running().await;

        // Check WhatsApp auth if browser is running
        let authorized = if browser_running {
            self.check_auth_status_directly().await.unwrap_or(false)
        } else {
            false
        };

        AccountInfo {
            id: self.id,
            phone_number: metadata.phone_number.clone(),
            display_name: metadata.display_name.clone(),
            status,
            authorized,
            created_at: metadata.created_at,
            last_activity: self.metrics.last_activity(),
        }
    }

    /// Called when WhatsApp Web authentication completes.
    /// Sets the phone number and display name from the authenticated session.
    pub async fn on_whatsapp_authenticated(&self, phone: &str) -> Result<()> {
        let auth_phone = crate::models::account::validate_phone_number(phone)
            .map_err(|e| anyhow!("Invalid authenticated phone: {}", e))?;

        // Check if another phone was already bound
        let current_phone = self.metadata.read().await.phone_number.clone();
        if let Some(ref existing) = current_phone {
            if *existing != auth_phone {
                return Err(anyhow!(
                    "Account is already bound to phone {}. Cannot rebind to {}.",
                    existing, auth_phone
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

        info!("Account '{}' authenticated with phone '{}'", self.id, auth_phone);
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

    /// Get reference to database
    pub fn database(&self) -> &Arc<DatabaseService> {
        &self.database
    }

    /// Get reference to webhook service
    pub fn webhook_service(&self) -> &Arc<WebhookService> {
        &self.webhook_service
    }

    /// Get reference to browser service
    pub fn browser_service(&self) -> &Arc<BrowserService> {
        &self.browser_service
    }

    /// Fire webhook for received message
    pub async fn fire_message_received_webhook(&self, data: WebhookMessageData) {
        self.webhook_service
            .fire(WebhookEvent::MessageReceived, data)
            .await;
    }

    // === Operations ===

    /// Check if the account is currently busy
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
                    "Account '{}' is busy with another operation",
                    self.id
                ));
            }
        };

        debug!(
            "Account '{}' - operation started (timeout: {:?})",
            self.id, timeout
        );

        // Wrap the operation with a timeout to prevent indefinite hangs
        let result = match tokio::time::timeout(timeout, operation).await {
            Ok(result) => result,
            Err(_) => {
                error!(
                    "Account '{}' - operation timed out after {:?}",
                    self.id, timeout
                );
                Err(anyhow!(
                    "Operation timed out after {:?}. Browser may be unresponsive.",
                    timeout
                ))
            }
        };

        drop(permit);
        debug!("Account '{}' - operation completed", self.id);

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
                warn!("Account '{}' - browser health check timed out", self.id);
                false
            }
        }
    }

    /// Check authentication status directly
    pub async fn check_auth_status_directly(&self) -> Result<bool> {
        let page = self.browser_service.get_whatsapp_page().await?;

        let script = r#"
            document.querySelector('#pane-side') !== null
        "#;

        match page.evaluate(script).await {
            Ok(result) => {
                let is_authorized = match result.into_value()? {
                    serde_json::Value::Bool(b) => b,
                    _ => false,
                };
                debug!(
                    "Account '{}' - auth check result: {}",
                    self.id, is_authorized
                );
                Ok(is_authorized)
            }
            Err(e) => {
                error!("Account '{}' - error checking auth status: {}", self.id, e);
                Ok(false)
            }
        }
    }

    /// Get authentication status with caching
    pub async fn get_auth_status(&self) -> Result<AuthStatusResponse> {
        // Check cache first
        {
            let cache = self.auth_cache.lock().await;
            if let Some(cached) = cache.as_ref() {
                if cached.cached_at.elapsed() < AUTH_CACHE_TTL {
                    debug!("Account '{}' - returning cached auth status", self.id);
                    return Ok(cached.status.clone());
                }
            }
        }

        // Cache miss or expired
        self.metrics.increment_auth_attempts();
        match self.auth_service.check_auth_status().await {
            Ok(auth_result) => {
                self.metrics.update_last_activity();

                let phone_number = if auth_result.authorized {
                    self.auth_service.get_sender_id().await.unwrap_or(None)
                } else {
                    None
                };

                let status = AuthStatusResponse {
                    authenticated: auth_result.authorized,
                    status: auth_result.status,
                    phone_number,
                };

                // Update cache
                {
                    let mut cache = self.auth_cache.lock().await;
                    *cache = Some(CachedAuthStatus {
                        status: status.clone(),
                        cached_at: Instant::now(),
                    });
                }

                Ok(status)
            }
            Err(e) => {
                self.metrics.increment_error_count();
                Err(e)
            }
        }
    }

    /// Invalidate auth cache
    pub async fn invalidate_auth_cache(&self) {
        let mut cache = self.auth_cache.lock().await;
        *cache = None;
    }

    /// Check if account is initialized
    pub async fn is_initialized(&self) -> bool {
        *self.initialized.lock().await
    }

    /// Health check
    pub async fn health_check(&self) -> Result<()> {
        if !self.is_initialized().await {
            return Err(anyhow!("Account '{}' not initialized", self.id));
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
                debug!("Account '{}' - dismissing dialog", self.id);
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

    /// Process pending messages from the queue
    pub async fn process_queue(&self) -> u32 {
        let mut processed_count = 0;

        loop {
            if self.is_busy().await {
                debug!("Account '{}' - busy, pausing queue", self.id);
                break;
            }

            let item = match self.database.dequeue_next() {
                Ok(Some(item)) => item,
                Ok(None) => {
                    debug!("Account '{}' - queue empty", self.id);
                    break;
                }
                Err(e) => {
                    error!("Account '{}' - error dequeuing: {}", self.id, e);
                    break;
                }
            };

            info!(
                "Account '{}' - processing message {} to {}",
                self.id, item.id, item.recipient
            );

            if let Err(e) = self.database.mark_processing(&item.id) {
                error!("Account '{}' - failed to mark processing: {}", self.id, e);
                continue;
            }

            let result = self
                .execute_with_busy_flag(async {
                    self.chat_service
                        .send_message(
                            &item.recipient,
                            item.text.as_deref(),
                            item.media_path.as_deref(),
                            None,
                        )
                        .await
                })
                .await;

            match result {
                Ok(_) => {
                    if let Err(e) = self.database.mark_sent(&item.id) {
                        error!("Account '{}' - failed to mark sent: {}", self.id, e);
                    }
                    processed_count += 1;
                    self.track_message_sent();
                    info!("Account '{}' - message {} sent", self.id, item.id);
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    error!(
                        "Account '{}' - failed to send {}: {}",
                        self.id, item.id, error_msg
                    );

                    if let Err(db_err) = self.database.mark_failed(&item.id, &error_msg) {
                        error!("Account '{}' - failed to mark failed: {}", self.id, db_err);
                    }
                    self.track_error();

                    if error_msg.contains("Not authorized") {
                        warn!("Account '{}' - stopping queue due to auth error", self.id);
                        break;
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        processed_count
    }

    /// Reset stuck messages
    pub fn reset_stuck_messages(&self) -> Result<()> {
        self.database.reset_stuck_processing()?;
        Ok(())
    }
}
