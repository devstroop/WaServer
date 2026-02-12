//! WhatsApp Web Element Locators
//!
//! Type-safe CSS selectors and XPath for WhatsApp Web UI elements.
//! Updated to match current WhatsApp Web UI (2024+).
//!
//! Also includes centralized timeout constants for consistency.

use anyhow::Result;
use chromiumoxide::page::Page;

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

/// WhatsApp Web element locators
pub struct Locators;

impl Locators {
    // ========================================
    // Authentication
    // ========================================

    /// QR code canvas element
    pub const QR_CODE_CANVAS: &'static str =
        "canvas[aria-label='Scan this QR code to link a device!']";

    /// Link with phone number button
    pub const PHONE_AUTH_LINK: &'static str = "span[role='button']";

    /// Phone number input field
    pub const PHONE_INPUT: &'static str = "[aria-label='Type your phone number.']";

    /// Phone code display element
    pub const PHONE_CODE: &'static str =
        "[aria-details='link-device-phone-number-code-screen-instructions']";

    /// Authorized side pane (indicates logged in)
    pub const SIDE_PANE: &'static str = "#pane-side";

    /// Loading progress indicator
    pub const LOADING_PROGRESS: &'static str = "progress[max='100']";

    // ========================================
    // Menu & Navigation
    // ========================================

    /// Menu button
    pub const MENU_BUTTON: &'static str = "button[title='Menu']";

    /// Logout menu item
    pub const LOGOUT_BUTTON: &'static str = "[aria-label='Log out']";

    /// Logout confirmation dialog
    pub const LOGOUT_DIALOG: &'static str = "[aria-label='Log out?']";

    // ========================================
    // Chat & Messaging
    // ========================================

    /// Message input field
    pub const MESSAGE_INPUT: &'static str =
        "#app #main footer div[aria-placeholder='Type a message']";

    /// Send button (icon)
    pub const SEND_BUTTON_ICON: &'static str = "span[data-icon='send']";

    /// Send button (aria-label)
    pub const SEND_BUTTON: &'static str = "button[aria-label='Send']";

    // ========================================
    // Attachments
    // ========================================

    /// Attach button
    pub const ATTACH_BUTTON: &'static str = "button[title='Attach']";

    /// Plus icon for attachments
    pub const ATTACH_PLUS_ICON: &'static str = "[data-icon='plus']";

    /// Photo/video file input
    pub const PHOTO_VIDEO_INPUT: &'static str =
        "input[accept='image/*,video/mp4,video/3gpp,video/quicktime']";

    /// Document file input
    pub const DOCUMENT_INPUT: &'static str = "input[accept='*']";

    /// Caption input for media
    pub const CAPTION_INPUT: &'static str = "#app div[aria-placeholder='Add a caption']";

    /// Send button for attachments
    pub const ATTACHMENT_SEND: &'static str = "#app div[aria-label='Send']";

    // ========================================
    // Chat List & Messages
    // ========================================

    /// Chat list container
    pub const CHAT_LIST: &'static str = "[data-testid='chat-list']";

    /// Chat list item row
    pub const CHAT_LIST_ITEM: &'static str = "[data-testid='cell-frame-container']";

    /// Conversation panel
    pub const CONVERSATION_PANEL: &'static str = "[data-testid='conversation-panel-messages']";

    /// Message container with ID
    pub const MESSAGE_ITEM: &'static str = "[data-id]";

    // ========================================
    // Dialogs
    // ========================================

    /// Generic dialog
    pub const DIALOG: &'static str = "[role='dialog']";

    /// Modal popup
    pub const MODAL_POPUP: &'static str = "div[data-animate-modal-popup='true']";

    /// Modal body
    pub const MODAL_BODY: &'static str = "div[data-animate-modal-body='true']";

    /// Invalid phone dialog
    pub const INVALID_PHONE_DIALOG: &'static str =
        "#app div[data-animate-modal-popup='true'] div[data-animate-modal-body='true']";

    // ========================================
    // Helper Methods
    // ========================================

    /// Get QR code as base64 PNG
    pub async fn get_qr_code_base64(page: &Page) -> Result<Option<String>> {
        let script = r#"
            (function() {
                var canvas = document.querySelector("canvas[aria-label='Scan this QR code to link a device!']");
                if (canvas) {
                    return canvas.toDataURL('image/png').split(',')[1];
                }
                return null;
            })();
        "#;

        match page.evaluate(script).await {
            Ok(result) => Ok(result.into_value::<Option<String>>().unwrap_or(None)),
            Err(_) => Ok(None),
        }
    }

    /// Get phone authentication code
    pub async fn get_phone_code(page: &Page) -> Result<Option<String>> {
        match page.find_element(Self::PHONE_CODE).await {
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
