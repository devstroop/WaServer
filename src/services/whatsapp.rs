use crate::{
    config::AppConfig,
    services::{
        auth_service::{AuthService, AuthServiceTrait},
        browser::BrowserService,
        chat_service::{ChatService, ChatServiceTrait},
    },
};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, error, info, warn};

/// Main WhatsApp service that coordinates all operations
#[derive(Debug)]
pub struct WhatsAppService {
    config: Arc<AppConfig>,
    browser_service: Arc<BrowserService>,
    auth_service: Arc<dyn AuthServiceTrait>,
    chat_service: Arc<dyn ChatServiceTrait>,
    busy_flag: Arc<Mutex<bool>>,
    operation_semaphore: Arc<Semaphore>,
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
        }
    }

    /// Initialize the WhatsApp service
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing WhatsApp service");
        
        // Initialize browser service
        self.browser_service.initialize().await?;
        
        info!("WhatsApp service initialized successfully");
        Ok(())
    }

    /// Get reference to auth service
    pub fn auth_service(&self) -> &Arc<dyn AuthServiceTrait> {
        &self.auth_service
    }

    /// Get reference to chat service
    pub fn chat_service(&self) -> &Arc<dyn ChatServiceTrait> {
        &self.chat_service
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
        let page = self.browser_service.get_or_create_page("https://web.whatsapp.com").await?;

        let script = r#"
            document.querySelector('#pane-side') !== null
        "#;

        match page.evaluate(script, None).await {
            Ok(result) => {
                let is_authorized = result.as_bool().unwrap_or(false);
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

        // Close browser service
        self.browser_service.close().await?;

        info!("WhatsApp service closed successfully");
        Ok(())
    }
}
