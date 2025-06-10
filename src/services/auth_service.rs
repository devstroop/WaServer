use crate::{
    config::AppConfig,
    locators::LocatorDictionary,
    services::browser::BrowserService,
};
use anyhow::Result;
use async_trait::async_trait;
use chromiumoxide::page::Page;
use std::sync::Arc;
use tracing::{debug, info};

/// Authentication service trait
#[async_trait]
pub trait AuthServiceTrait: Send + Sync {
    async fn is_authorized(&self) -> Result<bool>;
    async fn get_sender_id(&self) -> Result<Option<String>>;
    async fn get_auth_qr_code(&self) -> Result<String>;
    async fn login_with_phone_number(&self, phone_number: &str) -> Result<Option<String>>;
    async fn logout(&self) -> Result<()>;
}

/// WhatsApp authentication service
pub struct AuthService {
    config: Arc<AppConfig>,
    browser_service: Arc<BrowserService>,
}

impl AuthService {
    pub fn new(config: Arc<AppConfig>, browser_service: Arc<BrowserService>) -> Self {
        Self {
            config,
            browser_service,
        }
    }

    /// Get page from browser service
    async fn get_page(&self) -> Result<Page> {
        self.browser_service.get_or_create_page("https://web.whatsapp.com").await
    }

    /// Wait for QR code to be visible and extract it
    async fn extract_qr_code(&self, page: &Page) -> Result<String> {
        let _locators = LocatorDictionary::new();

        // Wait for QR code canvas to be visible
        page.find_element("canvas").await?;

        // Extract QR code from canvas
        let canvas_result = page.evaluate("document.getElementsByTagName('canvas')[0].toDataURL('image/png');").await?;
        
        let canvas_string = match canvas_result.into_value()? {
            serde_json::Value::String(data) => data,
            _ => return Err(anyhow::anyhow!("Failed to get QR code canvas data")),
        };

        if canvas_string.is_empty() {
            return Err(anyhow::anyhow!("Failed to get QR code canvas"));
        }

        let parts: Vec<&str> = canvas_string.split(',').collect();
        if parts.len() < 2 || parts[1].is_empty() {
            return Err(anyhow::anyhow!("Invalid QR code data format"));
        }

        // The data is already base64 encoded, just return the image data part
        Ok(parts[1].to_string())
    }
}

