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
use tracing::{debug, info, error};

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
    _config: Arc<AppConfig>, // Prefixed with _ to indicate intentionally unused for now
    browser_service: Arc<BrowserService>,
    message_queue: Semaphore,
}

impl ChatService {
    pub fn new(config: Arc<AppConfig>, browser_service: Arc<BrowserService>) -> Self {
        Self {
            _config: config,
            browser_service,
            message_queue: Semaphore::new(1), // Ensure only one message is processed at a time
        }
    }

    /// Get page from browser service
    async fn get_page(&self) -> Result<Page> {
        self.browser_service.get_whatsapp_page().await
    }

    /// Pre-check to dismiss any dialogs that might be blocking operations
    async fn pre_check(&self, page: &Page) -> Result<()> {
        // Check if there's a dialog and dismiss it
        if let Ok(_dialog) = page.find_element("[role='dialog']").await {
            if let Ok(backdrop) = page.find_element("div[data-animate-modal-backdrop='true']").await {
                debug!("Dismissing dialog by clicking backdrop");
                backdrop.click().await?;
                
                // Wait for dialog to disappear
                tokio::time::timeout(
                    std::time::Duration::from_millis(10000),
                    async {
                        while page.find_element("[role='dialog']").await.is_ok() {
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        }
                    }
                ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for dialog to disappear"))?;
            }
        }
        
        Ok(())
    }

    /// Wait for loading indicators to disappear
    async fn wait_til_loading(&self, page: &Page) -> Result<()> {
        // Wait for loading progress indicator to disappear
        tokio::time::timeout(
            std::time::Duration::from_millis(10000),
            async {
                while page.find_element("progress[max='100']").await.is_ok() {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        ).await.map_err(|_| anyhow::anyhow!("Timeout waiting for loading to complete"))?;
        
        Ok(())
    }

    /// Check if user is authorized
    async fn check_authorization(&self, page: &Page) -> Result<bool> {
        let script = r#"
            document.querySelector('#pane-side') !== null
        "#;

        match page.evaluate(script).await {
            Ok(result) => {
                let is_authorized = match result.into_value()? {
                    serde_json::Value::Bool(b) => b,
                    _ => false,
                };
                debug!("Auth check result: {}", is_authorized);
                Ok(is_authorized)
            }
            Err(e) => {
                error!("Error checking auth status: {}", e);
                Ok(false)
            }
        }
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

        let locators = LocatorDictionary::new();
        
        // Wait for dialog to disappear with timeout (10 seconds)
        let timeout = std::time::Duration::from_millis(10000);
        let start_time = std::time::Instant::now();
        
        loop {
            if start_time.elapsed() > timeout {
                debug!("Timeout waiting for dialog to close");
                break;
            }
            
            // Check if dialog is still present
            match page.find_element(locators.dialog()).await {
                Ok(_dialog) => {
                    // Dialog still exists, wait a bit more
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                },
                Err(_) => {
                    debug!("Dialog element not found, chat navigation complete");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Send a text message
    async fn send_text_message(&self, page: &Page, text: &str) -> Result<()> {
        let locators = LocatorDictionary::new();

        debug!("Sending text message: {}", text);

        // Find message input with retry mechanism
        let mut attempts = 0;
        let max_attempts = 5;
        
        while attempts < max_attempts {
            match page.find_element(locators.type_a_message_input()).await {
                Ok(message_input) => {
                    // Clear the input first
                    match message_input.click().await {
                        Ok(_) => {
                            // Clear any existing content
                            let clear_script = r#"
                                const input = document.querySelector('[aria-label="Type a message"]');
                                if (input) {
                                    input.focus();
                                    input.select();
                                    input.value = '';
                                }
                            "#;
                            let _ = page.evaluate(clear_script).await;
                            
                            // Type the message
                            match message_input.type_str(text).await {
                                Ok(_) => {
                                    // Press Enter to send
                                    let enter_script = r#"
                                        const input = document.querySelector('[aria-label="Type a message"]');
                                        if (input) {
                                            const event = new KeyboardEvent('keydown', { 
                                                key: 'Enter', 
                                                code: 'Enter', 
                                                which: 13, 
                                                keyCode: 13,
                                                bubbles: true
                                            });
                                            input.dispatchEvent(event);
                                        }
                                    "#;
                                    
                                    match page.evaluate(enter_script).await {
                                        Ok(_) => {
                                            info!("Text message sent successfully");
                                            return Ok(());
                                        },
                                        Err(e) => {
                                            error!("Failed to send message via script: {}", e);
                                            // Try alternative approach - press Enter directly
                                            if let Err(press_err) = message_input.press_key("Enter").await {
                                                error!("Failed to press Enter: {}", press_err);
                                            } else {
                                                info!("Text message sent successfully via key press");
                                                return Ok(());
                                            }
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!("Failed to type message (attempt {}): {}", attempts + 1, e);
                                }
                            }
                        },
                        Err(e) => {
                            error!("Failed to click message input (attempt {}): {}", attempts + 1, e);
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to find message input (attempt {}): {}", attempts + 1, e);
                }
            }
            
            attempts += 1;
            if attempts < max_attempts {
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            }
        }

        Err(anyhow::anyhow!("Failed to send text message after {} attempts", max_attempts))
    }

    /// Send a file attachment with optional caption
    async fn send_attachment(&self, page: &Page, attachment_path: &str, _caption: Option<&str>) -> Result<()> {
        let locators = LocatorDictionary::new();

        debug!("Sending attachment: {}", attachment_path);

        // Determine file type  
        let mime_type = MimeGuess::from_path(attachment_path).first_or_octet_stream();
        let content_type = mime_type.to_string();

        // Click attach button
        let attach_button = page.find_element(locators.attachment_button()).await?;
        attach_button.click().await?;

        // Wait a moment for the attachment menu to appear
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        // Note: File uploads with chromiumoxide are limited
        // The set_input_files method may not be available or work as expected
        // This is a known limitation of chromiumoxide compared to Playwright
        
        if content_type.contains("image") || content_type.contains("video") {
            // Try to find and use the photo/video input
            match page.find_element(locators.photo_and_video_attachment_input()).await {
                Ok(_input_element) => {
                    // This is a workaround - chromiumoxide doesn't have a direct equivalent to SetInputFilesAsync
                    // We would need to use JavaScript to set the file input value, but this has security limitations
                    error!("Photo/video file upload not fully supported with chromiumoxide");
                    return Err(anyhow::anyhow!("Photo/video file uploads require Playwright for full functionality"));
                },
                Err(e) => {
                    error!("Failed to find photo/video input: {}", e);
                    return Err(anyhow::anyhow!("Failed to find photo/video input element"));
                }
            }
        } else {
            // Try to find and use the document input
            match page.find_element(locators.document_attachment_input()).await {
                Ok(_input_element) => {
                    error!("Document file upload not fully supported with chromiumoxide");
                    return Err(anyhow::anyhow!("Document file uploads require Playwright for full functionality"));
                },
                Err(e) => {
                    error!("Failed to find document input: {}", e);
                    return Err(anyhow::anyhow!("Failed to find document input element"));
                }
            }
        }
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
        let timeout = timeout_ms.unwrap_or(30000); // Default 30 seconds
        
        // Acquire message queue lock with timeout
        let _permit = tokio::time::timeout(
            std::time::Duration::from_millis(timeout),
            self.message_queue.acquire()
        ).await??;

        debug!("Acquired message queue lock for phone: {}", phone);

        let page = self.get_page().await?;

        // Pre-check for any dialogs that need dismissal
        if let Err(e) = self.pre_check(&page).await {
            error!("Pre-check failed: {}", e);
            return Err(e);
        }

        // Wait for loading to complete
        if let Err(e) = self.wait_til_loading(&page).await {
            error!("Wait til loading failed: {}", e);
            return Err(e);
        }

        // Check authorization
        match self.check_authorization(&page).await {
            Ok(authorized) => {
                if !authorized {
                    return Err(anyhow::anyhow!("Not authorized"));
                }
            },
            Err(e) => {
                error!("Failed to check authorization: {}", e);
                return Err(anyhow::anyhow!("Failed to check authorization: {}", e));
            }
        }

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
