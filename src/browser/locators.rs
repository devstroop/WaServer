//! WhatsApp Web Element Locators
//!
//! Locators are loaded from `config/locators.toml` at runtime.
//! Update the TOML file when WhatsApp Web UI changes - no recompilation needed.
//! Falls back to built-in defaults if config file is missing.
//!
//! Selectors support extended prefixes (see selector.rs):
//!   text:   — exact text match (walks up to clickable ancestor)
//!   text*:  — partial text match
//!   role:   — ARIA role + optional name (role:button[Next])
//!   xpath:  — XPath
//!   (none)  — CSS selector (default)

use anyhow::{Context, Result};
use chromiumoxide::page::Page;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use super::country_codes::CountryInfo;
use super::selector::{parse_selector, selector_click_js, selector_exists_js};
use tracing::info;

// ============================================================================
// Timeout Constants
// ============================================================================

/// Centralized timeout constants (in milliseconds)
pub struct Timeouts;

impl Timeouts {
    /// Default element wait timeout
    pub const ELEMENT_WAIT_MS: u64 = 10_000;
    /// Short timeout for quick checks
    pub const SHORT_MS: u64 = 5_000;
    /// Long timeout for slow operations
    pub const LONG_MS: u64 = 30_000;
    /// Send message operation timeout
    pub const SEND_MESSAGE_MS: u64 = 60_000;
    /// QR code wait timeout
    pub const QR_CODE_MS: u64 = 20_000;
    /// Navigation timeout
    pub const NAVIGATION_MS: u64 = 15_000;
    /// Dialog dismiss timeout
    pub const DIALOG_MS: u64 = 10_000;
    /// Delay between queue messages (rate limiting)
    pub const QUEUE_DELAY_MS: u64 = 500;
    /// Polling interval for element checks
    pub const POLL_INTERVAL_MS: u64 = 200;
}

// ============================================================================
// TOML Configuration Types
// ============================================================================

/// Global cached locator config
static CONFIG: OnceLock<LocatorConfig> = OnceLock::new();

