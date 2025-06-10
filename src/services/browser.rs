use crate::config::AppConfig;
use anyhow::Result;
use playwright::api::{Browser, BrowserContext, BrowserType, Page, Playwright};
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};
use tracing::{debug, error, info};

/// Browser service for managing Playwright browser instances
#[derive(Debug)]
pub struct BrowserService {
    config: Arc<AppConfig>,
    playwright: OnceCell<Playwright>,
    browser: Arc<Mutex<Option<Browser>>>,
    context: Arc<Mutex<Option<BrowserContext>>>,
    page: Arc<Mutex<Option<Page>>>,
}

impl BrowserService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            config,
            playwright: OnceCell::new(),
            browser: Arc::new(Mutex::new(None)),
            context: Arc::new(Mutex::new(None)),
            page: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize the browser service
    pub async fn initialize(&self) -> Result<()> {
        debug!("Initializing browser service");

        // Initialize Playwright
        let playwright = self.playwright.get_or_try_init(|| async {
            Playwright::initialize().await
        }).await?;

        // Launch browser
        let chromium = playwright.chromium();
        let mut launch_options = playwright::api::BrowserTypeLaunchOptions::default();
        launch_options.headless = Some(self.config.browser.headless);
        launch_options.args = Some(self.config.browser.args.clone());

        let browser = chromium.launch(Some(launch_options)).await?;
        info!("Browser launched successfully");

        // Create browser context
        let mut context_options = playwright::api::BrowserNewContextOptions::default();
        // Set a realistic user agent
        context_options.user_agent = Some("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string());
        
        let context = browser.new_context(Some(context_options)).await?;
        debug!("Browser context created");

        // Store browser and context
        *self.browser.lock().await = Some(browser);
        *self.context.lock().await = Some(context);

        Ok(())
    }

    /// Get or create a page for the specified URL
    pub async fn get_or_create_page(&self, url: &str) -> Result<Page> {
        // Ensure browser is initialized
        if self.browser.lock().await.is_none() {
            self.initialize().await?;
        }

        let mut page_guard = self.page.lock().await;
        
        if let Some(ref page) = *page_guard {
            // Check if the page is still valid
            if page.is_closed() {
                debug!("Existing page is closed, creating new one");
                *page_guard = None;
            } else {
                debug!("Reusing existing page");
                return Ok(page.clone());
            }
        }

        // Create new page
        let context = self.context.lock().await;
        let context = context.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Browser context not initialized")
        })?;

        let page = context.new_page().await?;
        
        // Navigate to URL
        debug!("Navigating to: {}", url);
        page.goto(url, None).await?;
        
        // Store the page
        *page_guard = Some(page.clone());
        
        info!("Page created and navigated to: {}", url);
        Ok(page)
    }

    /// Check if browser is running
    pub async fn is_running(&self) -> bool {
        if let Some(browser) = self.browser.lock().await.as_ref() {
            !browser.is_closed()
        } else {
            false
        }
    }

    /// Close the browser and clean up resources
    pub async fn close(&self) -> Result<()> {
        info!("Closing browser service");

        // Close page
        if let Some(page) = self.page.lock().await.take() {
            if !page.is_closed() {
                page.close(None).await?;
                debug!("Page closed");
            }
        }

        // Close context
        if let Some(context) = self.context.lock().await.take() {
            context.close().await?;
            debug!("Context closed");
        }

        // Close browser
        if let Some(browser) = self.browser.lock().await.take() {
            if !browser.is_closed() {
                browser.close().await?;
                debug!("Browser closed");
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
