use crate::config::AppConfig;
use anyhow::Result;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};
use futures_util::stream::StreamExt;

/// Browser service for managing Chrome browser instances
pub struct BrowserService {
    config: Arc<AppConfig>,
    browser: Arc<Mutex<Option<Browser>>>,
    whatsapp_page: Arc<Mutex<Option<Page>>>,
    user_data_dir: Arc<Mutex<Option<String>>>,
}

impl BrowserService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            config,
            browser: Arc::new(Mutex::new(None)),
            whatsapp_page: Arc::new(Mutex::new(None)),
            user_data_dir: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize the browser service
    pub async fn initialize(&self) -> Result<()> {
        debug!("Initializing browser service with chromiumoxide");

        // Clean up any leftover Chrome processes
        self.cleanup_existing_chrome_processes().await;

        // Create browser config
        let mut browser_config = BrowserConfig::builder();

        // Set headless mode
        if !self.config.browser.headless {
            browser_config = browser_config.with_head();
        }

        // Add Chrome args
        for arg in &self.config.browser.args {
            browser_config = browser_config.arg(arg);
        }

        // Generate a unique user data directory to avoid singleton lock issues
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let user_data_dir = format!("/tmp/chromiumoxide-whatsapp-{}-{}", std::process::id(), timestamp);

        // Add essential args for stability
        browser_config = browser_config
            .arg("--no-sandbox")
            .arg("--disable-setuid-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-web-security")
            .arg("--disable-features=VizDisplayCompositor")
            .arg("--no-first-run")
            .arg("--disable-default-apps")
            .arg("--disable-background-timer-throttling")
            .arg("--disable-renderer-backgrounding")
            .arg("--disable-backgrounding-occluded-windows")
            .arg("--disable-extensions")
            .arg("--disable-plugins")
            .arg("--disable-gpu")
            .arg("--remote-debugging-port=0") // Let Chrome choose an available port
            .arg(&format!("--user-data-dir={}", user_data_dir));

        let config = browser_config.build().map_err(|e| anyhow::anyhow!("Failed to build browser config: {}", e))?;

        // Store the user data directory for cleanup
        *self.user_data_dir.lock().await = Some(user_data_dir);

        // Launch browser with timeout
        info!("Launching Chrome browser with chromiumoxide...");
        
        match tokio::time::timeout(
            std::time::Duration::from_secs(15),
            Browser::launch(config)
        ).await {
            Ok(Ok((browser, mut handler))) => {
                info!("Browser launched successfully with chromiumoxide");
                
                // Spawn handler task to manage browser process
                tokio::spawn(async move {
                    while let Some(h) = handler.next().await {
                        if let Err(e) = h {
                            // Log browser handler errors but don't break unless it's a critical error
                            tracing::debug!("Browser handler event error (this is normal): {:?}", e);
                            // Only break on critical connection errors
                            if e.to_string().contains("connection closed") || e.to_string().contains("broken pipe") {
                                tracing::error!("Critical browser connection error: {:?}", e);
                                break;
                            }
                        }
                    }
                    tracing::debug!("Browser handler task completed");
                });
                
                *self.browser.lock().await = Some(browser);
                Ok(())
            }
            Ok(Err(e)) => {
                tracing::error!("Failed to launch browser: {}", e);
                tracing::warn!("Continuing without browser - browser-dependent features will not work");
                Ok(())
            }
            Err(_) => {
                tracing::error!("Browser launch timed out after 15 seconds");
                tracing::warn!("Continuing without browser - browser-dependent features will not work");
                Ok(())
            }
        }
    }

    /// Clean up any existing Chrome processes that might be holding locks
    async fn cleanup_existing_chrome_processes(&self) {
        debug!("Checking for existing Chrome processes...");
        
        // Try to kill any Chrome processes that might be using our user data directory
        let _ = tokio::process::Command::new("pkill")
            .args(&["-f", "chromium-browser"])
            .output()
            .await;
            
        let _ = tokio::process::Command::new("pkill")
            .args(&["-f", "chrome"])
            .output()
            .await;
            
        let _ = tokio::process::Command::new("pkill")
            .args(&["-f", "google-chrome"])
            .output()
            .await;

        // Wait a moment for processes to terminate
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        debug!("Chrome cleanup completed");
    }

    /// Get or create a page for the specified URL
    pub async fn get_or_create_page(&self, url: &str) -> Result<Page> {
        // For WhatsApp Web, always use the same persistent page
        if url.contains("web.whatsapp.com") {
            return self.get_whatsapp_page().await;
        }

        // For other URLs, create new pages as needed
        self.create_new_page(url).await
    }

    /// Get the persistent WhatsApp Web page (creates if doesn't exist)
    pub async fn get_whatsapp_page(&self) -> Result<Page> {
        // Check if we already have a WhatsApp page
        {
            let page_guard = self.whatsapp_page.lock().await;
            if let Some(ref page) = *page_guard {
                // Verify the page is still active
                if page.url().await.is_ok() {
                    debug!("Reusing existing WhatsApp Web page");
                    return Ok(page.clone());
                }
            }
        }

        // Create new WhatsApp page if none exists or previous one is inactive
        debug!("Creating new WhatsApp Web page");
        let page = self.create_new_page("https://web.whatsapp.com").await?;
        
        // Store the page for future use
        *self.whatsapp_page.lock().await = Some(page.clone());
        
        Ok(page)
    }

    /// Create a new page for any URL
    async fn create_new_page(&self, url: &str) -> Result<Page> {
        // Ensure browser is initialized
        if self.browser.lock().await.is_none() {
            self.initialize().await?;
        }

        // Create new page
        let browser = self.browser.lock().await;
        let browser = browser.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Browser not initialized")
        })?;

        let page = browser.new_page(url).await?;
        
        // Set user agent  
        page.set_user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36").await?;
        
        debug!("Created new page and navigated to: {}", url);
        Ok(page)
    }

    /// Check if browser is running
    pub async fn is_running(&self) -> bool {
        self.browser.lock().await.is_some()
    }

    /// Close the browser and clean up resources
    pub async fn close(&self) -> Result<()> {
        info!("Closing browser service");

        // Clear the WhatsApp page reference
        *self.whatsapp_page.lock().await = None;

        // Close browser
        if let Some(mut browser) = self.browser.lock().await.take() {
            if let Err(e) = browser.close().await {
                tracing::error!("Error closing browser: {}", e);
            }
            debug!("Browser closed");
        }

        // Clean up user data directory
        if let Some(user_data_dir) = self.user_data_dir.lock().await.take() {
            if std::path::Path::new(&user_data_dir).exists() {
                if let Err(e) = std::fs::remove_dir_all(&user_data_dir) {
                    tracing::warn!("Failed to remove user data directory {}: {}", user_data_dir, e);
                } else {
                    debug!("Cleaned up user data directory: {}", user_data_dir);
                }
            }
        }

        info!("Browser service closed successfully");
        Ok(())
    }
}

impl Drop for BrowserService {
    fn drop(&mut self) {
        debug!("BrowserService dropped");
    }
}
