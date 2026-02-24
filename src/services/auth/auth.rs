//! Authentication Service
//!
//! Handles WhatsApp Web authentication including QR code and phone number methods.

use crate::{
    browser::{country_codes, BrowserService, Locators},
    config::AppConfig,
};
use anyhow::Result;
use async_trait::async_trait;
use chromiumoxide::page::Page;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

// ============================================================================
// Types
// ============================================================================

/// Authentication check result with status information
#[derive(Debug, Clone)]
pub struct AuthCheckResult {
    /// Whether the user is authorized
    pub authorized: bool,
    /// Status reason: "authenticated", "not_authenticated", "checking"
    pub status: String,
}

impl AuthCheckResult {
    pub fn authenticated() -> Self {
        Self {
            authorized: true,
            status: "authenticated".to_string(),
        }
    }

    pub fn not_authenticated() -> Self {
        Self {
            authorized: false,
            status: "not_authenticated".to_string(),
        }
    }

    pub fn checking() -> Self {
        Self {
            authorized: false,
            status: "checking".to_string(),
        }
    }
}

// ============================================================================
// Trait Definition
// ============================================================================

/// Authentication service trait
#[async_trait]
pub trait AuthServiceTrait: Send + Sync {
    /// Check if user is authorized (logged in) - returns detailed status
    async fn check_auth_status(&self) -> Result<AuthCheckResult>;

    /// Check if user is authorized (simple bool for compatibility)
    async fn is_authorized(&self) -> Result<bool>;

    /// Get the sender's phone number/ID
    async fn get_sender_id(&self) -> Result<Option<String>>;

    /// Get QR code for authentication
    async fn get_auth_qr_code(&self) -> Result<String>;

    /// Authenticate using phone number
    async fn login_with_phone_number(&self, phone_number: &str) -> Result<Option<String>>;

    /// Logout from WhatsApp Web
    async fn logout(&self) -> Result<()>;
}

// ============================================================================
// Configuration
// ============================================================================

/// Timeout configuration for authentication operations
#[derive(Debug, Clone)]
pub struct AuthTimeouts {
    pub navigation: Duration,
    pub element_wait: Duration,
    pub code_detection: Duration,
    pub total_operation: Duration,
}

impl Default for AuthTimeouts {
    fn default() -> Self {
        Self {
            navigation: Duration::from_secs(15),
            element_wait: Duration::from_secs(10),
            code_detection: Duration::from_secs(30),
            total_operation: Duration::from_secs(60),
        }
    }
}

// ============================================================================
// Service Implementation
// ============================================================================

/// WhatsApp authentication service
pub struct AuthService {
    #[allow(dead_code)]
    config: Arc<AppConfig>,
    browser_service: Arc<BrowserService>,
    timeouts: AuthTimeouts,
}

impl AuthService {
    pub fn new(config: Arc<AppConfig>, browser_service: Arc<BrowserService>) -> Self {
        Self {
            config,
            browser_service,
            timeouts: AuthTimeouts::default(),
        }
    }

    /// Get page from browser service
    async fn get_page(&self) -> Result<Page> {
        self.browser_service
            .get_or_create_page("https://web.whatsapp.com")
            .await
    }

    /// Wait for element with timeout
    async fn wait_for_element(&self, page: &Page, selector: &str, timeout_ms: u64) -> bool {
        Locators::wait_for(page, selector, timeout_ms).await
    }

    /// Extract QR code from canvas
    async fn extract_qr_code(&self, page: &Page) -> Result<String> {
        // Wait for QR code canvas
        page.find_element("canvas").await?;

        // Extract QR code from canvas
        let canvas_result = page
            .evaluate("document.getElementsByTagName('canvas')[0].toDataURL('image/png');")
            .await?;

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

        Ok(parts[1].to_string())
    }

    /// Format phone number for WhatsApp
    fn format_phone_number(&self, phone: &str) -> String {
        let cleaned: String = phone
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '+')
            .collect();

