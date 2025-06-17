use crate::{
    config::AppConfig,
    services::{
        auth_service::{AuthService, AuthServiceTrait},
        browser::BrowserService,
        chat_service::{ChatService, ChatServiceTrait},
    },
    utils::metrics::{ServiceMetrics, MetricsSnapshot},
    models::auth::AuthStatusResponse,
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, error, info};

/// Main WhatsApp service that coordinates all operations
pub struct WhatsAppService {
    config: Arc<AppConfig>,
    browser_service: Arc<BrowserService>,
    auth_service: Arc<dyn AuthServiceTrait>,
    chat_service: Arc<dyn ChatServiceTrait>,
    busy_flag: Arc<Mutex<bool>>,
    operation_semaphore: Arc<Semaphore>,
    metrics: ServiceMetrics,
    initialized: Arc<Mutex<bool>>,
}

impl WhatsAppService {
    /// Create a new WhatsApp service instance
    pub fn new(config: Arc<AppConfig>) -> Self {
        let browser_service = Arc::new(BrowserService::new(config.clone()));
        let auth_service = Arc::new(AuthService::new(config.clone(), browser_service.clone()));
        let chat_service = Arc::new(ChatService::new(config.clone(), browser_service.clone()));

        Self {
            config: config.clone(),
            browser_service,
            auth_service: auth_service as Arc<dyn AuthServiceTrait>,
            chat_service: chat_service as Arc<dyn ChatServiceTrait>,
            busy_flag: Arc::new(Mutex::new(false)),
            operation_semaphore: Arc::new(Semaphore::new(config.limits.max_concurrent_requests)),
            metrics: ServiceMetrics::new(),
            initialized: Arc::new(Mutex::new(false)),
        }
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

    /// Pre-check to dismiss any dialogs that might be blocking operations
    pub async fn pre_check(&self) -> Result<()> {
        let page = self.browser_service.get_whatsapp_page().await?;
        
        // Check if there's a dialog and dismiss it
        if let Ok(_dialog) = page.find_element("[role='dialog']").await {
            if let Ok(backdrop) = page.find_element("div[data-animate-modal-backdrop='true']").await {
                debug!("Dismissing dialog by clicking backdrop");
                backdrop.click().await?;
                
                // Wait for dialog to disappear
                tokio::time::timeout(
                    std::time::Duration::from_millis(10000),
                    async {
                        while page.find_element("[role='dialog']").await.is_ok() {
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        }
                    }
                ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for dialog to disappear"))?;
            }
        }
        
        Ok(())
    }

    /// Wait for loading indicators to disappear
    pub async fn wait_til_loading(&self) -> Result<()> {
        let page = self.browser_service.get_whatsapp_page().await?;
        
        // Wait for loading progress indicator to disappear
        tokio::time::timeout(
            std::time::Duration::from_millis(10000),
            async {
                while page.find_element("progress[max='100']").await.is_ok() {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for loading to complete"))?;
        
        Ok(())
    }

    /// Check if the service is currently busy
    pub async fn is_busy(&self) -> bool {
        *self.busy_flag.lock().await
    }

    /// Execute an operation with the busy flag set
    pub async fn execute_with_busy_flag<F, T>(&self, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        // Acquire semaphore permit to limit concurrent operations
        let _permit = self.operation_semaphore.acquire().await?;
        
        // Set busy flag
        {
            let mut busy = self.busy_flag.lock().await;
            if *busy {
                return Err(anyhow::anyhow!("Service is already busy with another operation"));
            }
            *busy = true;
        }

        debug!("Operation started - service marked as busy");

        // Execute the operation
        let result = operation.await;

        // Clear busy flag
        {
            let mut busy = self.busy_flag.lock().await;
            *busy = false;
        }

        debug!("Operation completed - service marked as available");

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
}
