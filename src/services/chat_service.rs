use crate::{
    config::AppConfig,
    locators::LocatorDictionary,
    services::browser::BrowserService,
};
use anyhow::Result;
use async_trait::async_trait;
use mime_guess::MimeGuess;
use headless_chrome::Tab;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info};

/// Chat service trait
#[async_trait]
pub trait ChatServiceTrait: Send + Sync {
    async fn send_message(
        &self,
        phone: &str,
        text: Option<&str>,
        attachment_path: Option<&str>,
        timeout_ms: Option<u64>,
    ) -> Result<()>;
}

/// WhatsApp chat service
pub struct ChatService {
    config: Arc<AppConfig>,
    browser_service: Arc<BrowserService>,
    message_queue: Semaphore,
}

impl ChatService {
    pub fn new(config: Arc<AppConfig>, browser_service: Arc<BrowserService>) -> Self {
        Self {
            config,
            browser_service,
            message_queue: Semaphore::new(1), // Ensure only one message is processed at a time
        }
    }

    /// Get page from browser service
    async fn get_tab(&self) -> Result<Arc<Tab>> {
        self.browser_service.get_or_create_tab("https://web.whatsapp.com").await
    }

    /// Navigate to a specific chat by phone number
    async fn navigate_to_chat(&self, tab: &Arc<Tab>, phone: &str) -> Result<()> {
        let phone_number = if phone.contains('@') || phone.contains(':') {
            phone.split('@').next()
                .and_then(|part| part.split(':').next())
                .unwrap_or(phone)
        } else {
            phone
        };

        debug!("Navigating to chat for phone: {}", phone_number);

        // Inject JavaScript to navigate to the chat
        let script = format!(
            r#"
            var pLdr = document.querySelectorAll('#phoneLoaderParent');
            if(pLdr.length == 0) {{
                document.querySelector('#pane-side').innerHTML += '<div id="phoneLoaderParent"></div>';
            }}
            document.querySelector('#phoneLoaderParent').innerHTML = '<a id="phoneLoader" href="https://api.whatsapp.com/send?phone={}"></a>';
            document.querySelector('#phoneLoader').click();
            document.querySelector('#phoneLoaderParent').remove();
            "#,
            phone_number
        );

        tab.evaluate(&script, false)?;

        // Wait for any dialog to close
        let _locators = LocatorDictionary::new(tab.clone());
        
        // Try to wait for dialog to close, but don't fail if it doesn't exist
        std::thread::sleep(std::time::Duration::from_millis(2000));

        Ok(())
    }

    /// Send a text message
    async fn send_text_message(&self, tab: &Arc<Tab>, text: &str) -> Result<()> {
        let _locators = LocatorDictionary::new(tab.clone());

        debug!("Sending text message: {}", text);

        // Find message input and send text
        let message_input = tab.find_element("[aria-label='Type a message']")?;
        message_input.click()?;
        message_input.type_into(text)?;
        
        // Press Enter to send
        tab.press_key("Enter")?;

        info!("Text message sent successfully");
        Ok(())
    }

    /// Send a file attachment with optional caption
    async fn send_attachment(&self, tab: &Arc<Tab>, attachment_path: &str, caption: Option<&str>) -> Result<()> {
        let _locators = LocatorDictionary::new(tab.clone());

        debug!("Sending attachment: {}", attachment_path);

        // Determine file type
        let mime_type = MimeGuess::from_path(attachment_path).first_or_octet_stream();
        let content_type = mime_type.to_string();

        // Click attach button
        let attach_button = tab.find_element("[data-icon='plus']")?;
        attach_button.click()?;

        // Choose appropriate input based on file type
        if content_type.starts_with("image/") || content_type.starts_with("video/") {
            debug!("Uploading as photo/video");
            let file_input = tab.find_element("input[accept='image/*,video/mp4,video/3gpp,video/quicktime']")?;
            file_input.set_input_files(&[attachment_path.into()])?;
        } else {
            debug!("Uploading as document");
            let file_input = tab.find_element("input[accept='*']")?;
            file_input.set_input_files(&[attachment_path.into()])?;
        }

        // Add caption if provided
        if let Some(caption_text) = caption {
            debug!("Adding caption: {}", caption_text);
            let caption_input = tab.find_element("[aria-label='Add a caption']")?;
            caption_input.type_into(caption_text)?;
        }

        // Send the attachment
        tab.press_key("Enter")?;

        info!("Attachment sent successfully");
        Ok(())
    }
}

#[async_trait]
impl ChatServiceTrait for ChatService {
    async fn send_message(
        &self,
        phone: &str,
        text: Option<&str>,
        attachment_path: Option<&str>,
        timeout_ms: Option<u64>,
    ) -> Result<()> {
        let timeout = timeout_ms.unwrap_or(self.config.browser.timeout_ms);
        
        // Acquire message queue lock with timeout
        let _permit = tokio::time::timeout(
            std::time::Duration::from_millis(timeout),
            self.message_queue.acquire()
        ).await??;

        debug!("Acquired message queue lock");

        let tab = self.get_tab().await?;

        // Validate inputs
        if phone.is_empty() {
            return Err(anyhow::anyhow!("Phone number cannot be empty"));
        }

        if attachment_path.is_none() && text.is_none() {
            return Err(anyhow::anyhow!("Either text or attachment must be provided"));
        }

        if let Some(path) = attachment_path {
            if !Path::new(path).exists() {
                return Err(anyhow::anyhow!("Attachment file does not exist: {}", path));
            }
        }

        // Navigate to the chat
        self.navigate_to_chat(&tab, phone).await?;

        // Send message based on type
        if let Some(attachment) = attachment_path {
            // Send attachment with optional caption
            self.send_attachment(&tab, attachment, text).await?;
        } else if let Some(message_text) = text {
            // Send text message
            self.send_text_message(&tab, message_text).await?;
        }

        info!("Message sent successfully to {}", phone);
        Ok(())
    }
}
