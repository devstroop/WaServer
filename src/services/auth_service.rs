use crate::{
    config::AppConfig,
    locators::LocatorDictionary,
    services::browser::BrowserService,
};
use anyhow::Result;
use async_trait::async_trait;
use playwright::api::Page;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

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
#[derive(Debug)]
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
        let locators = LocatorDictionary::new(page);

        // Wait for QR code to be visible
        let mut wait_options = playwright::api::LocatorWaitForOptions::default();
        wait_options.state = Some(playwright::api::WaitForSelectorState::Visible);
        wait_options.timeout = Some(10000.0);

        locators.scan_this_qr_element().wait_for(Some(wait_options)).await?;

        // Extract QR code from canvas
        let canvas_data = page.evaluate("document.getElementsByTagName('canvas')[0].toDataURL('image/png');", None).await?;
        
        let canvas_string = canvas_data.as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get QR code canvas data"))?;

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
        let locators = LocatorDictionary::new(&page);

        // Check if we're authorized by looking for the side pane
        let is_authorized = locators.authorized_side_pane()
            .is_visible(None)
            .await
            .unwrap_or(false);

        debug!("Authorization status: {}", is_authorized);
        Ok(is_authorized)
    }

    async fn get_sender_id(&self) -> Result<Option<String>> {
        let page = self.get_page().await?;

        if !self.is_authorized().await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        // Try to get sender ID from localStorage
        let sender_id = page.evaluate("window.localStorage.getItem('last-wid') || window.localStorage.getItem('last-wid-md') || '';", None).await?;
        
        let sender_string = sender_id.as_str()
            .unwrap_or("")
            .trim_matches('"');

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
        let locators = LocatorDictionary::new(&page);

        if self.is_authorized().await? {
            return Err(anyhow::anyhow!("Already authorized"));
        }

        // Check if we need to switch to QR code mode
        if locators.enter_phone_number_label().is_visible(None).await.unwrap_or(false) ||
           locators.enter_code_on_phone_label().is_visible(None).await.unwrap_or(false) {
            debug!("Switching to QR code login");
            locators.login_with_qr_code_link().click(None).await?;
        }

        // Wait for QR loading to complete
        if locators.qr_loading_indicator().is_visible(None).await.unwrap_or(false) {
            debug!("Waiting for QR code to load");
            let mut wait_options = playwright::api::LocatorWaitForOptions::default();
            wait_options.state = Some(playwright::api::WaitForSelectorState::Hidden);
            wait_options.timeout = Some(10000.0);

            if let Err(e) = locators.qr_loading_indicator().wait_for(Some(wait_options)).await {
                warn!("Timeout waiting for QR code to load: {}", e);
            }
        }

        // Check if we need to reload the QR code
        if locators.click_to_reload_qr_button().is_visible(None).await.unwrap_or(false) {
            debug!("Reloading QR code");
            locators.click_to_reload_qr_button().click(None).await?;
        }

        // Extract and return QR code
        self.extract_qr_code(&page).await
    }

    async fn login_with_phone_number(&self, phone_number: &str) -> Result<Option<String>> {
        let page = self.get_page().await?;
        let locators = LocatorDictionary::new(&page);

        if self.is_authorized().await? {
            return Err(anyhow::anyhow!("Already authorized"));
        }

        // Switch to phone number login if needed
        if locators.scan_this_qr_element().is_visible(None).await.unwrap_or(false) {
            debug!("Switching to phone number login");
            locators.login_with_phone_number_link().click(None).await?;
        }

        // Wait for phone input to be visible
        let mut wait_options = playwright::api::LocatorWaitForOptions::default();
        wait_options.state = Some(playwright::api::WaitForSelectorState::Visible);
        wait_options.timeout = Some(5000.0);

        locators.enter_phone_number_input().wait_for(Some(wait_options)).await?;

        // Enter phone number
        debug!("Entering phone number: {}", phone_number);
        locators.enter_phone_number_input().clear(None).await?;
        locators.enter_phone_number_input().fill(phone_number, None).await?;
        locators.submit_phone_number_button().click(None).await?;

        // Wait for code to appear
        let mut wait_options = playwright::api::LocatorWaitForOptions::default();
        wait_options.state = Some(playwright::api::WaitForSelectorState::Visible);
        wait_options.timeout = Some(10000.0);

        if let Err(e) = locators.enter_code_on_phone_value().wait_for(Some(wait_options)).await {
            warn!("Timeout waiting for phone code: {}", e);
            return Ok(None);
        }

        // Extract the code
        let code_element = locators.link_phone_number_code_element();
        if let Ok(Some(code)) = code_element.get_attribute("data-link-code", None).await {
            let formatted_code = code.replace(",", "");
            info!("Phone authentication code generated: {}", formatted_code);
            Ok(Some(formatted_code))
        } else {
            Ok(None)
        }
    }

    async fn logout(&self) -> Result<()> {
        let page = self.get_page().await?;
        let locators = LocatorDictionary::new(&page);

        if !self.is_authorized().await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        debug!("Logging out");

        // Click menu
        locators.menu().click(None).await?;

        // Wait for menu to open and click logout
        let mut wait_options = playwright::api::LocatorWaitForOptions::default();
        wait_options.state = Some(playwright::api::WaitForSelectorState::Visible);
        wait_options.timeout = Some(5000.0);

        locators.menu_logout().wait_for(Some(wait_options)).await?;
        locators.menu_logout().click(None).await?;

        info!("Logout completed");
        Ok(())
    }
}