#[async_trait]
impl AuthServiceTrait for AuthService {
    async fn is_authorized(&self) -> Result<bool> {
        let page = self.get_page().await?;
        
        // Wait for loading to complete first
        tokio::time::timeout(
            std::time::Duration::from_millis(10000),
            async {
                while page.find_element("progress[max='100']").await.is_ok() {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for loading to complete"))?;

        // Check various states to determine authorization
        if page.find_element("text='Log into WhatsApp Web'").await.is_ok() {
            debug!("Not authorized - Login screen visible");
            return Ok(false);
        }
        
        if page.find_element("text='Enter phone number'").await.is_ok() {
            debug!("Not authorized - Phone entry screen visible");
            return Ok(false);
        }
        
        if page.find_element("text='Enter code on phone'").await.is_ok() {
            debug!("Not authorized - Code entry screen visible");
            return Ok(false);
        }
        
        if page.find_element("#pane-side").await.is_ok() {
            debug!("Authorized - Side pane visible");
            return Ok(true);
        }

        debug!("Authorization status unclear, defaulting to false");
        Ok(false)
    }

    async fn get_sender_id(&self) -> Result<Option<String>> {
        let page = self.get_page().await?;

        if !self.is_authorized().await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        // Try to get sender ID from localStorage
        let sender_result = page.evaluate("window.localStorage.getItem('last-wid') || window.localStorage.getItem('last-wid-md') || '';").await?;
        
        let sender_string = match sender_result.into_value()? {
            serde_json::Value::String(s) => s,
            _ => String::new(),
        }
        .trim_matches('"')
        .to_string();

        if sender_string.is_empty() {
            return Ok(None);
        }

        // Extract the actual phone number from the sender ID
        let cleaned_id = sender_string
            .split('@')
            .next()
            .and_then(|part| part.split(':').next())
            .map(|s| s.to_string());

        debug!("Sender ID: {:?}", cleaned_id);
        Ok(cleaned_id)
    }

    async fn get_auth_qr_code(&self) -> Result<String> {
        let page = self.get_page().await?;

        if self.is_authorized().await? {
            return Err(anyhow::anyhow!("Already authorized"));
        }

        // Check if we need to switch to QR code mode
        if page.find_element("text='Enter phone number'").await.is_ok()
            || page.find_element("text='Enter code on phone'").await.is_ok() {
            debug!("Switching to QR code login");
            if let Ok(qr_link) = page.find_element("text='Log in with QR code'").await {
                qr_link.click().await?;
            }
        }

        // Wait for QR code to be visible or loading to complete
        if page.find_element("[aria-label='Scan this QR code to link a device!']").await.is_err() {
            if page.find_element("svg[role='status']").await.is_ok() {
                debug!("Waiting for QR code to load...");
                // Wait for loading indicator to disappear with extended timeout
                tokio::time::timeout(
                    std::time::Duration::from_millis(20000), // Increased to 20 seconds
                    async {
                        let mut attempts = 0;
                        while page.find_element("svg[role='status']").await.is_ok() && attempts < 40 {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            attempts += 1;
                            if attempts % 10 == 0 {
                                debug!("QR code still loading... (attempt {})", attempts);
                            }
                        }
                    }
                ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for QR code to load - please check your internet connection"))?;
            }
        }

        // Check if we need to reload the QR code
        if page.find_element("text='Click to reload QR code'").await.is_ok() {
            debug!("Reloading QR code");
            if let Ok(reload_button) = page.find_element("text='Click to reload QR code'").await {
                reload_button.click().await?;
            }
        }

        // Wait for QR code to be visible with extended timeout
        tokio::time::timeout(
            std::time::Duration::from_millis(15000), // Increased to 15 seconds
            async {
                let mut attempts = 0;
                while page.find_element("[aria-label='Scan this QR code to link a device!']").await.is_err() && attempts < 30 {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    attempts += 1;
                    if attempts % 6 == 0 {
                        debug!("Still waiting for QR code to appear... (attempt {})", attempts);
                    }
                }
            }
        ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for QR code to appear - please refresh and try again"))?;

        // Extract and return QR code
        self.extract_qr_code(&page).await
    }

    async fn login_with_phone_number(&self, phone_number: &str) -> Result<Option<String>> {
        let page = self.get_page().await?;

        if self.is_authorized().await? {
            return Err(anyhow::anyhow!("Already authorized"));
        }

        // Format phone number with + if not present
        let formatted_phone = if phone_number.contains('+') {
            phone_number.to_string()
        } else {
            format!("+{}", phone_number)
        };

        debug!("Starting phone authentication for: {}", formatted_phone);

        // Switch to phone number login if we're in QR mode
        if page.find_element("text='Log into WhatsApp Web'").await.is_ok() {
            debug!("Switching to phone number login");
            if let Ok(phone_link) = page.find_element("text='Log in with phone number'").await {
                phone_link.click().await?;
                
                // Wait for phone input to be visible with extended timeout
                tokio::time::timeout(
                    std::time::Duration::from_millis(20000), // Increased to 20 seconds
                    async {
                        let mut attempts = 0;
                        while page.find_element("text='Enter phone number'").await.is_err() && attempts < 40 {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            attempts += 1;
                            if attempts % 10 == 0 {
                                debug!("Still waiting for phone input screen... (attempt {})", attempts);
                            }
                        }
                    }
                ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for phone input screen - QR code may still be loading"))?;
            }
        }

        // Enter phone number
        if page.find_element("text='Enter phone number'").await.is_ok() {
            debug!("Entering phone number: {}", formatted_phone);
            
            // Wait for the input field to be ready
            let phone_input = tokio::time::timeout(
                std::time::Duration::from_millis(5000),
                async {
                    loop {
                        if let Ok(input) = page.find_element("[aria-label='Type your phone number.']").await {
                            return Ok::<_, anyhow::Error>(input);
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    }
                }
            ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for phone input field"))??;
            
            // Clear and fill the input
            phone_input.click().await?;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await; // Small delay
            page.evaluate("document.querySelector('[aria-label=\"Type your phone number.\"]').value = '';").await?;
            phone_input.type_str(&formatted_phone).await?;
            
            // Wait a moment for the input to register
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            
            // Click Next button
            if let Ok(submit_button) = page.find_element("text='Next'").await {
                debug!("Clicking Next button");
                submit_button.click().await?;
            }
        }

        // Wait for code screen to appear with extended timeout
        debug!("Waiting for code input screen...");
        tokio::time::timeout(
            std::time::Duration::from_millis(25000), // Increased to 25 seconds
            async {
                let mut attempts = 0;
                while page.find_element("[aria-label='Enter code on phone:']").await.is_err() && attempts < 50 {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    attempts += 1;
                    if attempts % 10 == 0 {
                        debug!("Still waiting for code input screen... (attempt {})", attempts);
                        
                        // Check if we're still on the phone number screen
                        if page.find_element("text='Enter phone number'").await.is_ok() {
                            debug!("Still on phone number screen - may need to retry submission");
                            if let Ok(submit_button) = page.find_element("text='Next'").await {
                                let _ = submit_button.click().await;
                            }
                        }
                    }
                }
            }
        ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for code input screen - phone number may be invalid or network issues"))?;

        // Wait for link code element to appear with extended timeout
        debug!("Waiting for link code element...");
        tokio::time::timeout(
            std::time::Duration::from_millis(15000), // Increased to 15 seconds
            async {
                let mut attempts = 0;
                while page.find_element("[aria-details='link-device-phone-number-code-screen-instructions']").await.is_err() && attempts < 30 {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    attempts += 1;
                    if attempts % 6 == 0 {
                        debug!("Still waiting for link code element... (attempt {})", attempts);
                    }
                }
            }
        ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for link code element - please check your phone for WhatsApp notifications"))?;

        // Extract the code using the locators
        let locators = LocatorDictionary::new();
        if let Ok(Some(code)) = locators.data_link_code(&page).await {
            let formatted_code = code.replace(",", "");
            info!("Phone authentication code generated: {}", formatted_code);
            Ok(Some(formatted_code))
        } else {
            Ok(None)
        }
    }

    async fn logout(&self) -> Result<()> {
        let page = self.get_page().await?;
        let _locators = LocatorDictionary::new();

        if !self.is_authorized().await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        debug!("Logging out");

        // Click menu
        if let Ok(menu) = page.find_element("[aria-label='Menu']").await {
            menu.click().await?;
        }

        // Wait for menu to open and click logout
        page.find_element("[aria-label='Log out']").await?;
        if let Ok(logout) = page.find_element("[aria-label='Log out']").await {
            logout.click().await?;
        }

        info!("Logout completed");
        Ok(())
    }
}