/// Root locator configuration (loaded from TOML)
#[derive(Debug, Clone, Deserialize)]
pub struct LocatorConfig {
    pub dialog: DialogLocators,
    pub auth: AuthLocators,
    pub menu: MenuLocators,
    pub loading: LoadingLocators,
    pub chat: ChatLocators,
    pub attachment: AttachmentLocators,
    #[serde(default)]
    pub scripts: ScriptLocators,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DialogLocators {
    pub root: String,
    pub backdrop: String,
    pub popup: String,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthLocators {
    pub login_with_phone_link: String,
    pub login_with_qr_link: String,
    pub login_label: String,
    pub phone_number_label: String,
    pub phone_number_input: String,
    pub phone_submit_button: String,
    pub invalid_phone_dialog: String,
    pub code_on_phone_label: String,
    pub code_on_phone_value: String,
    pub link_code_element: String,
    pub link_code_digits: String,
    pub qr_loading: String,
    pub qr_canvas: String,
    pub qr_reload_button: String,
    pub authorized_pane: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MenuLocators {
    pub button: String,
    pub dropdown: String,
    pub logout: String,
    pub logout_confirm: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoadingLocators {
    pub progress: String,
    pub phone_loader_parent: String,
    pub phone_loader: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatLocators {
    pub message_input: String,
    pub message_contenteditable: String,
    pub send_button: String,
    pub send_button_parent: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttachmentLocators {
    pub button: String,
    pub plus_icon: String,
    pub menu_plus_icon: String,
    pub photo_video_input: String,
    pub caption_input: String,
    pub document_input: String,
    pub send_button: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ScriptLocators {
    #[serde(default = "default_qr_script")]
    pub qr_code_base64: String,
}

fn default_qr_script() -> String {
    r#"(function() {
    var canvas = document.querySelector("canvas[aria-label='Scan this QR code to link a device!']");
    if (canvas) {
        return canvas.toDataURL('image/png').split(',')[1];
    }
    return null;
})();"#
        .to_string()
}

impl LocatorConfig {
    /// Load locators from TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read locators from {:?}", path.as_ref()))?;
        toml::from_str(&content).context("Failed to parse locators TOML")
    }

    /// Load from default path (config/locators.toml)
    pub fn load_default() -> Result<Self> {
        Self::load("config/locators.toml")
    }

    /// Get or initialize the global locator config
    pub fn global() -> &'static Self {
        CONFIG.get_or_init(|| {
            Self::load_default().unwrap_or_else(|e| {
                tracing::warn!("Failed to load locators.toml, using defaults: {}", e);
                Self::defaults()
            })
        })
    }

    /// Built-in defaults (fallback when config file is missing)
    pub fn defaults() -> Self {
        Self {
            dialog: DialogLocators {
                root: "[role='dialog']".into(),
                backdrop: "div[data-animate-modal-backdrop='true']".into(),
                popup: "div[data-animate-modal-popup='true']".into(),
                body: "div[data-animate-modal-body='true']".into(),
            },
            auth: AuthLocators {
                login_with_phone_link: "[role='button'] >> text:Log in with phone number".into(),
                login_with_qr_link: "[role='button'] >> text:Log in with QR code".into(),
                login_label: "text:Log into WhatsApp Web".into(),
                phone_number_label: "text:Enter phone number".into(),
                phone_number_input: "[aria-label='Type your phone number to log in to WhatsApp']".into(),
                phone_submit_button: "[role='button'] >> text:Next".into(),
                invalid_phone_dialog:
                    "#app div[data-animate-modal-popup='true'] div[data-animate-modal-body='true']"
                        .into(),
                code_on_phone_label: "text='Enter code on phone'".into(),
                code_on_phone_value:
                    "[aria-details='link-device-phone-number-code-screen-instructions']".into(),
                link_code_element:
                    "[aria-details='link-device-phone-number-code-screen-instructions']".into(),
                link_code_digits: "[data-link-code]".into(),
                qr_loading: "svg[role='status']".into(),
                qr_canvas: "canvas[aria-label='Scan this QR code to link a device!']".into(),
                qr_reload_button: "[data-icon='refresh-large']".into(),
                authorized_pane: "#pane-side".into(),
            },
            menu: MenuLocators {
                button: "button[title='Menu']".into(),
                dropdown: "[aria-label='Menu']".into(),
                logout: "[aria-label='Log out']".into(),
                logout_confirm: "[aria-label='Log out?']".into(),
            },
            loading: LoadingLocators {
                progress: "progress[max='100']".into(),
                phone_loader_parent: "#phoneLoaderParent".into(),
                phone_loader: "#phoneLoader".into(),
            },
            chat: ChatLocators {
                message_input: "#app #main footer div[aria-placeholder='Type a message']".into(),
                message_contenteditable:
                    "div[contenteditable='true'][aria-placeholder='Type a message']".into(),
                send_button: "span[data-icon='send']".into(),
                send_button_parent: "button span[data-icon='send']".into(),
            },
            attachment: AttachmentLocators {
                button: "button[title='Attach']".into(),
                plus_icon: "[data-icon='plus']".into(),
                menu_plus_icon: "[data-icon='attach-menu-plus']".into(),
                photo_video_input: "input[accept='image/*,video/mp4,video/3gpp,video/quicktime']"
                    .into(),
                caption_input: "#app div[aria-placeholder='Add a caption']".into(),
                document_input: "input[accept='*']".into(),
                send_button: "#app div[aria-label='Send']".into(),
            },
            scripts: ScriptLocators {
                qr_code_base64: default_qr_script(),
            },
        }
    }
}

// ============================================================================
// Static Locator Access (convenience methods using global config)
// ============================================================================

/// WhatsApp Web element locators - loads from config/locators.toml
pub struct Locators;

impl Locators {
    /// Get the global config
    pub fn config() -> &'static LocatorConfig {
        LocatorConfig::global()
    }

    // ========================================
    // Authentication
    // ========================================

    pub fn qr_code_canvas() -> &'static str {
        &Self::config().auth.qr_canvas
    }

    pub fn phone_auth_link() -> &'static str {
        &Self::config().auth.login_with_phone_link
    }

