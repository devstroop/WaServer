use crate::config::AppConfig;
use anyhow::Result;
use headless_chrome::{Browser, Tab, LaunchOptions};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Browser service for managing headless Chrome browser instances
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
        debug!("Initializing browser service");

        // Create launch options
        let args: Vec<&std::ffi::OsStr> = self.config.browser.args.iter()
            .map(|s| std::ffi::OsStr::new(s.as_str()))
            .collect();
            
        let launch_options = LaunchOptions::default_builder()
            .headless(self.config.browser.headless)
            .args(args)
            .user_data_dir(Some(std::env::temp_dir().join("whatsapp-engine")))
            .build()
            .expect("Failed to build launch options");

        // Launch browser
        let browser = Browser::new(launch_options)?;
        info!("Browser launched successfully");

        // Store browser
        *self.browser.lock().await = Some(browser);

        Ok(())
    }

    /// Get or create a tab for the specified URL
    pub async fn get_or_create_tab(&self, url: &str) -> Result<Arc<Tab>> {
        // Ensure browser is initialized
        if self.browser.lock().await.is_none() {
            self.initialize().await?;
        }

        // Create new tab
        let browser = self.browser.lock().await;
        let browser = browser.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Browser not initialized")
        })?;

        let tab = browser.new_tab()?;
        
        // Set user agent  
        tab.set_user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36", None, None)?;
        
        // Navigate to URL
        debug!("Navigating to: {}", url);
        tab.navigate_to(url)?;
        tab.wait_until_navigated()?;
        
        info!("Tab created and navigated to: {}", url);
        Ok(tab)
    }

    /// Check if browser is running
    pub async fn is_running(&self) -> bool {
        self.browser.lock().await.is_some()
    }

    /// Close the browser and clean up resources
    pub async fn close(&self) -> Result<()> {
        info!("Closing browser service");

        // Close browser
        if let Some(_browser) = self.browser.lock().await.take() {
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
