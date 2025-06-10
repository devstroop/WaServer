use headless_chrome::Tab;
use std::sync::Arc;

/// WhatsApp Web element locators dictionary
/// This struct provides type-safe access to all WhatsApp Web UI elements
pub struct LocatorDictionary {
    tab: Arc<Tab>,
}

impl LocatorDictionary {
    pub fn new(tab: Arc<Tab>) -> Self {
        Self { tab }
    }

    // Dialog elements
    pub fn dialog(&self) -> &str {
        "[role='dialog']"
    }

    pub fn dialog_backdrop(&self) -> &str {
        "div[data-animate-modal-backdrop='true']"
    }

    // Authentication elements
    pub fn login_with_phone_number_link(&self) -> &str {
        "text='Log in with phone number'"
    }

    pub fn login_with_qr_code_link(&self) -> &str {
        "text='Log in with QR code'"
    }

    pub fn enter_phone_number_label(&self) -> &str {
        "text='Enter phone number'"
    }

    pub fn enter_phone_number_input(&self) -> &str {
        "[aria-label='Type your phone number.']"
    }

    pub fn submit_phone_number_button(&self) -> &str {
        "text='Next'"
    }

    pub fn enter_code_on_phone_label(&self) -> &str {
        "text='Enter code on phone'"
    }

    pub fn enter_code_on_phone_value(&self) -> &str {
        "[aria-label='Enter code on phone:']"
    }

    pub fn login_to_whatsapp_web_label(&self) -> &str {
        "text='Log into WhatsApp Web'"
    }

    // QR Code elements
    pub fn qr_loading_indicator(&self) -> &str {
        "svg[role='status']"
    }

    pub fn scan_this_qr_element(&self) -> &str {
        "[aria-label='Scan this QR code to link a device!']"
    }

    pub fn click_to_reload_qr_button(&self) -> &str {
        "text='Click to reload QR code'"
    }

    // Loading and status elements
    pub fn loading_progress_indicator(&self) -> &str {
        "progress[max='100']"
    }

    pub fn authorized_side_pane(&self) -> &str {
        "#pane-side"
    }

    // Menu elements
    pub fn menu(&self) -> &str {
        "[aria-label='Menu']"
    }

    pub fn menu_logout(&self) -> &str {
        "[aria-label='Log out']"
    }

    // Phone linking elements
    pub fn link_phone_number_code_element(&self) -> &str {
        "[aria-details='link-device-phone-number-code-screen-instructions']"
    }

    // Chat elements
    pub fn type_a_message_input(&self) -> &str {
        "[aria-label='Type a message']"
    }

    pub fn send_button(&self) -> &str {
        "[aria-label='Send']"
    }

    // Attachment elements
    pub fn attachment_button(&self) -> &str {
        "[data-icon='plus']"
    }

    pub fn photo_and_video_attachment_input(&self) -> &str {
        "input[accept='image/*,video/mp4,video/3gpp,video/quicktime']"
    }

    pub fn attachment_caption_input(&self) -> &str {
        "[aria-label='Add a caption']"
    }

    pub fn document_attachment_input(&self) -> &str {
        "input[accept='*']"
    }

    /// Get the data-link-code attribute from the phone link element
    pub fn data_link_code(&self) -> anyhow::Result<Option<String>> {
        match self.tab.find_element(self.link_phone_number_code_element()) {
            Ok(element) => {
                match element.get_attribute_value("data-link-code") {
                    Ok(value) => Ok(value),
                    Err(_) => Ok(None),
                }
            },
            Err(_) => Ok(None),
        }
    }
}
