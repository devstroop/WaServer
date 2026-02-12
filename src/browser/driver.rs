//! Browser Driver
//!
//! Chrome browser lifecycle management using chromiumoxide.
//! Handles browser launch, page management, and session persistence.

use crate::config::AppConfig;
use anyhow::Result;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

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

        // Use a PERSISTENT user data directory for session preservation
        // This allows WhatsApp Web to remember the login session across restarts
        let base_dir = if cfg!(target_os = "windows") {
            std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
                std::env::var("APPDATA").unwrap_or_else(|_| "C:\\Users\\Public".to_string())
            })
        } else {
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
        };

        let user_data_dir = format!("{}/was/chrome-profile", base_dir);

        // Ensure the directory exists
        std::fs::create_dir_all(&user_data_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create user data directory: {}", e))?;

        debug!("Using persistent Chrome profile at: {}", user_data_dir);

        // Fix Chrome crash state (like .NET CrashFix)
        // This prevents the "Chrome didn't shut down correctly" dialog
        self.fix_chrome_crash_state(&user_data_dir);

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
            .arg("--remote-debugging-port=0")
            .arg(format!("--user-data-dir={}", user_data_dir));

        let config = browser_config
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build browser config: {}", e))?;

        // Store the user data directory for cleanup
        *self.user_data_dir.lock().await = Some(user_data_dir);

        // Launch browser with timeout
        info!("Launching Chrome browser with chromiumoxide...");

        match tokio::time::timeout(std::time::Duration::from_secs(30), Browser::launch(config))
            .await
        {
            Ok(Ok((browser, mut handler))) => {
                info!("Browser launched successfully with chromiumoxide");

                // Spawn handler task to manage browser process
                tokio::spawn(async move {
                    while let Some(h) = handler.next().await {
                        if let Err(e) = h {
                            tracing::debug!(
                                "Browser handler event error (this is normal): {:?}",
                                e
                            );
                            if e.to_string().contains("connection closed")
                                || e.to_string().contains("broken pipe")
                            {
                                tracing::error!("Critical browser connection error: {:?}", e);
                                break;
                            }
                        }
                    }
                    tracing::debug!("Browser handler task completed");
                });

                // Store browser first
                *self.browser.lock().await = Some(browser);

                // Now navigate to WhatsApp Web
                info!("Navigating to WhatsApp Web...");

                // Get browser reference
                let browser_guard = self.browser.lock().await;
                let browser = browser_guard.as_ref().unwrap();

                // Wait a moment for Chrome's default tab to be ready
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                // Get the default page that Chrome creates on startup and navigate it
                let whatsapp_page = match browser.pages().await {
                    Ok(pages) if !pages.is_empty() => {
                        // Use the existing default tab (about:blank or new tab page)
                        let page = pages.into_iter().next().unwrap();
                        debug!("Using Chrome's default tab, navigating to WhatsApp Web");
                        match page.goto("https://web.whatsapp.com").await {
                            Ok(_) => {
                                let _ = page.set_user_agent(Self::user_agent()).await;
                                Some(page)
                            }
                            Err(e) => {
                                tracing::warn!("Failed to navigate default page: {}", e);
                                None
                            }
                        }
                    }
                    _ => {
                        // Fallback: create new page if somehow no default page exists
                        debug!("No default tab found, creating new page for WhatsApp Web");
                        match browser.new_page("https://web.whatsapp.com").await {
                            Ok(page) => {
                                let _ = page.set_user_agent(Self::user_agent()).await;
                                Some(page)
                            }
                            Err(e) => {
                                tracing::warn!("Failed to create WhatsApp Web page: {}", e);
                                None
                            }
                        }
                    }
                };

                if let Some(page) = whatsapp_page {
                    info!("WhatsApp Web page loaded successfully");
                    drop(browser_guard); // Release browser lock before acquiring page lock
                    *self.whatsapp_page.lock().await = Some(page);
                } else {
                    tracing::warn!(
                        "Failed to load WhatsApp Web page on startup (will retry on first request)"
                    );
                }

                Ok(())
            }
            Ok(Err(e)) => {
                tracing::error!("Failed to launch browser: {}", e);
                Err(anyhow::anyhow!("Browser initialization failed: {}", e))
            }
            Err(_) => {
                tracing::error!("Browser launch timed out after 30 seconds");
                Err(anyhow::anyhow!(
                    "Browser launch timeout - please ensure Chrome is installed"
                ))
            }
        }
    }

    /// User agent string for WhatsApp Web
    fn user_agent() -> &'static str {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    }

    /// Fix Chrome crash state to prevent "Chrome didn't shut down correctly" dialog
    fn fix_chrome_crash_state(&self, user_data_dir: &str) {
        let profile_dirs = ["Default", "Profile 1"];

        for profile in profile_dirs {
            let preferences_path = format!("{}/{}/Preferences", user_data_dir, profile);

            if let Ok(content) = std::fs::read_to_string(&preferences_path) {
                if content.contains("\"Crashed\"") {
                    debug!("Fixing Chrome crash state in {}", preferences_path);
                    let fixed_content = content.replace("\"Crashed\"", "\"Normal\"");
                    if let Err(e) = std::fs::write(&preferences_path, fixed_content) {
                        tracing::warn!("Failed to fix Chrome crash state: {}", e);
                    } else {
                        debug!("Chrome crash state fixed successfully");
                    }
                }
            }
        }
    }

    /// Clean up any existing Chrome processes
    async fn cleanup_existing_chrome_processes(&self) {
        debug!("Checking for existing Chrome processes...");

        if cfg!(target_os = "windows") {
            let chrome_processes = ["chrome.exe", "msedge.exe", "chromium.exe"];
            for process in chrome_processes {
                let _ = tokio::process::Command::new("taskkill")
                    .args(["/F", "/IM", process])
                    .output()
                    .await;
            }
        } else {
            let chrome_processes = [
                "Google Chrome",
                "chromium-browser",
                "chrome",
                "google-chrome",
                "Chromium",
            ];
            for process in chrome_processes {
                let _ = tokio::process::Command::new("pkill")
                    .args(["-f", process])
                    .output()
                    .await;
            }

            tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

            for process in chrome_processes {
                let _ = tokio::process::Command::new("pkill")
                    .args(["-9", "-f", process])
                    .output()
                    .await;
            }
        }

        // Clean up temp directories
        let temp_dir = if cfg!(target_os = "windows") {
            std::env::var("TEMP").unwrap_or_else(|_| {
                std::env::var("TMP").unwrap_or_else(|_| "C:\\Windows\\Temp".to_string())
            })
        } else {
            "/tmp".to_string()
        };

        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.starts_with("chromiumoxide-whatsapp") {
                        let _ = std::fs::remove_dir_all(entry.path());
                    }
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        debug!("Chrome cleanup completed");
    }

    /// Get or create a page for the specified URL
    pub async fn get_or_create_page(&self, url: &str) -> Result<Page> {
        if url.contains("web.whatsapp.com") {
            return self.get_whatsapp_page().await;
        }
        self.create_new_page(url).await
    }

    /// Get the persistent WhatsApp Web page
    pub async fn get_whatsapp_page(&self) -> Result<Page> {
        {
            let page_guard = self.whatsapp_page.lock().await;
            if let Some(ref page) = *page_guard {
                if page.url().await.is_ok() {
                    debug!("Reusing existing WhatsApp Web page");
                    return Ok(page.clone());
                }
            }
        }

        debug!("Creating new WhatsApp Web page");
        let page = self.create_new_page("https://web.whatsapp.com").await?;
        *self.whatsapp_page.lock().await = Some(page.clone());
        Ok(page)
    }

    /// Create a new page for any URL
    async fn create_new_page(&self, url: &str) -> Result<Page> {
        let mut retries = 0;
        while self.browser.lock().await.is_none() && retries < 3 {
            info!(
                "Browser not initialized, attempting initialization (attempt {})",
                retries + 1
            );
            if let Err(e) = self.initialize().await {
                retries += 1;
                if retries >= 3 {
                    return Err(anyhow::anyhow!(
                        "Failed to initialize browser after {} attempts: {}",
                        retries,
                        e
                    ));
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
            break;
        }

        let browser = self.browser.lock().await;
        let browser = browser
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Browser not initialized"))?;

        let page = browser.new_page(url).await?;
        page.set_user_agent(Self::user_agent()).await?;

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

        *self.whatsapp_page.lock().await = None;

        if let Some(mut browser) = self.browser.lock().await.take() {
            if let Err(e) = browser.close().await {
                tracing::error!("Error closing browser: {}", e);
            }
            debug!("Browser closed");
        }

        if let Some(user_data_dir) = self.user_data_dir.lock().await.take() {
            debug!(
                "Preserving user data directory for session persistence: {}",
                user_data_dir
            );
        }

        info!("Browser service closed successfully");
        Ok(())
    }
}
