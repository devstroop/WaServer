use crate::{
    browser::BrowserService,
    config::AppConfig,
    models::auth::AuthStatusResponse,
    services::{
        auth::{AuthService, AuthServiceTrait},
        chat::{ChatService, ChatServiceTrait},
        database::DatabaseService,
        webhook::{WebhookEvent, WebhookMessageData, WebhookService},
    },
    utils::metrics::{MetricsSnapshot, ServiceMetrics},
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, error, info, warn};

/// Main WhatsApp service that coordinates all operations
pub struct WhatsAppService {
    config: Arc<AppConfig>,
    browser_service: Arc<BrowserService>,
    auth_service: Arc<dyn AuthServiceTrait>,
    chat_service: Arc<dyn ChatServiceTrait>,
    db: Arc<DatabaseService>,
    webhook_service: Arc<WebhookService>,
    /// Semaphore for limiting concurrent operations (set to 1 for mutual exclusion)
    operation_semaphore: Arc<Semaphore>,
    metrics: ServiceMetrics,
    initialized: Arc<Mutex<bool>>,
}

impl WhatsAppService {
    /// Create a new WhatsApp service instance
    pub fn new(config: Arc<AppConfig>) -> Self {
        let browser_service = Arc::new(BrowserService::new(config.clone()));

        // Initialize database in data directory
        let data_dir = config
            .environment
            .data_directory
            .clone()
            .unwrap_or_else(|| "data".to_string());
        let db = match DatabaseService::new(&data_dir) {
            Ok(db) => Arc::new(db),
            Err(e) => {
                warn!(
                    "Failed to initialize database: {}. Running without persistence.",
                    e
                );
                // Create in-memory fallback (won't persist but won't crash)
                Arc::new(DatabaseService::in_memory().expect("In-memory DB should work"))
            }
        };

        // Initialize webhook service
        let webhook_service = WebhookService::new(config.webhooks.clone()).start_worker();
        if config.webhooks.enabled {
            info!(
                "Webhooks enabled with {} endpoint(s)",
                webhook_service.endpoints_count()
            );
        }

        let auth_service = Arc::new(AuthService::new(config.clone(), browser_service.clone()));
        let chat_service = Arc::new(ChatService::with_database(
            config.clone(),
            browser_service.clone(),
            db.clone(),
        ));

        Self {
            config: config.clone(),
            browser_service,
            auth_service: auth_service as Arc<dyn AuthServiceTrait>,
            chat_service: chat_service as Arc<dyn ChatServiceTrait>,
            db,
            webhook_service,
            operation_semaphore: Arc::new(Semaphore::new(1)), // Single permit for mutual exclusion
            metrics: ServiceMetrics::new(),
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    /// Get reference to the database service
    pub fn database(&self) -> &Arc<DatabaseService> {
        &self.db
    }

    /// Initialize the WhatsApp service
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing WhatsApp service");

        // Initialize browser service
        self.browser_service.initialize().await?;

        // Mark as initialized
        {
            let mut initialized = self.initialized.lock().await;
            *initialized = true;
        }

        info!("WhatsApp service initialized successfully");
        Ok(())
    }

    /// Get the API token from configuration
    pub fn get_api_token(&self) -> &str {
        &self.config.auth.api_token
    }

    /// Get reference to auth service
    pub fn get_auth_service(&self) -> &Arc<dyn AuthServiceTrait> {
        &self.auth_service
    }

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

    /// Fire webhook for received message
    pub async fn fire_message_received_webhook(&self, data: WebhookMessageData) {
        self.webhook_service
            .fire(WebhookEvent::MessageReceived, data)
            .await;
    }

    /// Pre-check to dismiss any dialogs that might be blocking operations
    pub async fn pre_check(&self) -> Result<()> {
        let page = self.browser_service.get_whatsapp_page().await?;

        // Check if there's a dialog and dismiss it
        if let Ok(_dialog) = page.find_element("[role='dialog']").await {
            if let Ok(backdrop) = page
                .find_element("div[data-animate-modal-backdrop='true']")
                .await
            {
                debug!("Dismissing dialog by clicking backdrop");
                backdrop.click().await?;

                // Wait for dialog to disappear
                tokio::time::timeout(std::time::Duration::from_millis(10000), async {
                    while page.find_element("[role='dialog']").await.is_ok() {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                })
                .await
                .map_err(|_| anyhow::anyhow!("Timeout waiting for dialog to disappear"))?;
            }
        }

        Ok(())
    }

    /// Wait for loading indicators to disappear
    pub async fn wait_til_loading(&self) -> Result<()> {
        let page = self.browser_service.get_whatsapp_page().await?;

        // Wait for loading progress indicator to disappear
        tokio::time::timeout(std::time::Duration::from_millis(10000), async {
            while page.find_element("progress[max='100']").await.is_ok() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        })
        .await
        .map_err(|_| anyhow::anyhow!("Timeout waiting for loading to complete"))?;

        Ok(())
    }

    /// Check if the service is currently busy (no permits available)
    pub async fn is_busy(&self) -> bool {
        self.operation_semaphore.available_permits() == 0
    }

    /// Execute an operation with exclusive access (acquires semaphore permit)
    pub async fn execute_with_busy_flag<F, T>(&self, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        // Try to acquire semaphore permit (non-blocking check first)
        let permit = match self.operation_semaphore.try_acquire() {
            Ok(permit) => permit,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "Service is already busy with another operation"
                ));
            }
        };

