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
}

impl BrowserService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            config,
            browser: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize the browser service
    pub async fn initialize(&self) -> Result<()> {
        debug!("Initializing browser service with chromiumoxide");

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

        // Add essential args for stability
        browser_config = browser_config
            .arg("--no-sandbox")
            .arg("--disable-setuid-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-web-security")
            .arg("--disable-features=VizDisplayCompositor");

        let config = browser_config.build().map_err(|e| anyhow::anyhow!("Failed to build browser config: {}", e))?;

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
                        if h.is_err() {
                            tracing::error!("Browser handler error: {:?}", h);
                            break;
                        }
                    }
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

    /// Get or create a page for the specified URL
    pub async fn get_or_create_page(&self, url: &str) -> Result<Page> {
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

        // Close browser
        if let Some(mut browser) = self.browser.lock().await.take() {
            if let Err(e) = browser.close().await {
                tracing::error!("Error closing browser: {}", e);
            }
            debug!("Browser closed");
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
