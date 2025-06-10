use crate::{
    config::AppConfig,
    locators::LocatorDictionary,
    services::browser::BrowserService,
};
use anyhow::Result;
use async_trait::async_trait;
use headless_chrome::Tab;
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

    /// Get tab from browser service
    async fn get_tab(&self) -> Result<Arc<Tab>> {
        self.browser_service.get_or_create_tab("https://web.whatsapp.com").await
    }

    /// Wait for QR code to be visible and extract it
    async fn extract_qr_code(&self, tab: &Arc<Tab>) -> Result<String> {
        let _locators = LocatorDictionary::new(tab.clone());

        // Wait for QR code canvas to be visible
        tab.wait_for_element("canvas")?;

        // Extract QR code from canvas
        let canvas_result = tab.evaluate("document.getElementsByTagName('canvas')[0].toDataURL('image/png');", false)?;
        
        let canvas_data = canvas_result.value.ok_or_else(|| {
            anyhow::anyhow!("Failed to get QR code canvas data")
        })?;
        
        let canvas_string = canvas_data.as_str().ok_or_else(|| {
            anyhow::anyhow!("Canvas data is not a string")
        })?;

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
        let tab = self.get_tab().await?;
        let _locators = LocatorDictionary::new(tab.clone());

        // Check if we're authorized by looking for the side pane
        let is_authorized = match tab.find_element("#pane-side") {
            Ok(_) => true,
            Err(_) => false,
        };

        debug!("Authorization status: {}", is_authorized);
        Ok(is_authorized)
    }

    async fn get_sender_id(&self) -> Result<Option<String>> {
        let tab = self.get_tab().await?;

        if !self.is_authorized().await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        // Try to get sender ID from localStorage
        let sender_result = tab.evaluate("window.localStorage.getItem('last-wid') || window.localStorage.getItem('last-wid-md') || '';", false)?;
        
        let sender_string = sender_result.value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default()
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
        let tab = self.get_tab().await?;
        let _locators = LocatorDictionary::new(tab.clone());

        if self.is_authorized().await? {
            return Err(anyhow::anyhow!("Already authorized"));
        }

        // Check if we need to switch to QR code mode
        let phone_number_visible = tab.find_element("text='Enter phone number'").is_ok();
        let enter_code_visible = tab.find_element("text='Enter code on phone'").is_ok();
        
        if phone_number_visible || enter_code_visible {
            debug!("Switching to QR code login");
            if let Ok(qr_link) = tab.find_element("text='Log in with QR code'") {
                qr_link.click()?;
            }
        }

        // Wait for QR loading to complete
        if tab.find_element("svg[role='status']").is_ok() {
            debug!("Waiting for QR code to load");
            // Wait a bit for QR to load
            std::thread::sleep(std::time::Duration::from_millis(2000));
        }

        // Check if we need to reload the QR code
        if tab.find_element("text='Click to reload QR code'").is_ok() {
            debug!("Reloading QR code");
            if let Ok(reload_button) = tab.find_element("text='Click to reload QR code'") {
                reload_button.click()?;
            }
        }

        // Extract and return QR code
        self.extract_qr_code(&tab).await
    }

    async fn login_with_phone_number(&self, phone_number: &str) -> Result<Option<String>> {
        let tab = self.get_tab().await?;
        let _locators = LocatorDictionary::new(tab.clone());

        if self.is_authorized().await? {
            return Err(anyhow::anyhow!("Already authorized"));
        }

        // Switch to phone number login if needed
        if tab.find_element("[aria-label='Scan this QR code to link a device!']").is_ok() {
            debug!("Switching to phone number login");
            if let Ok(phone_link) = tab.find_element("text='Log in with phone number'") {
                phone_link.click()?;
            }
        }

        // Wait for phone input to be visible
        tab.wait_for_element("[aria-label='Type your phone number.']")?;

        // Enter phone number
        debug!("Entering phone number: {}", phone_number);
        let phone_input = tab.find_element("[aria-label='Type your phone number.']")?;
        
        // Clear the input using JavaScript
        tab.evaluate("document.querySelector('[aria-label=\"Type your phone number.\"]').value = '';", false)?;
        phone_input.type_into(phone_number)?;
        
        if let Ok(submit_button) = tab.find_element("text='Next'") {
            submit_button.click()?;
        }

        // Wait for code to appear
        tab.wait_for_element("[aria-label='Enter code on phone:']")?;

        // Extract the code using the locators
        let locators = LocatorDictionary::new(tab.clone());
        if let Ok(Some(code)) = locators.data_link_code() {
            let formatted_code = code.replace(",", "");
            info!("Phone authentication code generated: {}", formatted_code);
            Ok(Some(formatted_code))
        } else {
            Ok(None)
        }
    }

    async fn logout(&self) -> Result<()> {
        let tab = self.get_tab().await?;
        let _locators = LocatorDictionary::new(tab.clone());

        if !self.is_authorized().await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        debug!("Logging out");

        // Click menu
        if let Ok(menu) = tab.find_element("[aria-label='Menu']") {
            menu.click()?;
        }

        // Wait for menu to open and click logout
        tab.wait_for_element("[aria-label='Log out']")?;
        if let Ok(logout) = tab.find_element("[aria-label='Log out']") {
            logout.click()?;
        }

        info!("Logout completed");
        Ok(())
    }
}