        debug!("Operation started - acquired exclusive access");

        // Execute the operation
        let result = operation.await;

        // Permit is automatically released when dropped
        drop(permit);
        debug!("Operation completed - released exclusive access");

        result
    }

    /// Check authentication status directly without busy flag
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
                debug!("Direct auth check result: {}", is_authorized);
                Ok(is_authorized)
            }
            Err(e) => {
                error!("Error checking auth status: {}", e);
                Ok(false)
            }
        }
    }

    /// Close the WhatsApp service and clean up resources
    pub async fn close(&self) -> Result<()> {
        info!("Closing WhatsApp service");

        // Mark as not initialized
        {
            let mut initialized = self.initialized.lock().await;
            *initialized = false;
        }

        // Close browser service
        self.browser_service.close().await?;

        info!("WhatsApp service closed successfully");
        Ok(())
    }

    /// Check if the service is initialized and ready
    pub async fn is_initialized(&self) -> bool {
        *self.initialized.lock().await
    }

    /// Health check for the WhatsApp service
    pub async fn health_check(&self) -> Result<()> {
        // Check if service is initialized
        if !self.is_initialized().await {
            return Err(anyhow::anyhow!("Service not initialized"));
        }

        // Check if browser service is healthy
        match self.browser_service.get_whatsapp_page().await {
            Ok(_) => Ok(()),
            Err(e) => {
                self.metrics.increment_error_count();
                Err(anyhow::anyhow!("Browser service unhealthy: {}", e))
            }
        }
    }

    /// Get service metrics snapshot
    pub async fn get_metrics(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Get authentication status with metrics tracking
    pub async fn get_auth_status(&self) -> Result<AuthStatusResponse> {
        self.metrics.increment_auth_attempts();
        match self.auth_service.is_authorized().await {
            Ok(is_authorized) => {
                self.metrics.update_last_activity();
                Ok(AuthStatusResponse {
                    authorized: is_authorized,
                    sender_id: self.auth_service.get_sender_id().await.unwrap_or(None),
                })
            }
            Err(e) => {
                self.metrics.increment_error_count();
                Err(e)
            }
        }
    }

    /// Track message sending with metrics
    pub fn track_message_sent(&self) {
        self.metrics.increment_messages_sent();
    }

    /// Track errors
    pub fn track_error(&self) {
        self.metrics.increment_error_count();
    }

    /// Process pending messages from the queue
    ///
    /// This processes messages one at a time until the queue is empty
    /// or an error occurs. Returns the number of messages processed.
    pub async fn process_queue(&self) -> u32 {
        let mut processed_count = 0;

        loop {
            // Check if we can process (not busy)
            if self.is_busy().await {
                debug!("Service busy, pausing queue processing");
                break;
            }

            // Get next message from queue
            let item = match self.db.dequeue_next() {
                Ok(Some(item)) => item,
                Ok(None) => {
                    debug!("Queue empty, stopping processor");
                    break;
                }
                Err(e) => {
                    error!("Error dequeuing message: {}", e);
                    break;
                }
            };

            info!(
                "Processing queued message {} to {}",
                item.id, item.recipient
            );

            // Mark as processing
            if let Err(e) = self.db.mark_processing(&item.id) {
                error!("Failed to mark message {} as processing: {}", item.id, e);
                continue;
            }

            // Send the message with busy flag (recipient is the phone for outgoing)
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
                    if let Err(e) = self.db.mark_sent(&item.id) {
                        error!("Failed to mark message {} as sent: {}", item.id, e);
                    }
                    processed_count += 1;
                    self.track_message_sent();
                    info!("Queued message {} sent successfully", item.id);
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    error!("Failed to send queued message {}: {}", item.id, error_msg);

                    if let Err(db_err) = self.db.mark_failed(&item.id, &error_msg) {
                        error!("Failed to mark message {} as failed: {}", item.id, db_err);
                    }
                    self.track_error();

                    // If it's a critical error (not authorized), stop processing
                    if error_msg.contains("Not authorized") {
                        warn!("Stopping queue processing due to auth error");
                        break;
                    }
                }
            }

            // Small delay between messages to avoid rate limiting
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        processed_count
    }

    /// Reset any stuck messages on startup
    pub fn reset_stuck_messages(&self) -> Result<()> {
        self.db.reset_stuck_processing()?;
        Ok(())
    }
}
