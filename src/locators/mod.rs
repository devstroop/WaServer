use playwright::api::{Locator, Page};

/// WhatsApp Web element locators dictionary
/// This struct provides type-safe access to all WhatsApp Web UI elements
#[derive(Debug)]
pub struct LocatorDictionary<'a> {
    page: &'a Page,
}

impl<'a> LocatorDictionary<'a> {
    pub fn new(page: &'a Page) -> Self {
        Self { page }
    }

    // Dialog elements
    pub fn dialog(&self) -> Locator {
        self.page.get_by_role(playwright::api::AriaRole::Dialog, None)
    }

    pub fn dialog_backdrop(&self) -> Locator {
        self.page.locator("div[data-animate-modal-backdrop='true']")
    }

    // Authentication elements
    pub fn login_with_phone_number_link(&self) -> Locator {
        self.page.get_by_text("Log in with phone number", None)
    }

    pub fn login_with_qr_code_link(&self) -> Locator {
        self.page.get_by_text("Log in with QR code", None)
    }

    pub fn enter_phone_number_label(&self) -> Locator {
        self.page.get_by_text("Enter phone number", None)
    }

    pub fn enter_phone_number_input(&self) -> Locator {
        self.page.get_by_label("Type your phone number.", None)
    }

    pub fn submit_phone_number_button(&self) -> Locator {
        self.page.get_by_text("Next", None)
    }

    pub fn enter_code_on_phone_label(&self) -> Locator {
        self.page.get_by_text("Enter code on phone", None)
    }

    pub fn enter_code_on_phone_value(&self) -> Locator {
        self.page.get_by_label("Enter code on phone:", None)
    }

    pub fn login_to_whatsapp_web_label(&self) -> Locator {
        self.page.get_by_text("Log into WhatsApp Web", None)
    }

    // QR Code elements
    pub fn qr_loading_indicator(&self) -> Locator {
        self.page.locator("svg[role='status']")
    }

    pub fn scan_this_qr_element(&self) -> Locator {
        self.page.get_by_label("Scan this QR code to link a device!", None)
    }

    pub fn click_to_reload_qr_button(&self) -> Locator {
        self.page.get_by_text("Click to reload QR code", None)
    }

    // Loading and status elements
    pub fn loading_progress_indicator(&self) -> Locator {
        self.page.locator("progress[max='100']")
    }

    pub fn authorized_side_pane(&self) -> Locator {
        self.page.locator("#pane-side")
    }

    // Menu elements
    pub fn menu(&self) -> Locator {
        self.page.get_by_label("Menu", None)
    }

    pub fn menu_logout(&self) -> Locator {
        self.page.get_by_label("Log out", None)
    }

    // Phone linking elements
    pub fn link_phone_number_code_element(&self) -> Locator {
        self.page.locator("[aria-details='link-device-phone-number-code-screen-instructions']")
    }

    // Chat elements
    pub fn type_a_message_input(&self) -> Locator {
        self.page.get_by_label("Type a message", None)
    }

    pub fn send_button(&self) -> Locator {
        self.page.get_by_label("Send", None)
    }

    // Attachment elements
    pub fn attachment_button(&self) -> Locator {
        self.page.locator("[data-icon='plus']")
    }

    pub fn photo_and_video_attachment_input(&self) -> Locator {
        self.page.locator("input[accept='image/*,video/mp4,video/3gpp,video/quicktime']")
    }

    pub fn attachment_caption_input(&self) -> Locator {
        self.page.get_by_label("Add a caption", None)
    }

    pub fn document_attachment_input(&self) -> Locator {
        self.page.locator("input[accept='*']")
    }
}

impl<'a> LocatorDictionary<'a> {
    /// Get the data-link-code attribute from the phone link element
    pub async fn data_link_code(&self) -> Result<Option<String>, playwright::Error> {
        self.link_phone_number_code_element()
            .get_attribute("data-link-code", None)
            .await
    }
}
