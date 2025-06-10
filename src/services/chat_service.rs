use crate::{
    config::AppConfig,
    locators::LocatorDictionary,
    services::browser::BrowserService,
};
use anyhow::Result;
use async_trait::async_trait;
use mime_guess::MimeGuess;
use chromiumoxide::page::Page;
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
    async fn get_page(&self) -> Result<Page> {
        self.browser_service.get_or_create_page("https://web.whatsapp.com").await
    }

    /// Navigate to a specific chat by phone number
    async fn navigate_to_chat(&self, page: &Page, phone: &str) -> Result<()> {
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

        page.evaluate(script).await?;

        // Wait for any dialog to close
        let _locators = LocatorDictionary::new();
        
        // Try to wait for dialog to close, but don't fail if it doesn't exist
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        Ok(())
    }

    /// Send a text message
    async fn send_text_message(&self, page: &Page, text: &str) -> Result<()> {
        let _locators = LocatorDictionary::new();

        debug!("Sending text message: {}", text);

        // Find message input and send text
        let message_input = page.find_element("[aria-label='Type a message']").await?;
        message_input.click().await?;
        message_input.type_str(text).await?;
        
        // Press Enter to send
        let enter_script = r#"
            document.querySelector('[aria-label="Type a message"]').dispatchEvent(
                new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', which: 13, keyCode: 13 })
            );
        "#;
        page.evaluate(enter_script).await?;

        info!("Text message sent successfully");
        Ok(())
    }

    /// Send a file attachment with optional caption
    async fn send_attachment(&self, page: &Page, attachment_path: &str, _caption: Option<&str>) -> Result<()> {
        let _locators = LocatorDictionary::new();

        debug!("Sending attachment: {}", attachment_path);

        // Determine file type  
        let _mime_type = MimeGuess::from_path(attachment_path).first_or_octet_stream();
        let _content_type = _mime_type.to_string();

        // Click attach button
        let attach_button = page.find_element("[data-icon='plus']").await?;
        attach_button.click().await?;

        // File uploads are not yet implemented with chromiumoxide
        // This requires more complex file input handling
        Err(anyhow::anyhow!("File uploads not yet implemented with chromiumoxide"))
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

        let page = self.get_page().await?;

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
        self.navigate_to_chat(&page, phone).await?;

        // Send message based on type
        if let Some(attachment) = attachment_path {
            // Send attachment with optional caption
            self.send_attachment(&page, attachment, text).await?;
        } else if let Some(message_text) = text {
            // Send text message
            self.send_text_message(&page, message_text).await?;
        }

        info!("Message sent successfully to {}", phone);
        Ok(())
    }
}