        if !cleaned.starts_with('+') && cleaned.len() >= 10 {
            format!("+{}", cleaned)
        } else {
            cleaned
        }
    }

    /// Extract verification code from page
    async fn extract_verification_code(&self, page: &Page) -> Result<Option<String>> {
        let code_script = r#"
            (function() {
                // Method 1: data-link-code attribute
                const linkCodeEl = document.querySelector('[data-link-code]');
                if (linkCodeEl) {
                    const code = linkCodeEl.getAttribute('data-link-code');
                    if (code && code.length >= 6) return code;
                }
                
                // Method 2: Look for code pattern in body text
                const bodyText = document.body.textContent || '';
                const codeMatch = bodyText.match(/\b[A-Z0-9]{3,4}[-][A-Z0-9]{3,4}\b/);
                if (codeMatch) return codeMatch[0];
                
                // Method 3: Simple alphanumeric pattern
                const simpleMatch = bodyText.match(/\b[A-Z0-9]{6,9}\b/);
                if (simpleMatch && !simpleMatch[0].match(/^\d+$/)) {
                    return simpleMatch[0];
                }
                
                return null;
            })()
        "#;

        let result = page.evaluate(code_script).await?;
        if let Ok(value) = result.into_value::<serde_json::Value>() {
            if let Some(code_str) = value.as_str() {
                if !code_str.is_empty() && code_str != "null" && code_str.len() >= 4 {
                    return Ok(Some(code_str.replace(" ", "")));
                }
            }
        }
        Ok(None)
    }
}

#[async_trait]
impl AuthServiceTrait for AuthService {
    async fn check_auth_status(&self) -> Result<AuthCheckResult> {
        let page = self.get_page().await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Ensure we're on WhatsApp Web, not some redirect
        if !Locators::ensure_whatsapp_page(&page).await {
            return Err(anyhow::anyhow!("Failed to navigate to WhatsApp Web"));
        }

        let check_js = r##"
            (() => {
                // Check if we're still loading (progress bar visible)
                const isLoading = document.querySelector('progress[max="100"]') !== null
                    || document.body.innerText.includes('Loading your chats');
                
                if (isLoading) {
                    return { authorized: false, reason: 'loading' };
                }
                
                // Check for authenticated state (chat list pane visible)
                const paneExists = document.querySelector('#pane-side') !== null 
                    || document.querySelector('[data-testid="chat-list"]') !== null
                    || document.querySelector('div[aria-label="Chat list"]') !== null;
                
                if (paneExists) {
                    return { authorized: true, reason: 'pane_visible' };
                }
                
                // Check for QR code login screen
                const qrCodeVisible = document.querySelector("canvas[aria-label='Scan this QR code to link a device!']") !== null
                    || document.querySelector('canvas[aria-label="Scan me!"]') !== null;
                
                // Check for general login screen indicators
                const loginScreen = qrCodeVisible
                    || document.body.innerText.includes('Log into WhatsApp Web')
                    || document.body.innerText.includes('Use WhatsApp on your computer')
                    || document.body.innerText.includes('Link with phone number');
                
                // Check for phone number entry
                const phoneEntry = document.body.innerText.includes('Enter phone number')
                    || document.querySelector('input[aria-label="Type your phone number to log in to WhatsApp"]') !== null;
                
                // Check for pairing code entry
                const codeEntry = document.body.innerText.includes('Enter code on phone')
                    || document.querySelector('[data-testid="link-device-phone-number-code-entry"]') !== null
                    || document.querySelector('[aria-details="link-device-phone-number-code-screen-instructions"]') !== null;
                
                if (loginScreen || phoneEntry || codeEntry) {
                    return { authorized: false, reason: qrCodeVisible ? 'login' : phoneEntry ? 'phone' : codeEntry ? 'code' : 'login' };
                }
                
                return { authorized: false, reason: 'unclear' };
            })()
        "##;

        let result = page.evaluate(check_js).await?;
        let value: serde_json::Value = result.into_value()?;

        let authorized = value
            .get("authorized")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let reason = value
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        debug!(
            "Authorization check: authorized={}, reason={}",
            authorized, reason
        );

        // Map the reason to a user-friendly status
        let status = match (authorized, reason) {
            (true, _) => "authenticated",
            (false, "login") | (false, "phone") | (false, "code") => "not_authenticated",
            (false, "unclear") | (false, _) => "checking",
        };

        Ok(AuthCheckResult {
            authorized,
            status: status.to_string(),
        })
    }

