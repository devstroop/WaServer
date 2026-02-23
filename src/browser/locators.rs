//! WhatsApp Web Element Locators
//!
//! Locators are loaded from `config/locators.toml` at runtime.
//! Update the TOML file when WhatsApp Web UI changes - no recompilation needed.
//! Falls back to built-in defaults if config file is missing.

use anyhow::{Context, Result};
use chromiumoxide::page::Page;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

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
                login_with_phone_link: "span[role='button']".into(),
                login_with_qr_link: "span[role='button']".into(),
                login_label: "text='Log into WhatsApp Web'".into(),
                phone_number_label: "text='Enter phone number'".into(),
                phone_number_input: "[aria-label='Type your phone number.']".into(),
                phone_submit_button: "div[role='button']".into(),
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
    // Helper Methods
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

    /// Check if element exists
    pub async fn element_exists(page: &Page, selector: &str) -> bool {
        let script = format!(
            "document.querySelector('{}') !== null",
            selector.replace('\'', "\\'")
        );

        page.evaluate(script.as_str())
            .await
            .ok()
            .and_then(|r| r.into_value::<bool>().ok())
            .unwrap_or(false)
    }

    /// Wait for element with timeout
    pub async fn wait_for_element(page: &Page, selector: &str, timeout_ms: u64) -> Result<bool> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            if start.elapsed() > timeout {
                return Ok(false);
            }

            if Self::element_exists(page, selector).await {
                return Ok(true);
            }

            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }

    /// Click element using JavaScript
    pub async fn click_element(page: &Page, selector: &str) -> Result<bool> {
        let script = format!(
            r#"(function() {{
                var el = document.querySelector('{}');
                if (el) {{ el.click(); return true; }}
                return false;
            }})();"#,
            selector.replace('\'', "\\'")
        );

        let result = page.evaluate(script.as_str()).await?;
        Ok(result.into_value::<bool>().unwrap_or(false))
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
