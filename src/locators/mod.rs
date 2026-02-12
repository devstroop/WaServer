use anyhow::Result;
use chromiumoxide::page::Page;

/// WhatsApp Web element locators dictionary
/// This struct provides type-safe access to all WhatsApp Web UI elements
/// Updated to match current WhatsApp Web UI (2024+)
pub struct LocatorDictionary {}

impl LocatorDictionary {
    pub fn new() -> Self {
        Self {}
    }

    // ========================================
    // Dialog elements
    // ========================================

    pub fn dialog(&self) -> &str {
        "[role='dialog']"
    }

    pub fn dialog_backdrop(&self) -> &str {
        "div[data-animate-modal-backdrop='true']"
    }

    pub fn dialog_popup(&self) -> &str {
        "div[data-animate-modal-popup='true']"
    }

    pub fn dialog_body(&self) -> &str {
        "div[data-animate-modal-body='true']"
    }

    // ========================================
    // Authentication elements
    // ========================================

    pub fn login_with_phone_number_link(&self) -> &str {
        "span[role='button']" // Updated: Look for "Link with phone number" text
    }

    pub fn login_with_qr_code_link(&self) -> &str {
        "span[role='button']" // Updated: Look for "Link with QR code" text
    }

    pub fn enter_phone_number_label(&self) -> &str {
        "text='Enter phone number'"
    }

    pub fn enter_phone_number_input(&self) -> &str {
        "[aria-label='Type your phone number.']"
    }

    pub fn submit_phone_number_button(&self) -> &str {
        "div[role='button']" // Look for "Next" text
    }

    pub fn enter_code_on_phone_label(&self) -> &str {
        "text='Enter code on phone'"
    }

    pub fn enter_code_on_phone_value(&self) -> &str {
        "[aria-details='link-device-phone-number-code-screen-instructions']"
    }

    pub fn login_to_whatsapp_web_label(&self) -> &str {
        "text='Log into WhatsApp Web'"
    }

    // ========================================
    // QR Code elements
    // ========================================

    pub fn qr_loading_indicator(&self) -> &str {
        "svg[role='status']"
    }

    pub fn scan_this_qr_element(&self) -> &str {
        "canvas[aria-label='Scan this QR code to link a device!']"
    }

    pub fn click_to_reload_qr_button(&self) -> &str {
        "[data-icon='refresh-large']"
    }

    // ========================================
    // Loading and status elements
    // ========================================

    pub fn loading_progress_indicator(&self) -> &str {
        "progress[max='100']"
    }

    pub fn authorized_side_pane(&self) -> &str {
        "#pane-side"
    }

    // ========================================
    // Menu elements
    // ========================================

    pub fn menu(&self) -> &str {
        "button[title='Menu']"
    }

    pub fn menu_dropdown(&self) -> &str {
        "[aria-label='Menu']"
    }

    pub fn menu_logout(&self) -> &str {
        "[aria-label='Log out']"
    }

    pub fn logout_confirm_dialog(&self) -> &str {
        "[aria-label='Log out?']"
    }

    // ========================================
    // Phone linking elements
    // ========================================

    pub fn link_phone_number_code_element(&self) -> &str {
        "[aria-details='link-device-phone-number-code-screen-instructions']"
    }

    pub fn link_code_digits(&self) -> &str {
        "[data-link-code]"
    }

    // ========================================
    // Chat elements
    // ========================================

    pub fn type_a_message_input(&self) -> &str {
        "#app #main footer div[aria-placeholder='Type a message']"
    }

    pub fn message_input_contenteditable(&self) -> &str {
        "div[contenteditable='true'][aria-placeholder='Type a message']"
    }

    pub fn send_button(&self) -> &str {
        "span[data-icon='send']"
    }

    pub fn send_button_parent(&self) -> &str {
        "button span[data-icon='send']"
    }

    // ========================================
    // Attachment elements
    // ========================================

    pub fn attachment_button(&self) -> &str {
        "button[title='Attach']"
    }

    pub fn attachment_plus_icon(&self) -> &str {
        "[data-icon='plus']"
    }

    pub fn attachment_menu_plus_icon(&self) -> &str {
        "[data-icon='attach-menu-plus']"
    }

    pub fn photo_and_video_attachment_input(&self) -> &str {
        "input[accept='image/*,video/mp4,video/3gpp,video/quicktime']"
    }

    pub fn attachment_caption_input(&self) -> &str {
        "#app div[aria-placeholder='Add a caption']"
    }

    pub fn document_attachment_input(&self) -> &str {
        "input[accept='*']"
    }

    pub fn attachment_send_button(&self) -> &str {
        "#app div[aria-label='Send']"
    }

    // ========================================
    // Phone loader (for navigation)
    // ========================================

    pub fn phone_loader_parent(&self) -> &str {
        "#phoneLoaderParent"
    }

    pub fn phone_loader(&self) -> &str {
        "#phoneLoader"
    }

    // ========================================
    // Invalid phone dialog
    // ========================================

    pub fn invalid_phone_dialog(&self) -> &str {
        "#app div[data-animate-modal-popup='true'] div[data-animate-modal-body='true']"
    }

    /// Get the data-link-code attribute from the phone link element
    pub async fn data_link_code(&self, page: &Page) -> Result<Option<String>> {
        match page
            .find_element(self.link_phone_number_code_element())
            .await
        {
            Ok(element) => match element.attribute("data-link-code").await {
                Ok(value) => Ok(value),
                Err(_) => Ok(None),
            },
            Err(_) => Ok(None),
        }
    }

    /// Get QR code canvas as base64
    pub async fn get_qr_code_base64(&self, page: &Page) -> Result<Option<String>> {
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
}

impl Default for LocatorDictionary {
    fn default() -> Self {
        Self::new()
    }
}