    async fn is_authorized(&self) -> Result<bool> {
        let result = self.check_auth_status().await?;
        Ok(result.authorized)
    }

    async fn get_sender_id(&self) -> Result<Option<String>> {
        let page = self.get_page().await?;

        if !self.is_authorized().await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        let sender_result = page.evaluate(
            "window.localStorage.getItem('last-wid') || window.localStorage.getItem('last-wid-md') || '';"
        ).await?;

        let sender_string = match sender_result.into_value()? {
            serde_json::Value::String(s) => s,
            _ => String::new(),
        }
        .trim_matches('"')
        .to_string();

        if sender_string.is_empty() {
            return Ok(None);
        }

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

        // Check if we need to switch to QR code mode (currently on phone login screen)
        if Locators::exists(&page, Locators::phone_number_label()).await
            || Locators::exists(&page, "text:Enter code on phone").await
        {
            debug!("Switching to QR code login");
            Locators::click(&page, Locators::qr_auth_link()).await?;
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }

        // Wait for QR code to be visible
        if !self
            .wait_for_element(&page, Locators::qr_code_canvas(), 20000)
            .await
        {
            // Check if we need to reload the QR code
            let _ = Locators::click(&page, Locators::config().auth.qr_reload_button.as_str()).await;
        }

        // Wait again after potential reload
        if !self
            .wait_for_element(&page, Locators::qr_code_canvas(), 15000)
            .await
        {
            return Err(anyhow::anyhow!("QR code not available"));
        }

        self.extract_qr_code(&page).await
    }

    async fn login_with_phone_number(&self, phone_number: &str) -> Result<Option<String>> {
        let page = self.get_page().await?;

        if self.is_authorized().await? {
            return Err(anyhow::anyhow!("Already authorized"));
        }

        let formatted_phone = self.format_phone_number(phone_number);
        info!("Starting phone authentication for: {}", formatted_phone);

        // Ensure we're on WhatsApp Web (not a download/redirect page)
        if !Locators::ensure_whatsapp_page(&page).await {
            return Err(anyhow::anyhow!("Failed to navigate to WhatsApp Web"));
        }

        // // Wait for page to load
        // tokio::time::sleep(Duration::from_millis(3000)).await;

        // // Diagnose page state for debugging
        // let diag = Locators::diagnose_page(&page).await;
        // info!("Page state before login: {}", diag);

        // Check current screen state — detect QR code / initial landing screen
        let on_qr_screen = Locators::exists(&page, Locators::login_label()).await
            || Locators::exists(&page, Locators::qr_code_canvas()).await;

        if !on_qr_screen {
            let diag = Locators::diagnose_page(&page).await;
            info!("Not on QR/login screen, diagnosing: {}", diag);
        }

        // Switch to phone number login if on QR screen
        if on_qr_screen {
            info!("Switching to phone number login");
            if Locators::click(&page, "[role='button'] >> text:Log in with phone number").await? {
                // Wait for phone input screen
                if !Locators::wait_for(&page, "text:Enter phone number", self.timeouts.element_wait.as_millis() as u64).await {
                    let diag = Locators::diagnose_page(&page).await;
                    return Err(anyhow::anyhow!("Timeout waiting for phone input screen. Page state: {}", diag));
                }
            }
        }

        // Enter phone number — fill the phone number w/o country code (e.g. "9501005734") into the input.
        // WhatsApp auto-detects the country from the number, no dropdown manipulation needed.
        if Locators::exists(&page, "text:Enter phone number").await {
            // Read the currently selected country code from the phone input's value.
            // When India is selected, input value = "+91"; US = "+1", etc.
            let selected_code = page.evaluate(
                r#"(function() {
                    var input = document.querySelector("[aria-label*='phone number']")
                        || document.querySelector("input[type='text']");
                    if (!input || !input.value) return '';
                    var m = input.value.match(/^\+(\d+)/);
                    return m ? m[1] : '';
                })()"#
            ).await.ok().and_then(|v| v.into_value::<String>().ok()).unwrap_or_default();

