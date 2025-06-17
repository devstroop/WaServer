use crate::{
    config::AppConfig,
    locators::LocatorDictionary,
    services::{browser::BrowserService, improved_phone_auth::ImprovedPhoneAuthService},
};
use anyhow::Result;
use async_trait::async_trait;
use chromiumoxide::page::Page;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Authentication service trait
#[async_trait]
pub trait AuthServiceTrait: Send + Sync {
    async fn is_authorized(&self) -> Result<bool>;
    async fn get_sender_id(&self) -> Result<Option<String>>;
    async fn get_auth_qr_code(&self) -> Result<String>;
    async fn login_with_phone_number(&self, phone_number: &str) -> Result<Option<String>>;
    async fn logout(&self) -> Result<()>;

    /// Improved phone authentication using MCP Playwright (Phase 2 implementation)
    async fn login_with_phone_number_improved(&self, phone_number: &str) -> Result<Option<String>>;

    /// Compare old vs new phone authentication (for testing and migration)
    async fn compare_phone_auth_implementations(&self, phone_number: &str) -> Result<()>;
}

/// WhatsApp authentication service
pub struct AuthService {
    _config: Arc<AppConfig>,
    browser_service: Arc<BrowserService>,
}

impl AuthService {
    pub fn new(config: Arc<AppConfig>, browser_service: Arc<BrowserService>) -> Self {
        Self {
            _config: config,
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

        info!("Starting phone authentication for: {}", formatted_phone);

        // Debug: Log current page state
        let page_title = page.get_title().await.unwrap_or_default().unwrap_or_default();
        let page_url = page.url().await.unwrap_or_default().unwrap_or_else(|| "unknown".to_string());
        debug!("Current page - Title: '{}', URL: '{}'", page_title, page_url);

        // Wait for page to fully load
        debug!("Waiting for page to fully load...");
        tokio::time::sleep(std::time::Duration::from_millis(3000)).await;

        // Check what's currently visible on the page
        let has_qr_login = page.find_element("text='Log into WhatsApp Web'").await.is_ok();
        let has_phone_login = page.find_element("text='Enter phone number'").await.is_ok();
        let has_code_screen = page.find_element("text='Enter code on phone'").await.is_ok();
        
        debug!("Page state - QR login: {}, Phone login: {}, Code screen: {}", 
               has_qr_login, has_phone_login, has_code_screen);

        // Switch to phone number login if we're in QR mode
        if has_qr_login {
            debug!("Found QR login screen, switching to phone number login");
            if let Ok(phone_link) = page.find_element("text='Log in with phone number'").await {
                info!("Clicking 'Log in with phone number' link");
                phone_link.click().await?;
                
                // Wait for phone input to be visible with extended timeout
                tokio::time::timeout(
                    std::time::Duration::from_millis(15000),
                    async {
                        let mut attempts = 0;
                        while page.find_element("text='Enter phone number'").await.is_err() && attempts < 30 {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            attempts += 1;
                            if attempts % 5 == 0 {
                                debug!("Still waiting for phone input screen... (attempt {})", attempts);
                                // Debug: Check what's on the page now
                                let current_content = page.content().await.unwrap_or_default();
                                if current_content.contains("Enter phone number") {
                                    debug!("Phone input screen detected in content");
                                    break;
                                }
                            }
                        }
                    }
                ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for phone input screen - QR code may still be loading"))?;
            } else {
                return Err(anyhow::anyhow!("Could not find 'Log in with phone number' link"));
            }
        }

        // Enter phone number
        let phone_entered = if page.find_element("text='Enter phone number'").await.is_ok() {
            debug!("Found phone number input screen, entering: {}", formatted_phone);
            
            // Wait for the input field to be ready
            let phone_input = tokio::time::timeout(
                std::time::Duration::from_millis(10000),
                async {
                    loop {
                        if let Ok(input) = page.find_element("[aria-label='Type your phone number.']").await {
                            debug!("Found phone input field");
                            return Ok::<_, anyhow::Error>(input);
                        }
                        // Also try alternative selectors
                        if let Ok(input) = page.find_element("input[type='tel']").await {
                            debug!("Found tel input field");
                            return Ok::<_, anyhow::Error>(input);
                        }
                        if let Ok(input) = page.find_element("input[placeholder*='phone']").await {
                            debug!("Found input with phone placeholder");
                            return Ok::<_, anyhow::Error>(input);
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    }
                }
            ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for phone input field"))??;
            
            // Clear and fill the input
            phone_input.click().await?;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            
            // Try multiple methods to clear and enter the phone number
            let _ = page.evaluate("document.querySelector('[aria-label=\"Type your phone number.\"]').value = '';").await;
            let _ = phone_input.press_key("Control+A").await;
            let _ = phone_input.press_key("Delete").await;
            
            phone_input.type_str(&formatted_phone).await?;
            info!("Phone number entered: {}", formatted_phone);
            
            // Wait a moment for the input to register
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            
            // Click Next button with multiple attempts
            let mut next_clicked = false;
            for attempt in 1..=5 {
                if let Ok(submit_button) = page.find_element("text='Next'").await {
                    debug!("Clicking Next button (attempt {})", attempt);
                    submit_button.click().await?;
                    next_clicked = true;
                    break;
                } else if let Ok(submit_button) = page.find_element("button[type='submit']").await {
                    debug!("Clicking submit button (attempt {})", attempt);
                    submit_button.click().await?;
                    next_clicked = true;
                    break;
                } else if let Ok(submit_button) = page.find_element("button").await {
                    if let Ok(Some(button_text)) = submit_button.inner_text().await {
                        if button_text.contains("Next") || button_text.contains("Continue") {
                            debug!("Clicking button with text '{}' (attempt {})", button_text, attempt);
                            submit_button.click().await?;
                            next_clicked = true;
                            break;
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            
            if !next_clicked {
                return Err(anyhow::anyhow!("Could not find or click Next button"));
            }
            
            true
        } else {
            debug!("Not on phone number input screen");
            false
        };

        // Wait for code screen to appear with extended timeout and better debugging
        debug!("Waiting for code input screen...");
        let code_screen_found = tokio::time::timeout(
            std::time::Duration::from_millis(45000), // Increased to 45 seconds
            async {
                let mut attempts = 0;
                while attempts < 90 {
                    // Check multiple possible selectors for the code screen
                    let has_code_label = page.find_element("[aria-label='Enter code on phone:']").await.is_ok();
                    let has_code_text = page.find_element("text='Enter code on phone'").await.is_ok();
                    let has_link_device_text = page.find_element("text='Link a device'").await.is_ok();
                    let has_code_element = page.find_element("[aria-details='link-device-phone-number-code-screen-instructions']").await.is_ok();
                    
                    // Additional checks for code display patterns discovered through testing
                    let has_code_container = page.find_element("div[data-link-code]").await.is_ok();
                    let has_verification_text = page.find_element("text='verification'").await.is_ok();
                    let has_digits_pattern = page.find_element("div > div > div").await.is_ok() && {
                        // Check if we can find individual character elements
                        let content = page.content().await.unwrap_or_default();
                        content.contains("Verify") || content.contains("code") || content.contains("device")
                    };
                    
                    // Check URL change that might indicate we're on code screen
                    let current_url = page.url().await.unwrap_or_default().unwrap_or_default();
                    let url_indicates_code_screen = current_url.contains("code") || current_url.contains("link");
                    
                    if has_code_label || has_code_text || has_link_device_text || has_code_element || 
                       has_code_container || has_verification_text || has_digits_pattern || url_indicates_code_screen {
                        debug!("Code screen found! (code_label: {}, code_text: {}, link_device: {}, code_element: {}, code_container: {}, verification_text: {}, digits_pattern: {}, url_change: {})", 
                               has_code_label, has_code_text, has_link_device_text, has_code_element, 
                               has_code_container, has_verification_text, has_digits_pattern, url_indicates_code_screen);
                        return true;
                    }
                    
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    attempts += 1;
                    
                    if attempts % 10 == 0 {
                        debug!("Still waiting for code input screen... (attempt {})", attempts);
                        
                        // More detailed debugging every 20 attempts (10 seconds)
                        if attempts % 20 == 0 {
                            let current_url = page.url().await.unwrap_or_default().unwrap_or_default();
                            debug!("Current URL: {}", current_url);
                            
                            // Sample page content for debugging
                            let page_content = page.content().await.unwrap_or_default();
                            let content_sample = if page_content.len() > 500 {
                                format!("{}...", &page_content[..500])
                            } else {
                                page_content.clone()
                            };
                            debug!("Page content sample: {}", content_sample);
                        }
                        
                        // Check if we're still on the phone number screen
                        if page.find_element("text='Enter phone number'").await.is_ok() {
                            debug!("Still on phone number screen - may need to retry submission");
                            if let Ok(submit_button) = page.find_element("text='Next'").await {
                                info!("Retrying Next button click");
                                let _ = submit_button.click().await;
                                tokio::time::sleep(std::time::Duration::from_millis(2000)).await; // Wait longer after retry
                            }
                        }
                        
                        // Check for error messages
                        if let Ok(error_element) = page.find_element("[role='alert']").await {
                            if let Ok(Some(error_text)) = error_element.inner_text().await {
                                if !error_text.is_empty() {
                                    debug!("Found error message: {}", error_text);
                                }
                            }
                        }
                        
                        // Check for common error indicators
                        let page_content = page.content().await.unwrap_or_default();
                        if page_content.contains("Enter code") || page_content.contains("verification") {
                            debug!("Code screen text found in page content");
                        } else if page_content.contains("invalid") || page_content.contains("error") || page_content.contains("not found") {
                            debug!("Error detected in page content");
                        } else if page_content.contains("phone number") {
                            debug!("Still appears to be on phone number screen");
                        }
                    }
                }
                false
            }
        ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for code input screen - phone number may be invalid or network issues"))?;

        if !code_screen_found {
            return Err(anyhow::anyhow!("Code input screen never appeared"));
        }

        // Wait for link code element to appear and extract code with better error handling
        debug!("Waiting for link code element...");
        let link_code = tokio::time::timeout(
            std::time::Duration::from_millis(15000), // 15 seconds
            async {
                let mut attempts = 0;
                while attempts < 30 {
                    // Method 1: Look for the code displayed as individual characters
                    let code_extraction_script = r#"
                        (function() {
                            // Method 1: Look for the specific pattern where code is displayed after "Enter code on phone"
                            const codeContainer = document.evaluate(
                                "//div[contains(text(), 'Enter code on phone')]/following-sibling::div",
                                document,
                                null,
                                XPathResult.FIRST_ORDERED_NODE_TYPE,
                                null
                            ).singleNodeValue;
                            
                            if (codeContainer) {
                                // Look for child elements that contain individual characters
                                const children = Array.from(codeContainer.querySelectorAll('div'));
                                let codeChars = [];
                                
                                for (let child of children) {
                                    const text = child.textContent?.trim();
                                    // Look for single characters, numbers, or dashes that form the code
                                    if (text && text.length <= 2 && text.match(/[A-Z0-9-]/)) {
                                        codeChars.push(text);
                                    }
                                }
                                
                                // If we found enough characters (typically 8-9 including dash), join them
                                if (codeChars.length >= 6) {
                                    return codeChars.join('');
                                }
                            }
                            
                            // Method 2: Look for data-link-code attribute
                            const linkCodeElement = document.querySelector('[data-link-code]');
                            if (linkCodeElement) {
                                const linkCode = linkCodeElement.getAttribute('data-link-code');
                                if (linkCode && linkCode.length >= 6) {
                                    return linkCode;
                                }
                            }
                            
                            // Method 3: Look for specific code display containers
                            const codeElements = document.querySelectorAll('div[aria-details*="code"], div[role*="code"], div[class*="code"]');
                            for (let element of codeElements) {
                                const text = element.textContent?.trim();
                                if (text && text.match(/^[A-Z0-9]{3,4}[-]?[A-Z0-9]{3,4}$/)) {
                                    return text;
                                }
                            }
                            
                            // Method 4: Look for patterns in all divs that might contain individual characters
                            const allDivs = document.querySelectorAll('div');
                            let potentialCodeChars = [];
                            
                            for (let div of allDivs) {
                                const text = div.textContent?.trim();
                                if (text && text.length === 1 && text.match(/[A-Z0-9]/)) {
                                    // Check if this div is part of a sequence
                                    const parent = div.parentElement;
                                    if (parent) {
                                        const siblings = Array.from(parent.children);
                                        const charSequence = siblings.map(s => s.textContent?.trim()).filter(t => t && t.length <= 2);
                                        if (charSequence.length >= 6) {
                                            return charSequence.join('');
                                        }
                                    }
                                }
                            }
                            
                            // Method 5: Fallback - Look for the pattern directly in the page text
                            const bodyText = document.body.textContent || '';
                            // Match patterns like "1KBB-PEVN" (4 chars, dash, 4 chars)
                            const codeMatch = bodyText.match(/\b[A-Z0-9]{3,4}[-][A-Z0-9]{3,4}\b/);
                            if (codeMatch) {
                                return codeMatch[0];
                            }
                            
                            // Method 6: Another pattern - just alphanumeric codes
                            const simpleCodeMatch = bodyText.match(/\b[A-Z0-9]{6,9}\b/);
                            if (simpleCodeMatch && !simpleCodeMatch[0].match(/^\d+$/)) {
                                // Exclude pure numbers (like phone numbers)
                                return simpleCodeMatch[0];
                            }
                            
                            return null;
                        })()
                    "#;
                    
                    if let Ok(result) = page.evaluate(code_extraction_script).await {
                        if let Ok(value) = result.into_value::<serde_json::Value>() {
                            if let Some(code_str) = value.as_str() {
                                if !code_str.is_empty() && code_str != "null" && code_str.len() >= 4 {
                                    let formatted_code = code_str.replace(" ", "");
                                    info!("Phone authentication code found via character extraction: {}", formatted_code);
                                    return Ok::<Option<String>, anyhow::Error>(Some(formatted_code));
                                }
                            }
                        }
                    }
                    
                    // Method 2: Using the data-link-code attribute (legacy support)
                    let locators = LocatorDictionary::new();
                    if let Ok(Some(code)) = locators.data_link_code(&page).await {
                        let formatted_code = code.replace(",", "").replace(" ", "");
                        if !formatted_code.is_empty() && formatted_code.len() >= 4 {
                            info!("Phone authentication code found via data-link-code: {}", formatted_code);
                            return Ok::<Option<String>, anyhow::Error>(Some(formatted_code));
                        }
                    }
                    
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    attempts += 1;
                    
                    if attempts % 6 == 0 {
                        debug!("Still waiting for link code element... (attempt {})", attempts);
                        
                        // Debug: Log what we can see on the page
                        let page_content = page.content().await.unwrap_or_default();
                        if page_content.contains("code") {
                            debug!("Page contains 'code' text");
                        }
                        if page_content.contains("device") {
                            debug!("Page contains 'device' text");
                        }
                    }
                }
                Ok::<Option<String>, anyhow::Error>(None)
            }
        ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for link code element - please check your phone for WhatsApp notifications"))??;

        match link_code {
            Some(code) => {
                info!("Successfully extracted phone authentication code: {}", code);
                Ok(Some(code))
            },
            None => {
                debug!("Could not extract authentication code from page");
                Ok(None)
            }
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

    /// Improved phone authentication using MCP Playwright (Phase 2 implementation)
    async fn login_with_phone_number_improved(&self, phone_number: &str) -> Result<Option<String>> {
        info!("🔧 Using improved phone authentication service");
        
        let improved_service = ImprovedPhoneAuthService::new();
        
        match improved_service.authenticate_with_phone(phone_number).await {
            Ok(result) => {
                if result.success {
                    info!("✅ Improved phone auth successful: {:?}", result.verification_code);
                    info!("📊 Debug steps: {:?}", result.debug_info.steps_completed);
                    Ok(result.verification_code)
                } else {
                    warn!("❌ Improved phone auth failed: {:?}", result.error_message);
                    warn!("📊 Debug info: {:?}", result.debug_info);
                    Err(anyhow::anyhow!("Phone authentication failed: {:?}", result.error_message))
                }
            }
            Err(e) => {
                warn!("❌ Improved phone auth service error: {}", e);
                Err(e)
            }
        }
    }

    /// Compare old vs new phone authentication (for testing and migration)
    async fn compare_phone_auth_implementations(&self, phone_number: &str) -> Result<()> {
        info!("🔍 Comparing old vs new phone authentication implementations");
        
        // Test improved implementation
        let improved_start = std::time::Instant::now();
        let improved_result = self.login_with_phone_number_improved(phone_number).await;
        let improved_duration = improved_start.elapsed();
        
        // Test original implementation
        let original_start = std::time::Instant::now();
        let original_result = self.login_with_phone_number(phone_number).await;
        let original_duration = original_start.elapsed();
        
        // Compare results
        info!("📊 AUTHENTICATION COMPARISON RESULTS:");
        info!("   Improved: {:?} (took {:?})", 
              improved_result.as_ref().map(|r| r.as_ref().map(|s| s.as_str())), 
              improved_duration);
        info!("   Original: {:?} (took {:?})", 
              original_result.as_ref().map(|r| r.as_ref().map(|s| s.as_str())), 
              original_duration);
        
        match (improved_result.is_ok(), original_result.is_ok()) {
            (true, true) => info!("✅ Both implementations succeeded"),
            (true, false) => warn!("⚠️ Only improved implementation succeeded"),
            (false, true) => warn!("⚠️ Only original implementation succeeded"),
            (false, false) => warn!("❌ Both implementations failed"),
        }
        
        Ok(())
    }
}