    pub fn qr_auth_link() -> &'static str {
        &Self::config().auth.login_with_qr_link
    }

    pub fn login_label() -> &'static str {
        &Self::config().auth.login_label
    }

    pub fn phone_number_label() -> &'static str {
        &Self::config().auth.phone_number_label
    }

    pub fn phone_submit_button() -> &'static str {
        &Self::config().auth.phone_submit_button
    }

    pub fn phone_input() -> &'static str {
        &Self::config().auth.phone_number_input
    }

    pub fn phone_code() -> &'static str {
        &Self::config().auth.link_code_element
    }

    pub fn side_pane() -> &'static str {
        &Self::config().auth.authorized_pane
    }

    pub fn loading_progress() -> &'static str {
        &Self::config().loading.progress
    }

    // ========================================
    // Menu & Navigation
    // ========================================

    pub fn menu_button() -> &'static str {
        &Self::config().menu.button
    }

    pub fn logout_button() -> &'static str {
        &Self::config().menu.logout
    }

    pub fn logout_dialog() -> &'static str {
        &Self::config().menu.logout_confirm
    }

    // ========================================
    // Chat & Messaging
    // ========================================

    pub fn message_input() -> &'static str {
        &Self::config().chat.message_input
    }

    pub fn send_button_icon() -> &'static str {
        &Self::config().chat.send_button
    }

    pub fn send_button() -> &'static str {
        &Self::config().chat.send_button_parent
    }

    // ========================================
    // Attachments
    // ========================================

    pub fn attach_button() -> &'static str {
        &Self::config().attachment.button
    }

    pub fn attach_plus_icon() -> &'static str {
        &Self::config().attachment.plus_icon
    }

    pub fn photo_video_input() -> &'static str {
        &Self::config().attachment.photo_video_input
    }

    pub fn document_input() -> &'static str {
        &Self::config().attachment.document_input
    }

    pub fn caption_input() -> &'static str {
        &Self::config().attachment.caption_input
    }

    pub fn attachment_send() -> &'static str {
        &Self::config().attachment.send_button
    }

    // ========================================
    // Dialogs
    // ========================================

    pub fn dialog() -> &'static str {
        &Self::config().dialog.root
    }

    pub fn modal_popup() -> &'static str {
        &Self::config().dialog.popup
    }

    pub fn modal_body() -> &'static str {
        &Self::config().dialog.body
    }

    pub fn invalid_phone_dialog() -> &'static str {
        &Self::config().auth.invalid_phone_dialog
    }

    // ========================================
    // Helper Methods (unified selector support)
    // ========================================

    /// Get QR code as base64 PNG
    pub async fn get_qr_code_base64(page: &Page) -> Result<Option<String>> {
        match page
            .evaluate(Self::config().scripts.qr_code_base64.as_str())
            .await
        {
            Ok(result) => Ok(result.into_value::<Option<String>>().unwrap_or(None)),
            Err(_) => Ok(None),
        }
    }

    /// Get phone authentication code
    pub async fn get_phone_code(page: &Page) -> Result<Option<String>> {
        match page.find_element(Self::phone_code()).await {
            Ok(element) => match element.attribute("data-link-code").await {
                Ok(value) => Ok(value),
                Err(_) => Ok(None),
            },
            Err(_) => Ok(None),
        }
    }

    /// Check if element/text exists on page.
    /// Supports all selector prefixes: text:, text*:, role:, xpath:, CSS.
    pub async fn exists(page: &Page, selector: &str) -> bool {
        let parsed = parse_selector(selector);

        // CSS selectors: use native chromiumoxide for speed
        if let Some(css) = parsed.as_css() {
            return page.find_element(css).await.is_ok();
        }

        // Extended selectors: use JS
        let js = selector_exists_js(&parsed);
        page.evaluate(js.as_str())
            .await
            .ok()
            .and_then(|r| r.into_value::<bool>().ok())
            .unwrap_or(false)
    }

    /// Click an element by any selector type.
    /// For CSS: native chromiumoxide click. For text/role/xpath: JS click with scrollIntoView.
    pub async fn click(page: &Page, selector: &str) -> Result<bool> {
        let parsed = parse_selector(selector);

        // CSS selectors: use native chromiumoxide
        if let Some(css) = parsed.as_css() {
            return match page.find_element(css).await {
                Ok(el) => {
                    el.click().await.ok();
                    Ok(true)
                }
                Err(_) => Ok(false),
            };
        }

        // Extended selectors: JS click
        let js = selector_click_js(&parsed);
        let result = page.evaluate(js.as_str()).await?;
        Ok(result.into_value::<bool>().unwrap_or(false))
    }

    /// Wait for an element/text to appear, with timeout.
    /// Supports all selector prefixes.
    pub async fn wait_for(page: &Page, selector: &str, timeout_ms: u64) -> bool {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            if start.elapsed() > timeout {
                return false;
            }
            if Self::exists(page, selector).await {
                return true;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }

    // ========================================
    // Country & Phone Input Helpers
    // ========================================

    /// Select a country from WhatsApp's country dropdown by country info.
    /// Searches by country name for reliability, falls back to dial code scan.
    pub async fn select_country_by_code(page: &Page, country: &CountryInfo) -> Result<bool> {
        let digits = country.dial_code;
        info!("select_country: starting for {} (+{})", country.name, digits);

        // Step 0: Check if the correct country is already selected
        let check_js = r#"(function() {
            var chevron = document.querySelector('[data-icon="chevron"]');
            if (!chevron) return '';
            var container = chevron.closest('[role="button"]') || chevron.closest('button') || chevron.parentElement;
            if (!container) return '';
            return (container.textContent || '').trim();
        })()"#;
        let current = page.evaluate(check_js).await?
            .into_value::<String>().unwrap_or_default();
        let code_pattern = format!("+{}", digits);
        if current.contains(&code_pattern) {
            info!("select_country: already correct (showing '{}')", current);
            return Ok(true);
        }
        info!("select_country: current='{}', need +{}", current, digits);

        // Step 1: Click the country dropdown button (identified by chevron icon)
        let click_js = r#"(function() {
            var chevron = document.querySelector('[data-icon="chevron"]');
            if (!chevron) return 'no_chevron';
            var btn = chevron.closest('button') || chevron.parentElement;
            if (!btn) return 'no_button';
            btn.scrollIntoView({ behavior: 'instant', block: 'center' });
            btn.click();
            return 'clicked';
        })()"#;

        let click_result = page.evaluate(click_js).await?
            .into_value::<String>().unwrap_or_else(|_| "error".into());
        info!("select_country: step1 chevron click = {}", click_result);
        if click_result != "clicked" {
            return Ok(false);
        }

        // Step 2: Poll for popover content (up to 3 seconds)
        let focus_js = r#"(function() {
            var popover = document.querySelector('#wa-popovers-bucket');
            if (!popover) return 'no_popover';
            var children = popover.children.length;
            var el = popover.querySelector('div[contenteditable="true"]')
                || popover.querySelector('div[role="textbox"]')
                || popover.querySelector('input[type="text"]')
                || popover.querySelector('input');
            if (!el) return 'no_search_field (children=' + children + ')';
            el.focus();
            el.click();
            return 'focused:' + el.tagName + '.' + (el.className || '').substring(0, 30);
        })()"#;

        let mut focus_result = String::new();
        for attempt in 0..6 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            focus_result = page.evaluate(focus_js).await?
                .into_value::<String>().unwrap_or_else(|_| "error".into());
            if focus_result.starts_with("focused") {
                break;
            }
            if attempt < 5 {
                info!("select_country: step2 attempt {}, waiting: {}", attempt + 1, focus_result);
            }
        }
        info!("select_country: step2 focus = {}", focus_result);

        let mut searched = false;
        if focus_result.starts_with("focused") {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            // Type country name for reliable filtering (e.g., "India" instead of "91")
            match page.find_element(":focus").await {
                Ok(el) => {
                    match el.type_str(country.name).await {
                        Ok(_) => {
                            searched = true;
                            info!("select_country: step2 typed '{}'", country.name);
                            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                        }
                        Err(e) => info!("select_country: step2 type_str failed: {}", e),
                    }
                }
                Err(e) => info!("select_country: step2 find :focus failed: {}", e),
            }
        }

        // Step 3: Click the button with exact dial code match (works on filtered or full list)
        let select_js = format!(
            r#"(function() {{
                var popover = document.querySelector('#wa-popovers-bucket');
                if (!popover) return 'no_popover';
                var buttons = popover.querySelectorAll('button');
                if (buttons.length === 0) return 'no_buttons';
                var re = new RegExp('\\+{}(?!\\d)');
                for (var i = 0; i < buttons.length; i++) {{
                    var txt = buttons[i].textContent || '';
                    if (re.test(txt)) {{
                        buttons[i].scrollIntoView({{block: 'center'}});
                        buttons[i].click();
                        return 'selected:' + txt.trim().substring(0, 40);
                    }}
                }}
                return 'no_match (checked ' + buttons.length + ' buttons, search={})';
            }})()
            "#,
            digits,
            if searched { country.name } else { "none" },
        );

        let select_result = page.evaluate(select_js.as_str()).await?
            .into_value::<String>().unwrap_or_else(|_| "error".into());
        info!("select_country: step3 select = {}", select_result);

        let selected = select_result.starts_with("selected");

        if selected {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        } else {
            Self::close_popover(page).await;
        }

        Ok(selected)
    }

    /// Close any open popover by pressing Escape
    async fn close_popover(page: &Page) {
        let _ = page.evaluate(
            r#"(function() {
                var e = new KeyboardEvent('keydown', {key:'Escape', code:'Escape', bubbles:true});
                document.dispatchEvent(e);
                document.activeElement && document.activeElement.blur();
            })()"#
        ).await;
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }

    /// Set phone number input value via JavaScript (reliable with React inputs on any OS).
    /// Clears existing value first, then sets the new value.
    pub async fn set_phone_input_value(page: &Page, selector: &str, number: &str) -> Result<bool> {
        let js = format!(
            r#"(function() {{
                var input = document.querySelector("{selector}")
                    || document.querySelector("input[type='text']");
                if (!input) return false;
                input.focus();
                input.click();
                var setter = Object.getOwnPropertyDescriptor(
                    HTMLInputElement.prototype, 'value'
                ).set;
                setter.call(input, '');
                input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                setter.call(input, '{number}');
                input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                return true;
            }})()
            "#,
            selector = selector.replace('"', "\\\""),
            number = number,
        );

        let result = page.evaluate(js.as_str()).await?;
        Ok(result.into_value::<bool>().unwrap_or(false))
    }

    // ========================================
    // Diagnostics
    // ========================================

    /// Capture page diagnostic info for debugging.
    /// Returns a summary string with URL, title, and visible content hints.
    pub async fn diagnose_page(page: &Page) -> String {
        let url = page.url().await.ok().flatten().unwrap_or_default();
        let diag_js = r#"(function() {
            var title = document.title || '';
            var body = (document.body && document.body.innerText || '').substring(0, 500);
            var inputs = document.querySelectorAll('input').length;
            var buttons = document.querySelectorAll('button').length;
            var canvas = document.querySelectorAll('canvas').length;
            var hasQr = document.querySelector("canvas[aria-label*='QR']") !== null
                || document.querySelector("canvas[aria-label*='Scan']") !== null;
            var hasPaneSide = document.querySelector('#pane-side') !== null;
            var hasProgress = document.querySelector('progress') !== null;
            return JSON.stringify({
                title: title,
                body_preview: body.replace(/\n+/g, ' ').substring(0, 300),
                inputs: inputs,
                buttons: buttons,
                canvas: canvas,
                has_qr: hasQr,
                has_pane_side: hasPaneSide,
                has_progress: hasProgress
            });
        })()"#;

        let diag = page.evaluate(diag_js).await
            .ok()
            .and_then(|v| v.into_value::<String>().ok())
            .unwrap_or_else(|| "failed to evaluate".into());

        format!("url={} | {}", url, diag)
    }

    /// Ensure page is on web.whatsapp.com. If not, navigate there.
    /// Returns false if navigation failed.
    pub async fn ensure_whatsapp_page(page: &Page) -> bool {
        let url = page.url().await.ok().flatten().unwrap_or_default();
        if url.contains("web.whatsapp.com") {
            return true;
        }
        tracing::warn!("Page not on WhatsApp Web (url={}), navigating...", url);
        match page.goto("https://web.whatsapp.com").await {
            Ok(_) => {
                tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
                true
            }
            Err(e) => {
                tracing::error!("Failed to navigate to WhatsApp Web: {}", e);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let config = LocatorConfig::defaults();
        assert_eq!(config.dialog.root, "[role='dialog']");
        assert_eq!(config.auth.authorized_pane, "#pane-side");
    }
}
