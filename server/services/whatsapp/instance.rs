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
        InstanceRateLimits, InstanceSetupConfig, InstanceStatus, InstanceWebhookConfig,
        UpdateInstanceConfigRequest,
    },
    services::{
        auth::{AuthService, AuthServiceTrait},
        webhook::{WebhookEvent, WebhookMessageData, WebhookService},
    },
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
struct CachedAuthStatus {
    status: AuthStatusResponse,
    cached_at: Instant,
}

/// Cache TTL for auth status (5 seconds)
const AUTH_CACHE_TTL: Duration = Duration::from_secs(5);

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
    _config: InstanceSetupConfig,
    /// Global app config
    _app_config: Arc<AppConfig>,
    /// Data directory for this instance
    data_dir: PathBuf,
    /// Instance metadata (includes bound phone)
    metadata: Arc<RwLock<InstanceMetadata>>,
    /// Instance runtime configuration (API-managed)
    instance_config: Arc<RwLock<InstanceConfig>>,
    /// Isolated browser service
    browser_service: Arc<BrowserService>,
    /// Auth service
    auth_service: Arc<dyn AuthServiceTrait>,
    /// Chat service
    chat_service: Arc<dyn ChatServiceTrait>,
    /// Webhook service
    webhook_service: Arc<WebhookService>,
    /// Current account status
    status: Arc<RwLock<InstanceStatus>>,
    /// Operation semaphore for mutual exclusion
    operation_semaphore: Arc<Semaphore>,
    /// Service metrics
    metrics: ServiceMetrics,
    /// Whether the instance is initialized
    initialized: Arc<Mutex<bool>>,
    /// Cached auth status
    auth_cache: Arc<Mutex<Option<CachedAuthStatus>>>,
    /// Timestamp of last activity (for idle auto-sleep)
    last_activity: Arc<RwLock<Instant>>,
    /// Handle to the idle-sleep background task (cancelled on sleep/warmup)
    idle_sleep_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
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

        // Create webhook service
        let webhook_service = WebhookService::new(app_config.webhooks.clone()).start_worker();

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
            webhook_service,
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
                webhooks: InstanceWebhookConfig::default(),
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
    pub async fn update_config(
        &self,
        update: UpdateInstanceConfigRequest,
    ) -> Result<InstanceConfig> {
        let mut config = self.instance_config.write().await;

        // Apply partial updates
        if let Some(instance_name) = update.instance_name {
            config.instance_name = Some(instance_name);
        }
        if let Some(idle_timeout) = update.idle_timeout {
            config.idle_timeout = idle_timeout;
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

        // Ensure instance_id is set
        config.instance_id = Some(self.id);

        // Save to disk
        let config_path = self.data_dir.join(ACCOUNT_CONFIG_FILE);
        Self::save_instance_config_to_path(&config_path, &config).await?;

        info!("Updated account config for account '{}'", self.id);
        Ok(config.clone())
    }

    /// Warmup the instance (launch browser, navigate to WhatsApp)
    /// If already active, this is a no-op. If warming up, returns error.
    pub async fn warmup(&self) -> Result<()> {
        // If status is Active, verify the browser is actually alive
        {
            let status = self.status.read().await;
            if matches!(&*status, InstanceStatus::Active) {
                drop(status);
                if self.browser_service.is_responsive().await {
                    self.touch_activity().await;
                    return Ok(());
                }
                // Browser died externally — clean up stale handles and re-warm
                warn!(
                    "Instance '{}' — browser dead while status=Active, re-warming",
                    self.id
                );
                self.cancel_idle_timer().await;
                // Close clears the dead browser/page handles so initialize() starts fresh
                let _ = self.browser_service.close().await;
                let mut s = self.status.write().await;
                *s = InstanceStatus::Sleeping;
                drop(s);
            }
        }

        let mut status = self.status.write().await;

        match &*status {
            InstanceStatus::Active => {
                // Raced with another caller that already re-warmed
                drop(status);
                self.touch_activity().await;
                return Ok(());
            }
            InstanceStatus::WarmingUp => {
                return Err(anyhow!("Instance '{}' is already warming up", self.id));
            }
            _ => {}
        }

        *status = InstanceStatus::WarmingUp;
        drop(status);

        info!("Warming up account '{}'", self.id);

        match self.browser_service.initialize().await {
            Ok(()) => {
                // Start auth watcher immediately in background (parallel to UI check)
                let instance_id = self.id;
                let phone_number = self._config.phone_number.clone();
                let browser_service = self.browser_service.clone();
                let auth_service = self.auth_service.clone();
                let auth_cache = self.auth_cache.clone();
                let account_status = self.status.clone();
                tokio::spawn(async move {
                    Self::auth_watcher_loop(
                        instance_id,
                        phone_number,
                        browser_service,
                        auth_service,
                        auth_cache,
                        account_status,
                    )
                    .await;
                });

                // Wait for WhatsApp Web UI to be ready (either logged in or showing QR)
                // This prevents "Not authorized" errors when the UI hasn't loaded yet
                debug!("Instance '{}' — waiting for WhatsApp UI to be ready...", self.id);
                
                let max_wait = Duration::from_secs(10);
                let poll_interval = Duration::from_millis(200);
                let start = std::time::Instant::now();
                
                let ui_ready = loop {
                    if start.elapsed() > max_wait {
                        warn!("Instance '{}' — UI ready check timed out, proceeding anyway", self.id);
                        break false;
                    }
                    
                    // Check if either #pane-side (logged in) or QR canvas exists
                    if let Ok(page) = self.browser_service.get_whatsapp_page().await {
                        let check_script = r#"
                            document.querySelector('#pane-side') !== null || 
                            document.querySelector('canvas[aria-label]') !== null ||
                            document.querySelector('[data-ref]') !== null
                        "#;
                        if let Ok(result) = page.evaluate(check_script).await {
                            if result.into_value::<bool>().unwrap_or(false) {
                                break true;
                            }
                        }
                    }
                    
                    tokio::time::sleep(poll_interval).await;
                };
                
                if ui_ready {
                    debug!("Instance '{}' — WhatsApp UI is ready", self.id);
                }

                let mut status = self.status.write().await;
                *status = InstanceStatus::Active;
                let mut initialized = self.initialized.lock().await;
                *initialized = true;
                drop(initialized);
                drop(status);

                self.touch_activity().await;
                self.start_idle_timer().await;

                info!("Instance '{}' warmed up successfully", self.id);

                Ok(())
            }
            Err(e) => {
                let mut status = self.status.write().await;
                *status = InstanceStatus::Error(e.to_string());
                error!("Failed to warm up instance '{}': {}", self.id, e);
                Err(e)
            }
        }
    }

    /// Ensure the instance is warm (active). Auto-warms if sleeping.
    /// Call this before any operation that requires the browser.
    pub async fn ensure_warm(&self) -> Result<()> {
        let status = self.status.read().await.clone();
        match status {
            InstanceStatus::Active => {
                // Verify the browser is actually alive — it may have been closed externally
                if !self.browser_service.is_responsive().await {
                    warn!(
                        "Instance '{}' — browser dead while status=Active, re-warming",
                        self.id
                    );
                    self.cancel_idle_timer().await;
                    let _ = self.browser_service.close().await;
                    *self.status.write().await = InstanceStatus::Sleeping;
                    return self.warmup().await;
                }
                self.touch_activity().await;
                Ok(())
            }
            InstanceStatus::Sleeping | InstanceStatus::Error(_) => self.warmup().await,
            InstanceStatus::WarmingUp => {
                // Wait for warmup to complete (poll with short interval)
                drop(status);
                for _ in 0..300 {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    let s = self.status.read().await.clone();
                    match s {
                        InstanceStatus::Active => {
                            self.touch_activity().await;
                            return Ok(());
                        }
                        InstanceStatus::Error(e) => {
                            return Err(anyhow!("Instance warmup failed: {}", e));
                        }
                        InstanceStatus::Sleeping => {
                            return self.warmup().await;
                        }
                        InstanceStatus::WarmingUp => continue,
                    }
                }
                Err(anyhow!(
                    "Timeout waiting for account '{}' to warm up",
                    self.id
                ))
            }
        }
    }

    /// Update last activity timestamp (resets the idle timer)
    pub async fn touch_activity(&self) {
        let mut last = self.last_activity.write().await;
        *last = Instant::now();
    }

    /// Start the idle-sleep background timer
    async fn start_idle_timer(&self) {
        // Cancel any existing idle timer
        self.cancel_idle_timer().await;

        let idle_timeout_val = self.instance_config.read().await.idle_timeout;
        if idle_timeout_val == 0 {
            debug!(
                "Instance '{}' — idle auto-sleep disabled (timeout=0)",
                self.id
            );
            return;
        }

        let instance_id = self.id;
        let status = self.status.clone();
        let last_activity = self.last_activity.clone();
        let browser_service = self.browser_service.clone();
        let initialized = self.initialized.clone();
        let idle_timeout = Duration::from_secs(idle_timeout_val);

        let handle = tokio::spawn(async move {
            let check_interval = Duration::from_secs(30);
            loop {
                tokio::time::sleep(check_interval).await;

                // Check if still active
                if !matches!(&*status.read().await, InstanceStatus::Active) {
                    return;
                }

                let elapsed = last_activity.read().await.elapsed();
                if elapsed >= idle_timeout {
                    info!(
                        "Instance '{}' — idle for {:?}, auto-sleeping",
                        instance_id, elapsed
                    );

                    // Perform sleep: close browser, set status
                    let mut s = status.write().await;
                    *s = InstanceStatus::Sleeping;
                    drop(s);

                    let mut init = initialized.lock().await;
                    *init = false;
                    drop(init);

                    if let Err(e) = browser_service.close().await {
                        warn!("Instance '{}' — error during auto-sleep: {}", instance_id, e);
                    }

                    info!("Instance '{}' — auto-slept", instance_id);
                    return;
                }
            }
        });

        let mut guard = self.idle_sleep_handle.lock().await;
        *guard = Some(handle);
    }

    /// Cancel the idle-sleep timer
    async fn cancel_idle_timer(&self) {
        let mut guard = self.idle_sleep_handle.lock().await;
        if let Some(handle) = guard.take() {
            handle.abort();
        }
    }

    /// Sleep the instance (close browser, set to sleeping)
    pub async fn sleep(&self) -> Result<()> {
        info!("Sleeping account '{}'", self.id);

        self.cancel_idle_timer().await;

        let mut status = self.status.write().await;
        *status = InstanceStatus::Sleeping;
        drop(status);

        let mut initialized = self.initialized.lock().await;
        *initialized = false;
        drop(initialized);

        self.browser_service.close().await?;

        info!("Instance '{}' is now sleeping", self.id);
        Ok(())
    }

    // ========================================================================
    // DOM Auth Observer + Continuous Phone Watcher
    // ========================================================================

    /// JavaScript that installs a MutationObserver on the DOM to detect when
    /// WhatsApp transitions to the authenticated state (chat list appears).
    /// When detected, it reads the sender ID from localStorage and writes
    /// a timestamped event to `__was_auth_event` so the Rust watcher can pick it up.
    const AUTH_OBSERVER_JS: &'static str = r##"
        (function() {
            if (window.__was_auth_observer) return 'already_injected';

            // Helper: extract phone from WID
            function extractPhone() {
                try {
                    var wid = localStorage.getItem('last-wid') ?? localStorage.getItem('last-wid-md');
                    if (!wid) return null;
                    return wid.replace(/"/g, '').split('@')[0].split(':')[0];
                } catch(e) { return null; }
            }

            // Record an auth event
            function recordAuthEvent() {
                var phone = extractPhone();
                var evt = JSON.stringify({ ts: Date.now(), phone: phone || null });
                localStorage.setItem('__was_auth_event', evt);
            }

            // Check immediately if already authorized
            if (document.querySelector('#pane-side')
                || document.querySelector('[data-testid="chat-list"]')
                || document.querySelector('div[aria-label="Chat list"]')) {
                recordAuthEvent();
            }

            // Watch for future auth transitions
            var observer = new MutationObserver(function() {
                if (document.querySelector('#pane-side')
                    || document.querySelector('[data-testid="chat-list"]')
                    || document.querySelector('div[aria-label="Chat list"]')) {
                    recordAuthEvent();
                }
            });
            observer.observe(document.body, { childList: true, subtree: true });
            window.__was_auth_observer = observer;
            return 'injected';
        })()
    "##;

    /// Inject the DOM auth observer into the WhatsApp Web page.
    async fn inject_auth_observer(
        instance_id: InstanceId,
        browser_service: &Arc<BrowserService>,
    ) -> bool {
        let page = match browser_service.get_whatsapp_page().await {
            Ok(p) => p,
            Err(e) => {
                warn!(
                    "Instance '{}' — failed to get page for auth observer injection: {}",
                    instance_id, e
                );
                return false;
            }
        };

        match page.evaluate(Self::AUTH_OBSERVER_JS).await {
            Ok(result) => {
                let status = result
                    .into_value::<serde_json::Value>()
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default();
                info!("Instance '{}' — auth observer: {}", instance_id, status);
                true
            }
            Err(e) => {
                warn!(
                    "Instance '{}' — failed to inject auth observer: {}",
                    instance_id, e
                );
                false
            }
        }
    }

    /// Read and consume the `__was_auth_event` from localStorage.
    /// Returns the phone number from the event, if any.
    async fn consume_auth_event(
        instance_id: InstanceId,
        browser_service: &Arc<BrowserService>,
    ) -> Option<Option<String>> {
        let page = match browser_service.get_whatsapp_page().await {
            Ok(p) => p,
            Err(_) => return None,
        };

        let js = r#"
            (function() {
                var raw = localStorage.getItem('__was_auth_event');
                if (!raw) return null;
                localStorage.removeItem('__was_auth_event');
                try { return JSON.parse(raw); } catch(e) { return null; }
            })()
        "#;

        let result = match page.evaluate(js).await {
            Ok(r) => r,
            Err(e) => {
                debug!(
                    "Instance '{}' — failed to read auth event: {}",
                    instance_id, e
                );
                return None;
            }
        };

        let value: serde_json::Value = match result.into_value() {
            Ok(v) => v,
            Err(_) => return None,
        };

        if value.is_null() {
            return None;
        }

        // Event found — extract phone (may be null in the JSON)
        let phone = value
            .get("phone")
            .and_then(|v| v.as_str())
            .map(String::from);

        Some(phone)
    }

    /// Continuous background watcher that polls for auth events and verifies the phone number.
    /// The DOM observer is already injected by `start()` — this just consumes events.
    /// Runs until the instance is stopped.
    async fn auth_watcher_loop(
        instance_id: InstanceId,
        phone_number: Option<String>,
        browser_service: Arc<BrowserService>,
        auth_service: Arc<dyn AuthServiceTrait>,
        auth_cache: Arc<Mutex<Option<CachedAuthStatus>>>,
        status: Arc<RwLock<InstanceStatus>>,
    ) {
        let expected_phone = match phone_number {
            Some(p) => p,
            None => {
                info!(
                    "Instance '{}' — no phone configured, auth watcher not needed",
                    instance_id
                );
                return;
            }
        };

        info!(
            "Instance '{}' — auth watcher started (expected phone: {})",
            instance_id, expected_phone
        );

        // Poll loop: check for auth events
        // First check is immediate, then every 2 seconds
        let poll_interval = Duration::from_secs(2);
        let mut first_check = true;

        loop {
            if !first_check {
                tokio::time::sleep(poll_interval).await;
            }
            first_check = false;

            // Stop if account is no longer running
            if !Self::is_running_status(&status).await {
                info!(
                    "Instance '{}' — auth watcher exiting (account stopped)",
                    instance_id
                );
                return;
            }

            // Check for auth event from the DOM observer
            let event = Self::consume_auth_event(instance_id, &browser_service).await;

            let browser_phone = match event {
                None => continue, // no event yet
                Some(phone_opt) => {
                    match phone_opt {
                        Some(phone) if !phone.is_empty() => phone,
                        _ => {
                            // Event fired but couldn't read phone from localStorage yet.
                            // Fall back to fetching via auth_service.
                            debug!(
                                "Instance '{}' — auth event with no phone, fetching sender ID...",
                                instance_id
                            );
                            match auth_service.get_sender_id().await {
                                Ok(Some(p)) => p,
                                Ok(None) => {
                                    warn!(
                                        "Instance '{}' — auth event but sender ID still null",
                                        instance_id
                                    );
                                    continue;
                                }
                                Err(e) => {
                                    warn!(
                                        "Instance '{}' — auth event but get_sender_id failed: {}",
                                        instance_id, e
                                    );
                                    continue;
                                }
                            }
                        }
                    }
                }
            };

            info!(
                "Instance '{}' — auth event detected, browser phone: {}",
                instance_id, browser_phone
            );

            if browser_phone == expected_phone {
                info!(
                    "Instance '{}' — phone verified: {}",
                    instance_id, expected_phone
                );
                // Keep watching — user could log out and re-auth with wrong phone later
                continue;
            }

            warn!(
                "Instance '{}' — phone mismatch: expected {}, found {}. Triggering logout.",
                instance_id, expected_phone, browser_phone
            );

            if let Err(e) = auth_service.logout().await {
                error!("Instance '{}' — auto-logout failed: {}", instance_id, e);
            }
            // Invalidate auth cache
            let mut cache = auth_cache.lock().await;
            *cache = None;
            drop(cache);

            // After logout the page reloads — re-inject observer once it settles
            tokio::time::sleep(Duration::from_secs(5)).await;
            if Self::is_running_status(&status).await {
                // Re-inject by touching the page (triggers run_page_scripts if page was recreated,
                // otherwise inject manually since the in-memory page may have reloaded)
                Self::inject_auth_observer(instance_id, &browser_service).await;
            }
        }
    }

    /// Check if the instance status is Active
    async fn is_running_status(status: &Arc<RwLock<InstanceStatus>>) -> bool {
        matches!(&*status.read().await, InstanceStatus::Active)
    }

    /// Reset the instance — sleep browser and wipe all session data (chrome profile, sessions, media).
    /// The account record and config are preserved; only runtime/browser data is cleared.
    pub async fn reset(&self) -> Result<()> {
        info!("Resetting account '{}'", self.id);

        // Sleep browser first
        self.sleep().await.ok(); // ignore if already sleeping

        // Wipe session-related directories
        let dirs_to_clear = ["chrome-profile", "sessions", "media"];
        for dir_name in &dirs_to_clear {
            let dir = self.data_dir.join(dir_name);
            if dir.exists() {
                tokio::fs::remove_dir_all(&dir).await?;
                tokio::fs::create_dir_all(&dir).await?;
            }
        }

        // Clear auth cache
        self.invalidate_auth_cache().await;

        info!("Instance '{}' reset — session data cleared", self.id);
        Ok(())
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
                    "Instance '{}' - auth check result: {}",
                    self.id, is_authorized
                );
                Ok(is_authorized)
            }
            Err(e) => {
                error!("Instance '{}' - error checking auth status: {}", self.id, e);
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
                    debug!("Instance '{}' - returning cached auth status", self.id);
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

                // Phone mismatch guard: if the instance has a configured phone number
                // and the browser is authorized with a different number, auto-logout.
                if auth_result.authorized {
                    if let Some(ref browser_phone) = phone_number {
                        if let Some(ref expected_phone) = self._config.phone_number {
                            if browser_phone != expected_phone {
                                warn!(
                                    "Instance '{}' — phone mismatch: expected {}, got {}. Auto-logging out.",
                                    self.id, expected_phone, browser_phone
                                );
                                // Trigger logout in the background — don't block the status response
                                let auth_svc = self.auth_service.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = auth_svc.logout().await {
                                        tracing::error!("Auto-logout failed: {}", e);
                                    }
                                });
                                self.invalidate_auth_cache().await;

                                let status = AuthStatusResponse {
                                    authenticated: false,
                                    status: format!(
                                        "Phone mismatch — expected {}, got {}. Session terminated.",
                                        expected_phone, browser_phone
                                    ),
                                    phone_number: Some(browser_phone.clone()),
                                };
                                return Ok(status);
                            }
                        }
                    }
                }

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