            let (phone_country, national_number) = country_codes::parse_phone(&formatted_phone);

            // If the dropdown already shows the same country code, enter just the national number.
            // Otherwise, enter the full number with + prefix so WhatsApp auto-switches country.
            let value_to_enter = if !selected_code.is_empty() && selected_code == phone_country.dial_code {
                info!("Dropdown already shows +{}, entering national number: {}", selected_code, national_number);
                national_number.clone()
            } else {
                info!("Dropdown shows +{}, phone needs +{}, entering full number: {}", selected_code, phone_country.dial_code, formatted_phone);
                formatted_phone.clone()
            };

            // Set the phone number value
            if !Locators::set_phone_input_value(&page, Locators::phone_input(), &value_to_enter)
                .await?
            {
                // Fallback: find input element directly and type
                info!("JS input setter failed, falling back to element typing");
                let phone_input = tokio::time::timeout(self.timeouts.element_wait, async {
                    loop {
                        if let Ok(input) = page.find_element(Locators::phone_input()).await {
                            return Ok::<_, anyhow::Error>(input);
                        }
                        if let Ok(input) = page.find_element("input[type='text']").await {
                            return Ok(input);
                        }
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }
                })
                .await
                .map_err(|_| anyhow::anyhow!("Phone input not found"))??;

                phone_input.click().await?;
                tokio::time::sleep(Duration::from_millis(300)).await;
                let _ = phone_input.press_key("Meta+A").await;
                let _ = phone_input.press_key("Delete").await;
                phone_input.type_str(&value_to_enter).await?;
            }

            tokio::time::sleep(Duration::from_millis(1000)).await;

            // Click Next button
            let next_clicked = Locators::click(&page, Locators::phone_submit_button()).await?;
            if !next_clicked {
                // Fallback: try clicking any button containing "Next" text
                let next_js = r#"(function() {
                    var els = document.querySelectorAll('button, [role="button"]');
                    for (var i = 0; i < els.length; i++) {
                        if ((els[i].textContent || '').trim() === 'Next') {
                            els[i].scrollIntoView({block:'center'});
                            els[i].click();
                            return true;
                        }
                    }
                    return false;
                })()"#;
                let fb = page.evaluate(next_js).await
                    .ok().and_then(|v| v.into_value::<bool>().ok()).unwrap_or(false);
                if !fb {
                    let diag = Locators::diagnose_page(&page).await;
                    return Err(anyhow::anyhow!("Could not find Next button. Page: {}", diag));
                }
            }
            info!("Clicked Next, waiting for verification code...");
        }

        // Wait for code screen
        debug!("Waiting for verification code screen...");
        let code_found = tokio::time::timeout(self.timeouts.code_detection, async {
            loop {
                let content = page.content().await.unwrap_or_default();
                if content.contains("Enter code") || content.contains("verification") {
                    return true;
                }
                if page.find_element(Locators::phone_code()).await.is_ok() {
                    return true;
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        })
        .await
        .unwrap_or(false);

        if !code_found {
            return Err(anyhow::anyhow!("Code screen not found"));
        }

        // Extract verification code
        let code = tokio::time::timeout(self.timeouts.element_wait, async {
            loop {
                if let Ok(Some(code)) = self.extract_verification_code(&page).await {
                    return Some(code);
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        })
        .await
        .ok()
        .flatten();

        if let Some(ref c) = code {
            info!("Verification code extracted: {}", c);
        }

        Ok(code)
    }

    async fn logout(&self) -> Result<()> {
        let page = self.get_page().await?;

        if !self.is_authorized().await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        debug!("Logging out");

        // Click menu
        if let Ok(menu) = page.find_element(Locators::menu_button()).await {
            menu.click().await?;
        }

        // Click logout
        tokio::time::sleep(Duration::from_millis(500)).await;
        if let Ok(logout) = page.find_element(Locators::logout_button()).await {
            logout.click().await?;
        }

        info!("Logout completed");
        Ok(())
    }
}
