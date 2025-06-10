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
        let _locators = LocatorDictionary::new();

        // Check if we're authorized by looking for the side pane
        let is_authorized = match page.find_element("#pane-side").await {
            Ok(_) => true,
            Err(_) => false,
        };

        debug!("Authorization status: {}", is_authorized);
        Ok(is_authorized)
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
        let _locators = LocatorDictionary::new();

        if self.is_authorized().await? {
            return Err(anyhow::anyhow!("Already authorized"));
        }

        // Check if we need to switch to QR code mode
        let phone_number_visible = page.find_element("text='Enter phone number'").await.is_ok();
        let enter_code_visible = page.find_element("text='Enter code on phone'").await.is_ok();
        
        if phone_number_visible || enter_code_visible {
            debug!("Switching to QR code login");
            if let Ok(qr_link) = page.find_element("text='Log in with QR code'").await {
                qr_link.click().await?;
            }
        }

        // Wait for QR loading to complete
        if page.find_element("svg[role='status']").await.is_ok() {
            debug!("Waiting for QR code to load");
            tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
        }

        // Check if we need to reload the QR code
        if page.find_element("text='Click to reload QR code'").await.is_ok() {
            debug!("Reloading QR code");
            if let Ok(reload_button) = page.find_element("text='Click to reload QR code'").await {
                reload_button.click().await?;
            }
        }

        // Extract and return QR code
        self.extract_qr_code(&page).await
    }

    async fn login_with_phone_number(&self, phone_number: &str) -> Result<Option<String>> {
        let page = self.get_page().await?;
        let _locators = LocatorDictionary::new();

        if self.is_authorized().await? {
            return Err(anyhow::anyhow!("Already authorized"));
        }

        // Switch to phone number login if needed
        if page.find_element("[aria-label='Scan this QR code to link a device!']").await.is_ok() {
            debug!("Switching to phone number login");
            if let Ok(phone_link) = page.find_element("text='Log in with phone number'").await {
                phone_link.click().await?;
            }
        }

        // Wait for phone input to be visible
        page.find_element("[aria-label='Type your phone number.']").await?;

        // Enter phone number
        debug!("Entering phone number: {}", phone_number);
        let phone_input = page.find_element("[aria-label='Type your phone number.']").await?;
        
        // Clear the input using JavaScript
        page.evaluate("document.querySelector('[aria-label=\"Type your phone number.\"]').value = '';").await?;
        phone_input.type_str(phone_number).await?;
        
        if let Ok(submit_button) = page.find_element("text='Next'").await {
            submit_button.click().await?;
        }

        // Wait for code to appear
        page.find_element("[aria-label='Enter code on phone:']").await?;

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
