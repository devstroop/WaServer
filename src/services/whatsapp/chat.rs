//! Chat/Messaging Service
//!
//! Handles sending text messages and attachments via WhatsApp Web.
//! Based on proven .NET implementation patterns.

use crate::{browser::BrowserService, config::AppConfig};
use anyhow::Result;
use async_trait::async_trait;
use base64::Engine;
use chromiumoxide::page::Page;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, error, info};

// ============================================================================
// Trait Definition
// ============================================================================

/// Chat service trait for sending messages
#[async_trait]
pub trait ChatServiceTrait: Send + Sync {
    /// Send a message with optional text and/or attachment
    async fn send_message(
        &self,
        phone: &str,
        text: Option<&str>,
        attachment_path: Option<&str>,
        timeout_ms: Option<u64>,
    ) -> Result<()>;

    /// Get list of visible chats from sidebar
    async fn get_chat_list(&self) -> Result<Vec<crate::models::chat::ChatInfo>>;

    /// Get messages from a specific chat
    async fn get_messages(
        &self,
        chat_id: &str,
        limit: Option<u32>,
        load_more: bool,
    ) -> Result<crate::models::chat::MessageListResponse>;

    /// Watch for new incoming messages
    async fn watch_messages(&self) -> Result<Vec<crate::models::chat::MessageInfo>>;

    /// Send typing indicator (composing/paused)
    async fn send_typing(
        &self,
        chat_id: &str,
        state: crate::models::chat::TypingState,
    ) -> Result<()>;

    /// Mark messages as read
    async fn mark_read(&self, chat_id: &str) -> Result<u32>;

    /// Get presence/online status for a contact
    async fn get_presence(&self, chat_id: &str) -> Result<crate::models::chat::PresenceInfo>;

    /// Get detailed group info
    async fn get_group_info(&self, group_id: &str) -> Result<crate::models::chat::GroupInfo>;

    /// Get contact profile info
    async fn get_contact_info(&self, contact_id: &str) -> Result<crate::models::chat::ContactInfo>;

    /// Send a reaction to a message
    async fn send_reaction(&self, chat_id: &str, message_id: &str, emoji: &str) -> Result<()>;

    /// Send a reply to a message
    async fn send_reply(&self, chat_id: &str, quoted_message_id: &str, text: &str) -> Result<()>;
}

// ============================================================================
// Service Implementation
// ============================================================================

/// WhatsApp chat service
pub struct ChatService {
    #[allow(dead_code)]
    config: Arc<AppConfig>,
    browser_service: Arc<BrowserService>,
    message_queue: Semaphore,
    /// Track if message listener has been injected into the browser
    listener_injected: AtomicBool,
}

impl ChatService {
    pub fn new(config: Arc<AppConfig>, browser_service: Arc<BrowserService>) -> Self {
        Self {
            config,
            browser_service,
            message_queue: Semaphore::new(1),
            listener_injected: AtomicBool::new(false),
        }
    }

    async fn get_page(&self) -> Result<Page> {
        self.browser_service.get_whatsapp_page().await
    }

    async fn check_authorization(&self, page: &Page) -> Result<bool> {
        let script = "document.querySelector('#pane-side') !== null";
        match page.evaluate(script).await {
            Ok(result) => Ok(result.into_value::<bool>().unwrap_or(false)),
            Err(e) => {
                error!("Error checking auth: {}", e);
                Ok(false)
            }
        }
    }

    /// Inject message listener into the browser page.
    /// This sets up MutationObserver + Notification API interception to capture
    /// incoming messages without polling the DOM.
    async fn inject_message_listener(&self, page: &Page) -> Result<()> {
        // Check if already injected (using atomic flag)
        if self.listener_injected.swap(true, Ordering::SeqCst) {
            // Already injected, but verify it's still active in the page
            let check_script = "typeof window.__WAS_MESSAGE_QUEUE !== 'undefined'";
            if let Ok(result) = page.evaluate(check_script).await {
                if result.into_value::<bool>().unwrap_or(false) {
                    return Ok(());
                }
            }
            // Queue was lost (page reload?), re-inject
            debug!("Re-injecting message listener (queue lost)");
        }

        info!("Injecting message listener into WhatsApp Web page");

        let listener_script = r##"
        (function() {
            // Prevent double injection
            if (window.__WAS_LISTENER_ACTIVE) return { status: 'already_active' };
            window.__WAS_LISTENER_ACTIVE = true;

            // Message queue for incoming messages
            window.__WAS_MESSAGE_QUEUE = [];
            
            // Track processed message IDs to avoid duplicates
            window.__WAS_PROCESSED_IDS = new Set();
            
            // Helper: Generate unique message ID
            function generateId() {
                return Date.now().toString(36) + '_' + Math.random().toString(36).substr(2, 9);
            }
            
            // Helper: Extract message info from chat row element
            function extractMessageFromRow(row) {
                if (!row) return null;
                
                const nameEl = row.querySelector('span[title]');
                const msgEl = row.querySelector('[data-testid="last-msg-status"]')?.parentElement 
                           || row.querySelector('[data-testid="cell-frame-secondary"]');
                const timeEl = row.querySelector('[data-testid="cell-frame-primary-detail"]');
                const unreadBadge = row.querySelector('[data-testid="icon-unread-count"]');
                
                if (!nameEl || !msgEl) return null;
                
                const chatName = nameEl.title || nameEl.innerText;
                const messageText = msgEl.innerText;
                const uniqueKey = chatName + ':' + messageText;
                
                // Skip if already processed
                if (window.__WAS_PROCESSED_IDS.has(uniqueKey)) return null;
                window.__WAS_PROCESSED_IDS.add(uniqueKey);
                
                // Limit processed IDs set size
                if (window.__WAS_PROCESSED_IDS.size > 500) {
                    const arr = Array.from(window.__WAS_PROCESSED_IDS);
                    window.__WAS_PROCESSED_IDS = new Set(arr.slice(-250));
                }
                
                return {
                    id: generateId(),
                    from_me: false,
                    sender: chatName,
                    text: messageText,
                    message_type: 'chat',
                    timestamp: timeEl ? timeEl.innerText : null,
                    timestamp_unix: Date.now(),
                    status: 'received',
                    media_info: null,
                    has_unread: !!unreadBadge
                };
            }
            
            // 1. MutationObserver on chat list for new messages
            function setupChatListObserver() {
                const chatList = document.querySelector('#pane-side') 
                              || document.querySelector('[data-testid="chat-list"]')
                              || document.querySelector('div[aria-label="Chat list"]');
                
                if (!chatList) {
                    console.warn('[WAS] Chat list not found, retrying in 2s...');
                    setTimeout(setupChatListObserver, 2000);
                    return;
                }
                
                const observer = new MutationObserver((mutations) => {
                    for (const mutation of mutations) {
                        // Check for attribute changes (unread badge appearing)
                        if (mutation.type === 'attributes' || mutation.type === 'childList') {
                            const target = mutation.target;
                            const row = target.closest('[role="listitem"], [role="row"]');
                            
                            if (row) {
                                const msgInfo = extractMessageFromRow(row);
                                if (msgInfo && msgInfo.has_unread) {
                                    window.__WAS_MESSAGE_QUEUE.push(msgInfo);
                                    console.log('[WAS] New message detected:', msgInfo.sender);
                                }
                            }
                        }
                        
                        // Check added nodes
                        if (mutation.addedNodes) {
                            mutation.addedNodes.forEach(node => {
                                if (node.nodeType === Node.ELEMENT_NODE) {
                                    const rows = node.querySelectorAll ? 
                                        [node, ...node.querySelectorAll('[role="listitem"], [role="row"]')] : [];
                                    rows.forEach(row => {
                                        const msgInfo = extractMessageFromRow(row);
                                        if (msgInfo && msgInfo.has_unread) {
                                            window.__WAS_MESSAGE_QUEUE.push(msgInfo);
                                            console.log('[WAS] New message (added node):', msgInfo.sender);
                                        }
                                    });
                                }
                            });
                        }
                    }
                });
                
                observer.observe(chatList, {
                    childList: true,
                    subtree: true,
                    attributes: true,
                    attributeFilter: ['data-testid', 'aria-label']
                });
                
                console.log('[WAS] Chat list observer installed');
                window.__WAS_CHAT_OBSERVER = observer;
            }
            
            // 2. Intercept Notification API to capture browser notifications
            function interceptNotifications() {
                const OriginalNotification = window.Notification;
                
                window.Notification = function(title, options) {
                    // Queue the notification as a message
                    const msgInfo = {
                        id: generateId(),
                        from_me: false,
                        sender: title,
                        text: options?.body || '',
                        message_type: 'chat',
                        timestamp: new Date().toLocaleTimeString(),
                        timestamp_unix: Date.now(),
                        status: 'received',
                        media_info: null,
                        source: 'notification'
                    };
                    
                    const uniqueKey = title + ':' + (options?.body || '');
                    if (!window.__WAS_PROCESSED_IDS.has(uniqueKey)) {
                        window.__WAS_PROCESSED_IDS.add(uniqueKey);
                        window.__WAS_MESSAGE_QUEUE.push(msgInfo);
                        console.log('[WAS] Notification intercepted:', title);
                    }
                    
                    // Still show the original notification
                    return new OriginalNotification(title, options);
                };
                
                // Copy static properties
                Object.setPrototypeOf(window.Notification, OriginalNotification);
                window.Notification.permission = OriginalNotification.permission;
                window.Notification.requestPermission = OriginalNotification.requestPermission.bind(OriginalNotification);
                
                console.log('[WAS] Notification API intercepted');
            }
            
            // 3. Provide a drain function for the Rust side
            window.__WAS_DRAIN_QUEUE = function() {
                const messages = window.__WAS_MESSAGE_QUEUE.splice(0);
                return messages;
            };
            
            // 4. Provide a function to check listener status
            window.__WAS_LISTENER_STATUS = function() {
                return {
                    active: window.__WAS_LISTENER_ACTIVE,
                    queue_size: window.__WAS_MESSAGE_QUEUE.length,
                    processed_count: window.__WAS_PROCESSED_IDS.size,
                    has_chat_observer: !!window.__WAS_CHAT_OBSERVER
                };
            };
            
            // Initialize
            setupChatListObserver();
            interceptNotifications();
            
            return { status: 'installed' };
        })();
        "##;

        let result = page.evaluate(listener_script).await?;
        let status: serde_json::Value = result.into_value().unwrap_or_default();

        let status_str = status
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        match status_str {
            "installed" => info!("Message listener installed successfully"),
            "already_active" => debug!("Message listener already active"),
            _ => debug!("Message listener status: {}", status_str),
        }

        Ok(())
    }

    /// Reset the listener injection flag (call after page reload/reconnect)
    pub fn reset_listener(&self) {
        self.listener_injected.store(false, Ordering::SeqCst);
    }

    async fn navigate_to_chat(&self, page: &Page, phone: &str) -> Result<()> {
        let phone_number: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        debug!("Navigating to chat: {}", phone_number);

        let script = format!(
            r##"(function() {{
                var pLdr = document.querySelectorAll('#phoneLoaderParent');
                if(pLdr.length == 0) {{
                    document.querySelector('#pane-side').innerHTML += '<div id="phoneLoaderParent"></div>';
                }}
                document.querySelector('#phoneLoaderParent').innerHTML = '<a id="phoneLoader" href="https://api.whatsapp.com/send?phone={}" style="display:none"></a>';
                document.querySelector('#phoneLoader').click();
                document.querySelector('#phoneLoaderParent').remove();
            }})();"##,
            phone_number
        );

        page.evaluate(script.as_str()).await?;
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

        // Check for invalid phone
        let check = r##"(function() {
            var d = document.querySelector('#app div[data-animate-modal-popup="true"] div[data-animate-modal-body="true"] div');
            if (d && d.innerHTML.trim() === 'Phone number shared via url is invalid.') {
                var btns = document.querySelectorAll('#app div[data-animate-modal-popup="true"] button');
                if (btns.length > 0) btns[btns.length - 1].click();
                return 'invalid';
            }
            return 'ok';
        })();"##;

        if let Ok(result) = page.evaluate(check).await {
            if result.into_value::<String>().unwrap_or_default() == "invalid" {
                return Err(anyhow::anyhow!("Invalid phone number"));
            }
        }

        self.wait_for_element(
            page,
            r##"#app #main footer div[aria-placeholder="Type a message"]"##,
            10000,
        )
        .await
    }

    async fn wait_for_element(&self, page: &Page, selector: &str, timeout_ms: u64) -> Result<()> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        while start.elapsed() < timeout {
            let script = format!(
                "document.querySelector('{}') !== null",
                selector.replace('\'', "\\'")
            );
            if let Ok(result) = page.evaluate(script.as_str()).await {
                if result.into_value::<bool>().unwrap_or(false) {
                    return Ok(());
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
        Err(anyhow::anyhow!("Timeout waiting for: {}", selector))
    }

    async fn click_attach_button(&self, page: &Page) -> Result<()> {
        let script = r##"(function() {
            var btn = document.querySelector('button[title="Attach"]') ||
                      document.querySelector('button[aria-label*="Attach"]') ||
                      document.querySelector('span[data-icon="plus"]')?.closest('button') ||
                      document.querySelector('div[title="Attach"]');
            if (btn) { btn.click(); return true; }
            var result = document.evaluate("//button[@title='Attach']", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
            if (result.singleNodeValue) { result.singleNodeValue.click(); return true; }
            return false;
        })();"##;

        let result = page.evaluate(script).await?;
        if !result.into_value::<bool>().unwrap_or(false) {
            return Err(anyhow::anyhow!("Failed to find attach button"));
        }
        Ok(())
    }

    async fn send_text_only(&self, page: &Page, text: &str) -> Result<()> {
        debug!("Sending text message");

        let input = r##"#app #main footer div[aria-placeholder="Type a message"]"##;
        self.wait_for_element(page, input, 10000).await?;

        for (i, line) in text.split('\n').enumerate() {
            let escaped = serde_json::to_string(line).unwrap_or_else(|_| "\"\"".to_string());
            let script = format!(
                r##"(function() {{
                    var el = document.querySelector('#app #main footer div[aria-placeholder="Type a message"]');
                    if (!el) return false;
                    el.focus();
                    document.execCommand('insertText', false, {});
                    return true;
                }})();"##,
                escaped
            );
            page.evaluate(script.as_str()).await?;

            if i < text.split('\n').count() - 1 {
                page.evaluate(r##"(function() {
                    var el = document.querySelector('#app #main footer div[aria-placeholder="Type a message"]');
                    if (el) { el.focus(); document.execCommand('insertLineBreak'); }
                })();"##).await?;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

        let send = r##"(function() {
            var btn = document.querySelector('button[aria-label="Send"]') ||
                      document.querySelector('span[data-icon="send"]')?.closest('button');
            if (btn) { btn.click(); return true; }
            return false;
        })();"##;

        if !page
            .evaluate(send)
            .await?
            .into_value::<bool>()
            .unwrap_or(false)
        {
            return Err(anyhow::anyhow!("Send button not found"));
        }

        info!("Text sent successfully");
        Ok(())
    }

    /// Enable file chooser interception to prevent native dialog from opening
    async fn enable_file_chooser_intercept(&self, page: &Page) -> Result<()> {
        use chromiumoxide::cdp::browser_protocol::page::SetInterceptFileChooserDialogParams;
        page.execute(
            SetInterceptFileChooserDialogParams::builder()
                .enabled(true)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build intercept params: {}", e))?,
        )
        .await?;
        debug!("File chooser interception enabled");
        Ok(())
    }

    /// Disable file chooser interception
    async fn disable_file_chooser_intercept(&self, page: &Page) -> Result<()> {
        use chromiumoxide::cdp::browser_protocol::page::SetInterceptFileChooserDialogParams;
        let _ = page
            .execute(
                SetInterceptFileChooserDialogParams::builder()
                    .enabled(false)
                    .build()
                    .map_err(|e| anyhow::anyhow!("Failed to build intercept params: {}", e))?,
            )
            .await;
        debug!("File chooser interception disabled");
        Ok(())
    }

    async fn send_image_or_video(
        &self,
        page: &Page,
        file_path: &str,
        caption: Option<&str>,
    ) -> Result<()> {
        debug!("Sending image/video: {}", file_path);

        // Get absolute path first
        let abs_path = std::fs::canonicalize(file_path)?;
        let abs_path_str = abs_path.to_string_lossy().to_string();

        // Step 1: Enable file chooser interception to prevent native dialog
        self.enable_file_chooser_intercept(page).await?;

        // Step 2: Click attach button
        self.click_attach_button(page).await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Step 3: Click "Photos & videos" menu item
        let click_media = r##"(function() {
            var item = document.querySelector('[role="menuitem"][aria-label="Photos & videos"]') ||
                       document.querySelector('[role="menuitem"][aria-label*="Photo"]') ||
                       document.querySelector('li[role="menuitem"] span[data-icon="attach-image"]')?.closest('[role="menuitem"]') ||
                       document.querySelector('button[aria-label="Photos & videos"]') ||
                       document.querySelector('span[data-icon="attach-image"]')?.closest('button');
            if (item) { item.click(); return true; }
            return false;
        })();"##;

        let clicked = page
            .evaluate(click_media)
            .await?
            .into_value::<bool>()
            .unwrap_or(false);
        if !clicked {
            self.disable_file_chooser_intercept(page).await?;
            return Err(anyhow::anyhow!("Failed to click Photos & videos menu item"));
        }

        // Step 4: Wait for file input to appear
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Step 5: Set files on the input
        let selectors = [
            r##"input[accept="image/*,video/mp4,video/3gpp,video/quicktime"]"##,
            r##"input[accept="image/*"]"##,
            "body > input[type=\"file\"]",
            "body > input",
        ];

        let mut upload_success = false;
        for selector in &selectors {
            if self
                .set_file_input_files(page, selector, &abs_path_str)
                .await
                .is_ok()
            {
                upload_success = true;
                debug!("File set via CDP on selector: {}", selector);
                break;
            }
        }

        if !upload_success {
            for selector in &selectors {
                if self.upload_file(page, selector, file_path).await.is_ok() {
                    upload_success = true;
                    debug!("File set via JS on selector: {}", selector);
                    break;
                }
            }
        }

        // Disable interception
        self.disable_file_chooser_intercept(page).await?;

        if !upload_success {
            return Err(anyhow::anyhow!(
                "Could not set file on any image/video input"
            ));
        }

        // Wait for preview to load (Send button appears) instead of fixed delay
        self.wait_for_element(page, r##"div[aria-label="Send"]"##, 10000)
            .await?;

        self.add_caption_and_send(page, caption).await?;
        info!("Image/video sent");
        Ok(())
    }

    async fn send_document(&self, page: &Page, file_path: &str, text: Option<&str>) -> Result<()> {
        debug!("Sending document: {}", file_path);

        // Get absolute path first
        let abs_path = std::fs::canonicalize(file_path)?;
        let abs_path_str = abs_path.to_string_lossy().to_string();

        // Step 1: Enable file chooser interception to prevent native dialog
        self.enable_file_chooser_intercept(page).await?;

        // Step 2: Click attach button to reveal the menu
        self.click_attach_button(page).await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Step 3: Click "Document" menu item
        let click_document = r##"(function() {
            var item = document.querySelector('[role="menuitem"][aria-label="Document"]') ||
                       document.querySelector('[role="menuitem"][aria-label*="Document"]') ||
                       document.querySelector('li[role="menuitem"] span[data-icon="attach-document"]')?.closest('[role="menuitem"]') ||
                       document.querySelector('button[aria-label="Document"]') ||
                       document.querySelector('span[data-icon="attach-document"]')?.closest('button');
            if (item) { item.click(); return true; }
            return false;
        })();"##;

        let clicked = page
            .evaluate(click_document)
            .await?
            .into_value::<bool>()
            .unwrap_or(false);
        if !clicked {
            self.disable_file_chooser_intercept(page).await?;
            return Err(anyhow::anyhow!("Failed to click Document menu item"));
        }

        // Step 4: Wait for file input to appear after clicking Document
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Step 5: Set files on the input that was created
        let selectors = [
            r##"input[accept="*"]"##,
            r##"input[type="file"][accept="*"]"##,
            r##"body > input[type="file"]"##,
        ];

        let mut upload_success = false;
        for selector in &selectors {
            if self
                .set_file_input_files(page, selector, &abs_path_str)
                .await
                .is_ok()
            {
                upload_success = true;
                debug!("File set via CDP on selector: {}", selector);
                break;
            }
        }

        if !upload_success {
            for selector in &selectors {
                if self.upload_file(page, selector, file_path).await.is_ok() {
                    upload_success = true;
                    debug!("File set via JS on selector: {}", selector);
                    break;
                }
            }
        }

        // Disable interception
        self.disable_file_chooser_intercept(page).await?;

        if !upload_success {
            return Err(anyhow::anyhow!("Could not set file on any document input"));
        }

        // Wait for preview to load (Send button appears) instead of fixed delay
        self.wait_for_element(page, r##"div[aria-label="Send"]"##, 10000)
            .await?;

        // Documents also have a caption field like images - use the same add_caption_and_send method
        self.add_caption_and_send(page, text).await?;

        info!("Document sent");
        Ok(())
    }

    /// Set files on a file input element using CDP (equivalent to Selenium SendKeys on file input)
    async fn set_file_input_files(
        &self,
        page: &Page,
        selector: &str,
        file_path: &str,
    ) -> Result<()> {
        use chromiumoxide::cdp::browser_protocol::dom::{
            QuerySelectorParams, SetFileInputFilesParams,
        };

        // First, get the document root
        let doc = page
            .execute(chromiumoxide::cdp::browser_protocol::dom::GetDocumentParams::default())
            .await?;
        let root_id = doc.root.node_id;

        // Query for the file input element
        let query_result = page
            .execute(
                QuerySelectorParams::builder()
                    .node_id(root_id)
                    .selector(selector)
                    .build()
                    .map_err(|e| anyhow::anyhow!("Failed to build query: {}", e))?,
            )
            .await?;

        let node_id = query_result.node_id;
        if *node_id.inner() == 0 {
            return Err(anyhow::anyhow!(
                "File input element not found: {}",
                selector
            ));
        }

        // Set the file on the input element
        page.execute(
            SetFileInputFilesParams::builder()
                .files(vec![file_path.to_string()])
                .node_id(node_id)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build set files params: {}", e))?,
        )
        .await?;

        debug!("File set on input element: {}", file_path);
        Ok(())
    }

    async fn upload_file(&self, page: &Page, selector: &str, file_path: &str) -> Result<()> {
        let content = tokio::fs::read(file_path).await?;
        let base64 = base64::engine::general_purpose::STANDARD.encode(&content);
        let name = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        let mime = mime_guess::from_path(file_path)
            .first_or_octet_stream()
            .to_string();

        let script = format!(
            r##"(function() {{
                var el = document.querySelector('{}');
                if (!el) return false;
                var bytes = atob('{}');
                var arr = new Uint8Array(bytes.length);
                for (var i = 0; i < bytes.length; i++) arr[i] = bytes.charCodeAt(i);
                var file = new File([arr], '{}', {{ type: '{}' }});
                var dt = new DataTransfer();
                dt.items.add(file);
                el.files = dt.files;
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }})();"##,
            selector,
            base64,
            name.replace('\'', "\\'"),
            mime
        );

        if !page
            .evaluate(script.as_str())
            .await?
            .into_value::<bool>()
            .unwrap_or(false)
        {
            return Err(anyhow::anyhow!("Failed to upload file"));
        }
        Ok(())
    }

    async fn add_caption_and_send(&self, page: &Page, caption: Option<&str>) -> Result<()> {
        // The preview dialog should already be loaded (caller waits for Send button)
        // Small delay for UI to stabilize
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Add caption if provided
        if let Some(text) = caption {
            if !text.is_empty() {
                // Find the caption input in the media preview dialog
                // It's a contenteditable div, similar to the main chat input
                let find_caption_input = r#"(function() {
                        // Look for the caption input in the preview - it's inside the modal/preview area
                        var inputs = document.querySelectorAll('div[contenteditable="true"][role="textbox"]');
                        for (var i = 0; i < inputs.length; i++) {
                            var input = inputs[i];
                            // The caption input usually has "Type a message" or "Add a caption" as aria-label
                            var label = input.getAttribute('aria-label') || '';
                            var placeholder = input.getAttribute('aria-placeholder') || '';
                            if (label.includes('message') || label.includes('caption') || 
                                placeholder.includes('message') || placeholder.includes('caption')) {
                                return true;
                            }
                        }
                        return false;
                    })();"#;

                let has_input = page
                    .evaluate(find_caption_input)
                    .await?
                    .into_value::<bool>()
                    .unwrap_or(false);
                debug!("Caption input found: {}", has_input);

                // Type the caption text
                for (i, line) in text.split('\n').enumerate() {
                    let escaped =
                        serde_json::to_string(line).unwrap_or_else(|_| "\"\"".to_string());
                    let script = format!(
                        r##"(function() {{
                            // Find the caption contenteditable input
                            var inputs = document.querySelectorAll('div[contenteditable="true"][role="textbox"]');
                            var el = null;
                            for (var i = 0; i < inputs.length; i++) {{
                                var input = inputs[i];
                                var label = input.getAttribute('aria-label') || '';
                                var placeholder = input.getAttribute('aria-placeholder') || '';
                                if (label.includes('message') || label.includes('caption') || 
                                    placeholder.includes('message') || placeholder.includes('caption')) {{
                                    el = input;
                                    break;
                                }}
                            }}
                            if (!el) return false;
                            el.focus();
                            document.execCommand('insertText', false, {});
                            return true;
                        }})();"##,
                        escaped
                    );
                    page.evaluate(script.as_str()).await?;

                    // Add newline if not the last line
                    if i < text.split('\n').count() - 1 {
                        page.evaluate(r##"(function() {
                            var inputs = document.querySelectorAll('div[contenteditable="true"][role="textbox"]');
                            var el = null;
                            for (var i = 0; i < inputs.length; i++) {
                                var input = inputs[i];
                                var label = input.getAttribute('aria-label') || '';
                                var placeholder = input.getAttribute('aria-placeholder') || '';
                                if (label.includes('message') || label.includes('caption') || 
                                    placeholder.includes('message') || placeholder.includes('caption')) {
                                    el = input;
                                    break;
                                }
                            }
                            if (el) { el.focus(); document.execCommand('insertLineBreak'); }
                        })();"##).await?;
                    }
                }

                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }
        }

        // Click the Send button to send the media with caption
        let send_script = r##"(function() {
            // Look for the send button in the media preview
            var btn = document.querySelector('div[aria-label="Send"]') ||
                      document.querySelector('span[data-icon="send"]')?.closest('div[role="button"]') ||
                      document.querySelector('span[data-icon="send"]')?.parentElement;
            if (btn) { 
                btn.click(); 
                return true; 
            }
            return false;
        })();"##;

        let sent = page
            .evaluate(send_script)
            .await?
            .into_value::<bool>()
            .unwrap_or(false);
        if !sent {
            return Err(anyhow::anyhow!(
                "Failed to click send button in media preview"
            ));
        }

        debug!("Media send button clicked");
        Ok(())
    }

    fn get_content_type(&self, path: &str) -> String {
        mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string()
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
        let timeout = timeout_ms.unwrap_or(60000);

        if phone.is_empty() {
            return Err(anyhow::anyhow!("Phone number required"));
        }

        let has_text = text.map(|t| !t.is_empty()).unwrap_or(false);
        let has_file = attachment_path.is_some();

        if !has_text && !has_file {
            return Err(anyhow::anyhow!("Text or attachment required"));
        }

        if let Some(path) = attachment_path {
            if !Path::new(path).exists() {
                return Err(anyhow::anyhow!("File not found: {}", path));
            }
        }

        let _permit = tokio::time::timeout(
            std::time::Duration::from_millis(timeout),
            self.message_queue.acquire(),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Queue timeout"))?
        .map_err(|e| anyhow::anyhow!("Queue error: {}", e))?;

        let page = self.get_page().await?;

        if !self.check_authorization(&page).await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        // Send the message
        self.navigate_to_chat(&page, phone).await?;

        match (attachment_path, text) {
            (None, Some(msg)) if !msg.is_empty() => {
                self.send_text_only(&page, msg).await?;
            }
            (Some(path), caption) => {
                let mime = self.get_content_type(path);
                if mime.contains("image") || mime.contains("video") {
                    self.send_image_or_video(&page, path, caption).await?;
                } else {
                    self.send_document(&page, path, caption).await?;
                }
            }
            _ => return Err(anyhow::anyhow!("Invalid parameters")),
        }

        info!("Message sent to {}", phone);
        Ok(())
    }

    // ========================================================================
    // DOM-Based Chat/Message Reading
    // ========================================================================

    /// Get list of visible chats from WhatsApp sidebar
    async fn get_chat_list(&self) -> Result<Vec<crate::models::chat::ChatInfo>> {
        let page = self.get_page().await?;

        if !self.check_authorization(&page).await? {
            return Err(anyhow::anyhow!(
                "Not authorized - please scan QR code first"
            ));
        }

        // New implementation based on WhatsApp Web Feb 2026 DOM structure
        // Discovered through Playwright MCP inspection
        let script = r##"
        (function() {
            const chats = [];
            const seen = new Set();
            
            // === TRY TO ACCESS WHATSAPP INTERNAL STORE FOR PROPER JIDs ===
            // WhatsApp Web stores chat data in internal modules accessible via window.Store
            // This gives us proper JIDs (like 919123456789@c.us or 120363123456@g.us)
            let storeChats = null;
            try {
                if (window.Store && window.Store.Chat) {
                    storeChats = window.Store.Chat.getModelsArray ? 
                        window.Store.Chat.getModelsArray() : 
                        (window.Store.Chat._models || window.Store.Chat.models || []);
                    console.log('[WAS] Found Store.Chat with', storeChats?.length || 0, 'chats');
                }
            } catch (e) {
                console.log('[WAS] Store.Chat not accessible:', e.message);
            }
            
            // Build a map of name -> JID from Store if available
            const nameToJid = new Map();
            if (storeChats && storeChats.length > 0) {
                for (const chat of storeChats) {
                    try {
                        const jid = chat.id?._serialized || chat.id?.toString?.();
                        const chatName = chat.name || chat.contact?.pushname || chat.contact?.name;
                        if (jid && chatName) {
                            nameToJid.set(chatName, jid);
                        }
                        // Also map by formatted name for phone numbers
                        if (chat.contact?.formattedName) {
                            nameToJid.set(chat.contact.formattedName, jid);
                        }
                    } catch (e) {}
                }
                console.log('[WAS] Built nameToJid map with', nameToJid.size, 'entries');
            }
            
            // WhatsApp uses grid[aria-label="Chat list"] with role="row" children
            const chatList = document.querySelector('[aria-label="Chat list"], [data-testid="chat-list"]');
            if (!chatList) {
                console.error('[WAS] Chat list container not found');
                return [];
            }
            
            const chatRows = chatList.querySelectorAll('[role="row"]');
            console.log('[WAS] Found', chatRows.length, 'chat rows');
            
            chatRows.forEach((row, idx) => {
                try {
                    // === GET NAME from first gridcell or span with title ===
                    // Structure: row > gridcell > generic > img + content
                    // The name is typically in a span with title attribute or in a specific gridcell
                    let name = null;
                    
                    // Method 1: Look for span with title attribute (most reliable)
                    const titleSpan = row.querySelector('span[title]');
                    if (titleSpan) {
                        name = titleSpan.getAttribute('title') || titleSpan.innerText;
                    }
                    
                    // Method 2: Look for gridcell containing the name
                    if (!name) {
                        const gridcells = row.querySelectorAll('[role="gridcell"]');
                        for (const cell of gridcells) {
                            const text = cell.innerText?.split('\n')[0]?.trim();
                            // First gridcell with substantial text is usually the name
                            if (text && text.length > 0 && text.length < 100 && 
                                !text.match(/^\d{1,2}:\d{2}/) && 
                                !text.match(/^(Yesterday|Today|Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday)/) &&
                                !text.includes('unread')) {
                                name = text;
                                break;
                            }
                        }
                    }
                    
                    // Method 3: Parse from row's accessible name
                    if (!name) {
                        const ariaLabel = row.getAttribute('aria-label') || '';
                        // Format: "Name Timestamp Message [UnreadCount]"
                        // First token before time/day is the name
                        const parts = ariaLabel.split(/\s+(\d{1,2}:\d{2}|Yesterday|Today|Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday|\d{1,2}\/\d{1,2}\/\d{2,4})\s+/);
                        if (parts.length > 0 && parts[0]) {
                            name = parts[0].trim();
                        }
                    }
                    
                    if (!name || name === 'Loading…' || name === 'Archived') return;
                    
                    // Deduplicate
                    if (seen.has(name)) return;
                    seen.add(name);
                    
                    // === GET TIMESTAMP ===
                    let timestamp = null;
                    
                    // Look for the timestamp pattern in elements
                    const allText = row.innerText || '';
                    const timestampPatterns = [
                        /(\d{1,2}:\d{2}(?:\s*[AP]M)?)/,           // "20:25" or "8:30 PM"
                        /(Yesterday|Today)/,
                        /(Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday)/,
                        /(\d{1,2}\/\d{1,2}\/\d{2,4})/             // "16/02/2026"
                    ];
                    
                    for (const pattern of timestampPatterns) {
                        const match = allText.match(pattern);
                        if (match) {
                            timestamp = match[1];
                            break;
                        }
                    }
                    
                    // === GET LAST MESSAGE ===
                    let lastMsg = null;
                    let lastMsgSender = null;
                    let isGroup = false;
                    
                    // === GROUP DETECTION ===
                    // Check multiple possible group icon selectors
                    const groupIconSelectors = [
                        '[data-icon="default-group"]',
                        '[data-icon="group"]',
                        '[data-icon="community"]',
                        '[data-testid="default-group"]',
                        '[data-testid="group"]',
                        'img[src*="groups"]',
                        'img[src*="group-icon"]'
                    ];
                    
                    for (const selector of groupIconSelectors) {
                        if (row.querySelector(selector)) {
                            isGroup = true;
                            break;
                        }
                    }
                    
                    // Also check any data-icon attribute containing "group"
                    if (!isGroup) {
                        const allIcons = row.querySelectorAll('[data-icon]');
                        for (const icon of allIcons) {
                            const iconName = icon.getAttribute('data-icon') || '';
                            if (iconName.toLowerCase().includes('group')) {
                                isGroup = true;
                                break;
                            }
                        }
                    }
                    
                    // Get message from specific DOM elements
                    // Look for spans that contain the message text (not name, not timestamp)
                    const allSpans = row.querySelectorAll('span');
                    let foundName = false;
                    let foundTimestamp = false;
                    
                    for (const span of allSpans) {
                        const text = span.innerText?.trim();
                        if (!text || text.length === 0) continue;
                        
                        // Skip the name
                        if (text === name || span.getAttribute('title') === name) {
                            foundName = true;
                            continue;
                        }
                        
                        // Skip timestamps
                        if (/^\d{1,2}:\d{2}(\s*[AP]M)?$/.test(text) ||
                            /^(Yesterday|Today|Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday)$/.test(text) ||
                            /^\d{1,2}\/\d{1,2}\/\d{2,4}$/.test(text)) {
                            foundTimestamp = true;
                            continue;
                        }
                        
                        // Skip unread count
                        if (/^\d+$/.test(text) || text.includes('unread message')) continue;
                        
                        // Skip very short text that might be icons or status
                        if (text.length < 2) continue;
                        
                        // This should be the message - clean up multiline text first
                        if (foundName || foundTimestamp) {
                            // Normalize: replace newlines around colon with proper separator
                            // "Sender\n:\nMessage" -> "Sender: Message"
                            let cleanText = text.replace(/\s*\n\s*:\s*\n\s*/g, ': ').replace(/\n/g, ' ').trim();
                            
                            // Check for sender prefix pattern: "SenderName: message" or "SenderName : message"
                            // Valid senders: personal names (Akash, Dharmendra), phone numbers, or (You)/You
                            // Pattern: starts with name/phone, then colon+space, then message
                            const senderMatch = cleanText.match(/^([A-Za-z\u0900-\u097F\+][A-Za-z0-9\u0900-\u097F\s\+\-\(\)]{0,25}):\s+(.+)$/s);
                            
                            if (senderMatch) {
                                const potentialSender = senderMatch[1].trim();
                                const potentialMsg = senderMatch[2].trim();
                                
                                // Validate sender - must look like a personal name or phone number
                                // Personal name: starts with capital, 1-2 words, only letters
                                // Phone: digits with optional +, spaces, dashes
                                // Special: (You), You, ~ (tilde prefix for some names)
                                const isPhoneNumber = /^[\+\d\s\-\(\)]+$/.test(potentialSender) && potentialSender.length >= 3;
                                const isPersonalName = /^[A-Z\u0900-\u097F~][a-zA-Z\u0900-\u097F]*(\s[A-Z\u0900-\u097F][a-zA-Z\u0900-\u097F]*)?$/.test(potentialSender);
                                const isYou = potentialSender === '(You)' || potentialSender === 'You';
                                
                                // Additional validation: sender should be SHORT (max 25 chars) and NOT look like message content
                                const isTooLong = potentialSender.length > 25;
                                const looksLikeContent = potentialSender.includes('http') || 
                                                         potentialSender.includes('.com') ||
                                                         /\d{5,}/.test(potentialSender.replace(/[\s\-\(\)\+]/g, '')) && !isPhoneNumber;
                                
                                if ((isPhoneNumber || isPersonalName || isYou) && !isTooLong && !looksLikeContent && potentialMsg.length > 0) {
                                    lastMsgSender = potentialSender === '(You)' ? 'You' : potentialSender;
                                    lastMsg = potentialMsg;
                                    // If we found a valid sender, this is definitely a group!
                                    if (!isYou) {
                                        isGroup = true;
                                    }
                                    break;
                                }
                            }
                            
                            // No valid sender pattern - use the clean text as message
                            if (!lastMsg) {
                                lastMsg = cleanText;
                                break;
                            }
                        }
                    }
                    
                    // Fallback: if still no message, get from row content more aggressively
                    if (!lastMsg) {
                        const rowText = row.innerText || '';
                        // Clean up the row text - normalize newlines around colons
                        const cleanRowText = rowText.replace(/\s*\n\s*:\s*\n\s*/g, ': ');
                        const lines = cleanRowText.split('\n').filter(l => l.trim());
                        // Look for a line that's not the name and not a timestamp
                        for (const line of lines) {
                            const trimmed = line.trim();
                            if (trimmed === name) continue;
                            if (/^\d{1,2}:\d{2}/.test(trimmed)) continue;
                            if (/^(Yesterday|Today|Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday)$/.test(trimmed)) continue;
                            if (/^\d+$/.test(trimmed)) continue;
                            if (/^\d{1,2}\/\d{1,2}\/\d{2,4}$/.test(trimmed)) continue;
                            if (trimmed.includes('unread message')) continue;
                            if (trimmed.length < 2) continue;
                            lastMsg = trimmed;
                            break;
                        }
                    }
                    
                    // === GET UNREAD COUNT ===
                    let unreadCount = 0;
                    const rowLabel = row.getAttribute('aria-label') || '';
                    const unreadMatch = rowLabel.match(/(\d+)\s+unread\s+message/i);
                    if (unreadMatch) {
                        unreadCount = parseInt(unreadMatch[1]) || 0;
                    }
                    
                    // Also check for badge element
                    if (unreadCount === 0) {
                        const badgeEl = row.querySelector('[aria-label*="unread"]');
                        if (badgeEl) {
                            const badgeText = badgeEl.innerText || badgeEl.getAttribute('aria-label') || '';
                            const match = badgeText.match(/(\d+)/);
                            if (match) unreadCount = parseInt(match[1]) || 0;
                        }
                    }
                    
                    // === GET AVATAR URL ===
                    let avatarUrl = null;
                    
                    // Look for img element in the row
                    const imgs = row.querySelectorAll('img');
                    for (const img of imgs) {
                        const src = img.src || img.getAttribute('src');
                        // WhatsApp profile pics are hosted on pps.whatsapp.net
                        if (src && (src.includes('pps.whatsapp.net') || src.includes('blob:') || 
                                    (src.startsWith('https://') && !src.includes('emoji')))) {
                            avatarUrl = src;
                            break;
                        }
                    }
                    
                    // === GET CHAT ID ===
                    // Try multiple approaches to extract the chat ID
                    let chatId = null;
                    
                    // Method 0: Look up from WhatsApp Store (most reliable if available)
                    if (nameToJid.has(name)) {
                        chatId = nameToJid.get(name);
                        // If JID ends with @g.us, it's definitely a group
                        if (chatId && chatId.endsWith('@g.us')) {
                            isGroup = true;
                        }
                    }
                    
                    // Method 1: data-id attribute (may not work in current WhatsApp version)
                    if (!chatId) {
                        let el = row;
                        for (let i = 0; i < 10 && el; i++) {
                            const dataId = el.getAttribute && el.getAttribute('data-id');
                            if (dataId && dataId.includes('@')) {
                                chatId = dataId;
                                if (chatId.endsWith('@g.us')) isGroup = true;
                                break;
                            }
                            el = el.parentElement;
                        }
                    }
                    
                    // Method 2: Extract phone number from name if it's a phone number
                    if (!chatId && name) {
                        // Names like "+91 97389 68141" or "919876543210"
                        const phoneClean = name.replace(/[\s\-\(\)]/g, '').replace(/^\+/, '');
                        if (/^\d{10,15}$/.test(phoneClean)) {
                            chatId = phoneClean + '@c.us';
                        }
                    }
                    
                    // Method 3: For groups without JID, use group: prefix
                    if (!chatId && isGroup) {
                        // For groups, use a group-style ID
                        chatId = 'group:' + name.replace(/[^a-zA-Z0-9\u0900-\u097F]/g, '_').substring(0, 50);
                    }
                    
                    // Method 4: Fallback to name-based ID for contacts without phone
                    if (!chatId) {
                        chatId = 'name:' + name;
                    }
                    
                    // === ADDITIONAL GROUP DETECTION ===
                    // Check if chat has "(You)" prefix which indicates self-chat or group
                    if (!isGroup && lastMsg && lastMsg.startsWith('(You)')) {
                        // Self chat
                        lastMsg = lastMsg.replace(/^\(You\)\s*/, '');
                    }
                    
                    // Check for group icon one more time
                    if (!isGroup) {
                        const groupIcon = row.querySelector('[data-icon="default-group"], [data-icon="group"]');
                        if (groupIcon) isGroup = true;
                    }
                    
                    // === PINNED/MUTED STATUS ===
                    let isPinned = null;
                    let isMuted = null;
                    
                    const pinnedIcon = row.querySelector('[data-icon="pinned"], [data-testid="pinned"]');
                    if (pinnedIcon) isPinned = true;
                    
                    const mutedIcon = row.querySelector('[data-icon="muted"], [data-testid="muted"]');
                    if (mutedIcon) isMuted = true;
                    
                    chats.push({
                        id: chatId,
                        name: name,
                        last_message: lastMsg,
                        last_message_sender: lastMsgSender,
                        timestamp: timestamp,
                        unread_count: unreadCount,
                        is_group: isGroup,
                        avatar_url: avatarUrl,
                        is_pinned: isPinned,
                        is_muted: isMuted
                    });
                    
                } catch(e) {
                    console.error('[WAS] Error parsing chat row', idx, ':', e);
                }
            });
            
            console.log('[WAS] Parsed', chats.length, 'chats successfully');
            return chats;
        })();
        "##;

        let result = page.evaluate(script).await?;
        let chats: Vec<crate::models::chat::ChatInfo> = result.into_value().unwrap_or_default();

        Ok(chats)
    }

    /// Get messages from the currently open chat or open a specific chat first
    async fn get_messages(
        &self,
        chat_id: &str,
        limit: Option<u32>,
        load_more: bool,
    ) -> Result<crate::models::chat::MessageListResponse> {
        let page = self.get_page().await?;

        if !self.check_authorization(&page).await? {
            return Err(anyhow::anyhow!(
                "Not authorized - please scan QR code first"
            ));
        }

        // Determine how to navigate to the chat
        let chat_name = if chat_id.starts_with("name:") {
            // Extract name from "name:Contact Name" format
            Some(chat_id.strip_prefix("name:").unwrap_or(chat_id))
        } else if chat_id.contains('@') {
            // It's a JID like "919876543210@c.us" - extract phone and also try name click
            None
        } else if chat_id.chars().all(|c| c.is_ascii_digit() || c == '+') {
            // Pure phone number
            None
        } else {
            // Assume it's a contact name
            Some(chat_id)
        };

        let mut navigated = false;

        // Try to click on the chat by name in the sidebar first (most reliable)
        if let Some(name) = chat_name {
            let click_script = format!(
                r##"(function() {{
                    const rows = document.querySelectorAll('[role="listitem"], [role="row"], [data-testid="cell-frame-container"]');
                    for (const row of rows) {{
                        const nameEl = row.querySelector('[data-testid="cell-frame-title"] span') ||
                                       row.querySelector('span[title]') ||
                                       row.querySelector('[dir="auto"]');
                        if (nameEl && (nameEl.title === "{0}" || nameEl.innerText === "{0}" || 
                            nameEl.getAttribute('title') === "{0}")) {{
                            row.click();
                            return true;
                        }}
                    }}
                    return false;
                }})();"##,
                name.replace('"', "\\\"").replace('\n', " ")
            );

            let clicked = page
                .evaluate(click_script.as_str())
                .await?
                .into_value::<bool>()
                .unwrap_or(false);

            if clicked {
                debug!("Clicked on chat by name: {}", name);
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                navigated = true;
            }
        }

        // If name click didn't work, try phone number navigation
        if !navigated {
            let phone = if chat_id.contains('@') {
                chat_id.split('@').next().unwrap_or(chat_id)
            } else if chat_id.starts_with("name:") {
                "" // Can't navigate by phone if we only have name
            } else {
                chat_id
            };

            if !phone.is_empty() && phone.chars().all(|c| c.is_ascii_digit() || c == '+') {
                self.navigate_to_chat(&page, phone).await?;
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            }
        }

        // Load more messages if requested
        if load_more {
            let scroll_script = r##"
            (function() {
                const container = document.querySelector('[role="application"]') ||
                                  document.querySelector('[data-testid="conversation-panel-messages"]')?.parentElement;
                if (container) {
                    container.scrollTop = 0;
                    return true;
                }
                return false;
            })();
            "##;
            let _ = page.evaluate(scroll_script).await;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        // Extract messages from DOM
        let limit_val = limit.unwrap_or(50);
        let extract_script = format!(
            r##"
            (function() {{
                const messages = [];
                const msgElements = document.querySelectorAll('[data-id]');
                
                msgElements.forEach(el => {{
                    try {{
                        const dataId = el.getAttribute('data-id');
                        if (!dataId || !dataId.includes('@')) return;
                        
                        // Parse message ID format: fromMe_chatId_msgId
                        const fromMe = dataId.startsWith('true_');
                        
                        // Get message text from span.copyable-text (WhatsApp Web 2026 structure)
                        let text = null;
                        const copyableSpans = el.querySelectorAll('span.copyable-text');
                        for (const span of copyableSpans) {{
                            const t = span.innerText?.trim();
                            // Skip if it looks like a timestamp (HH:MM format) or empty
                            if (t && t.length > 0 && !t.match(/^\d{{1,2}}:\d{{2}}$/)) {{
                                text = t;
                                break;
                            }}
                        }}
                        // Fallback: Try _ao3e class (WhatsApp's internal class for message text)
                        if (!text) {{
                            const ao3e = el.querySelector('span._ao3e');
                            if (ao3e) {{
                                const t = ao3e.innerText?.trim();
                                if (t && !t.match(/^\d{{1,2}}:\d{{2}}$/)) {{
                                    text = t;
                                }}
                            }}
                        }}
                        
                        // Get timestamp from data-pre-plain-text attribute
                        const preTextEl = el.querySelector('[data-pre-plain-text]');
                        let timestamp = null;
                        if (preTextEl) {{
                            const preText = preTextEl.getAttribute('data-pre-plain-text');
                            if (preText) {{
                                timestamp = preText.replace(/[\[\]]/g, '').trim();
                            }}
                        }}
                        // Fallback: get time from msg-meta
                        if (!timestamp) {{
                            const timeEl = el.querySelector('[data-testid="msg-meta"] span');
                            if (timeEl) {{
                                timestamp = timeEl.innerText;
                            }}
                        }}
                        
                        // Get sender for group chats
                        const senderEl = el.querySelector('[data-testid="msg-container"] span[aria-label]');
                        const sender = senderEl ? senderEl.getAttribute('aria-label')?.replace(':', '') : null;
                        
                        // Determine message type
                        let msgType = 'chat';
                        if (el.querySelector('[data-testid="image-thumb"]') || el.querySelector('img[src*="blob:"]')) {{
                            msgType = 'image';
                        }} else if (el.querySelector('[data-testid="video-thumb"]') || el.querySelector('video')) {{
                            msgType = 'video';
                        }} else if (el.querySelector('[data-testid="audio-play"]') || el.querySelector('audio')) {{
                            msgType = 'audio';
                        }} else if (el.querySelector('[data-testid="document-thumb"]') || el.querySelector('[data-icon="audio-document"]')) {{
                            msgType = 'document';
                        }} else if (el.querySelector('[data-testid="location"]')) {{
                            msgType = 'location';
                        }} else if (el.querySelector('[data-testid="contact-card"]')) {{
                            msgType = 'contact';
                        }} else if (el.querySelector('button[title*="Sticker"]') || el.querySelector('[data-testid="sticker"]')) {{
                            msgType = 'sticker';
                        }}
                        
                        // Get status (delivered, read, etc.)
                        let status = null;
                        const statusEl = el.querySelector('[data-testid="msg-dblcheck"]') ||
                                         el.querySelector('[data-testid="msg-check"]') ||
                                         el.querySelector('[data-icon="msg-dblcheck"]') ||
                                         el.querySelector('[data-icon="msg-check"]');
                        if (statusEl) {{
                            const icon = statusEl.getAttribute('data-icon') || statusEl.getAttribute('data-testid');
                            if (icon && icon.includes('dblcheck')) {{
                                status = el.querySelector('[data-icon="msg-dblcheck-ack"]') ? 'read' : 'delivered';
                            }} else {{
                                status = 'sent';
                            }}
                        }}
                        
                        // Get media info for non-text messages
                        let mediaInfo = null;
                        if (msgType !== 'chat') {{
                            const docName = el.querySelector('[data-testid="document-thumb"] + div span');
                            if (docName) mediaInfo = docName.innerText;
                        }}
                        
                        messages.push({{
                            id: dataId,
                            from_me: fromMe,
                            sender: sender,
                            text: text,
                            message_type: msgType,
                            timestamp: timestamp,
                            timestamp_unix: null,
                            status: status,
                            media_info: mediaInfo
                        }});
                    }} catch(e) {{
                        // Skip problematic messages
                    }}
                }});
                
                // Limit results
                return messages.slice(-{});
            }})();
            "##,
            limit_val
        );

        let result = page.evaluate(extract_script.as_str()).await?;
        let messages: Vec<crate::models::chat::MessageInfo> =
            result.into_value().unwrap_or_default();

        // Get chat name from header
        let name_script = r##"
        (function() {
            const header = document.querySelector('[data-testid="conversation-info-header-chat-title"]') ||
                           document.querySelector('#main header span[title]');
            return header ? header.innerText || header.title : null;
        })();
        "##;
        let chat_name: Option<String> = page
            .evaluate(name_script)
            .await
            .ok()
            .and_then(|r| r.into_value().ok());

        let total = messages.len();

        Ok(crate::models::chat::MessageListResponse {
            chat_id: chat_id.to_string(),
            chat_name,
            messages,
            total,
            has_more: total >= limit_val as usize,
        })
    }

    /// Watch for new incoming messages (returns new messages since last check)
    ///
    /// This uses an event-driven approach with MutationObserver and Notification API
    /// interception instead of polling the DOM. Much more efficient for real-time
    /// message detection.
    async fn watch_messages(&self) -> Result<Vec<crate::models::chat::MessageInfo>> {
        let page = self.get_page().await?;

        if !self.check_authorization(&page).await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        // Ensure the message listener is injected
        self.inject_message_listener(&page).await?;

        // Drain the message queue (returns and clears all queued messages)
        let drain_script = r##"
        (function() {
            if (typeof window.__WAS_DRAIN_QUEUE === 'function') {
                return window.__WAS_DRAIN_QUEUE();
            }
            // Fallback: listener not ready, return empty
            return [];
        })();
        "##;

        let result = page.evaluate(drain_script).await?;
        let messages: Vec<crate::models::chat::MessageInfo> =
            result.into_value().unwrap_or_default();

        if !messages.is_empty() {
            debug!("Drained {} messages from queue", messages.len());
        }

        Ok(messages)
    }

    /// Send typing indicator to a chat
    async fn send_typing(
        &self,
        chat_id: &str,
        state: crate::models::chat::TypingState,
    ) -> Result<()> {
        let page = self.get_page().await?;

        if !self.check_authorization(&page).await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        // Navigate to the chat first
        self.navigate_to_chat(&page, chat_id).await?;

        // Simulate typing by focusing and optionally typing/clearing
        let is_composing = state == crate::models::chat::TypingState::Composing;

        let script = format!(
            r##"
        (function() {{
            const input = document.querySelector('div[contenteditable="true"][data-tab="10"]') ||
                          document.querySelector('#main footer div[contenteditable="true"]') ||
                          document.querySelector('div[aria-placeholder="Type a message"]');
            
            if (!input) {{
                console.error('[WAS] Message input not found for typing');
                return false;
            }}
            
            // Focus the input to show typing state
            input.focus();
            
            if ({}) {{
                // Composing: type a space then delete it to trigger composing state
                const event = new InputEvent('input', {{ bubbles: true, cancelable: true }});
                input.textContent = ' ';
                input.dispatchEvent(event);
            }} else {{
                // Paused: clear and blur
                input.textContent = '';
                input.blur();
            }}
            
            return true;
        }})();
        "##,
            is_composing
        );

        let result: bool = page.evaluate(script).await?.into_value().unwrap_or(false);

        if !result {
            return Err(anyhow::anyhow!("Failed to send typing indicator"));
        }

        debug!("Sent typing indicator: {:?} to {}", state, chat_id);
        Ok(())
    }

    /// Mark all messages in a chat as read
    async fn mark_read(&self, chat_id: &str) -> Result<u32> {
        let page = self.get_page().await?;

        if !self.check_authorization(&page).await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        // Navigate to the chat - this automatically marks messages as read
        self.navigate_to_chat(&page, chat_id).await?;

        // Wait briefly for read receipts to be sent
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Get unread count (should be 0 now)
        let script = r##"
        (function() {
            // Check if there are any unread indicators in the current chat
            const unreadBadge = document.querySelector('#main [aria-label*="unread"]');
            return unreadBadge ? parseInt(unreadBadge.textContent) || 0 : 0;
        })();
        "##;

        let remaining: u32 = page.evaluate(script).await?.into_value().unwrap_or(0);

        debug!(
            "Marked messages as read in {}, remaining: {}",
            chat_id, remaining
        );
        Ok(remaining)
    }

    /// Get presence/online status for a contact
    async fn get_presence(&self, chat_id: &str) -> Result<crate::models::chat::PresenceInfo> {
        let page = self.get_page().await?;

        if !self.check_authorization(&page).await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        // Navigate to the chat to see presence
        self.navigate_to_chat(&page, chat_id).await?;

        // Wait for header to load
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        let script = r##"
        (function() {
            // Look for presence info in the chat header
            const header = document.querySelector('#main header');
            if (!header) return { status: 'unknown', last_seen: null, last_seen_hidden: false };
            
            // Look for "online", "typing", or "last seen" text
            const subtitleEl = header.querySelector('span[title]') ||
                               header.querySelector('span[dir="auto"]');
            
            // Also check for explicit status spans
            const statusSpans = header.querySelectorAll('span');
            let statusText = null;
            
            for (const span of statusSpans) {
                const text = span.innerText?.toLowerCase() || '';
                if (text.includes('online') || text.includes('typing') || 
                    text.includes('last seen') || text.includes('click here')) {
                    statusText = span.innerText;
                    break;
                }
            }
            
            if (!statusText && subtitleEl) {
                // Get the second line (status) from header
                const allText = header.innerText || '';
                const lines = allText.split('\n').filter(l => l.trim());
                if (lines.length > 1) {
                    statusText = lines[1];
                }
            }
            
            if (!statusText) {
                return { status: 'unknown', last_seen: null, last_seen_hidden: false };
            }
            
            const lowerStatus = statusText.toLowerCase();
            
            if (lowerStatus.includes('online')) {
                return { status: 'online', last_seen: null, last_seen_hidden: false };
            } else if (lowerStatus.includes('typing')) {
                return { status: 'online', last_seen: 'typing', last_seen_hidden: false };
            } else if (lowerStatus.includes('last seen')) {
                return { status: 'offline', last_seen: statusText, last_seen_hidden: false };
            } else if (lowerStatus.includes('click here') || lowerStatus.includes('tap here')) {
                // Privacy setting hides last seen
                return { status: 'unknown', last_seen: null, last_seen_hidden: true };
            }
            
            return { status: 'unknown', last_seen: null, last_seen_hidden: false };
        })();
        "##;

        #[derive(serde::Deserialize)]
        struct RawPresence {
            status: String,
            last_seen: Option<String>,
            last_seen_hidden: bool,
        }

        let raw: RawPresence = page
            .evaluate(script)
            .await?
            .into_value()
            .unwrap_or(RawPresence {
                status: "unknown".to_string(),
                last_seen: None,
                last_seen_hidden: false,
            });

        let status = match raw.status.as_str() {
            "online" => crate::models::chat::PresenceStatus::Online,
            "offline" => crate::models::chat::PresenceStatus::Offline,
            _ => crate::models::chat::PresenceStatus::Unknown,
        };

        Ok(crate::models::chat::PresenceInfo {
            chat_id: chat_id.to_string(),
            status,
            last_seen: raw.last_seen,
            last_seen_hidden: raw.last_seen_hidden,
        })
    }

    /// Get detailed group information
    async fn get_group_info(&self, group_id: &str) -> Result<crate::models::chat::GroupInfo> {
        let page = self.get_page().await?;

        if !self.check_authorization(&page).await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        // Navigate to the group
        self.navigate_to_chat(&page, group_id).await?;

        // Click on the group header to open info panel
        let click_script = r##"
        (function() {
            const header = document.querySelector('#main header');
            if (header) {
                header.click();
                return true;
            }
            return false;
        })();
        "##;

        let clicked: bool = page
            .evaluate(click_script)
            .await?
            .into_value()
            .unwrap_or(false);
        if !clicked {
            return Err(anyhow::anyhow!("Failed to open group info panel"));
        }

        // Wait for panel to open
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        let script = r##"
        (function() {
            // Find the info panel (usually on the right side)
            const panel = document.querySelector('[data-testid="conversation-panel-wrapper"]') ||
                          document.querySelector('span[data-testid="conversation-info-header"]')?.closest('div[tabindex]');
            
            if (!panel) {
                // Try alternate: look for the group name in any open panel
                const panels = document.querySelectorAll('[role="application"], [data-animate-drawer-content="true"]');
                for (const p of panels) {
                    if (p.innerText?.includes('participants') || p.innerText?.includes('Group info')) {
                        panel = p;
                        break;
                    }
                }
            }
            
            const info = {
                name: null,
                description: null,
                avatar_url: null,
                created_at: null,
                created_by: null,
                participant_count: 0,
                participants: [],
                is_announce: false,
                is_locked: false,
                invite_link: null
            };
            
            // Get group name from header
            const nameEl = document.querySelector('#main header span[title]');
            info.name = nameEl?.getAttribute('title') || nameEl?.innerText;
            
            // Get avatar
            const avatar = document.querySelector('#main header img[src*="pps.whatsapp.net"]');
            info.avatar_url = avatar?.src;
            
            // Try to find participant list
            const participantSection = document.querySelector('[data-testid="participants-section"]') ||
                                       document.evaluate("//span[contains(text(), 'participants')]", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue?.closest('div');
            
            if (participantSection) {
                // Count participants from text like "50 participants"
                const countMatch = participantSection.innerText?.match(/(\d+)\s*participant/i);
                if (countMatch) {
                    info.participant_count = parseInt(countMatch[1]);
                }
                
                // Get individual participants
                const participantRows = participantSection.querySelectorAll('[role="listitem"], [role="row"], [data-testid*="participant"]');
                for (const row of participantRows) {
                    const nameSpan = row.querySelector('span[title]');
                    const name = nameSpan?.getAttribute('title') || nameSpan?.innerText;
                    if (name) {
                        const isAdmin = row.innerText?.toLowerCase().includes('admin') || false;
                        const isOwner = row.innerText?.toLowerCase().includes('group admin') || false;
                        info.participants.push({
                            id: name,
                            name: name,
                            phone: null,
                            is_admin: isAdmin,
                            is_owner: isOwner
                        });
                    }
                }
            }
            
            // Get description
            const descSection = document.querySelector('[data-testid="group-description"]') ||
                               document.evaluate("//span[contains(text(), 'Add group description')]", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue?.closest('div');
            if (descSection && !descSection.innerText?.includes('Add group')) {
                info.description = descSection.innerText?.trim();
            }
            
            return info;
        })();
        "##;

        #[derive(serde::Deserialize)]
        struct RawGroupInfo {
            name: Option<String>,
            description: Option<String>,
            avatar_url: Option<String>,
            created_at: Option<String>,
            created_by: Option<String>,
            participant_count: u32,
            participants: Vec<crate::models::chat::GroupParticipant>,
            is_announce: bool,
            is_locked: bool,
            invite_link: Option<String>,
        }

        let raw: RawGroupInfo = page
            .evaluate(script)
            .await?
            .into_value()
            .unwrap_or(RawGroupInfo {
                name: None,
                description: None,
                avatar_url: None,
                created_at: None,
                created_by: None,
                participant_count: 0,
                participants: vec![],
                is_announce: false,
                is_locked: false,
                invite_link: None,
            });

        // Close the panel by clicking elsewhere or pressing Escape
        let _ = page.evaluate("document.body.click();").await;

        Ok(crate::models::chat::GroupInfo {
            id: group_id.to_string(),
            name: raw.name.unwrap_or_else(|| group_id.to_string()),
            description: raw.description,
            avatar_url: raw.avatar_url,
            created_at: raw.created_at,
            created_by: raw.created_by,
            participant_count: raw.participant_count,
            participants: raw.participants,
            is_announce: raw.is_announce,
            is_locked: raw.is_locked,
            invite_link: raw.invite_link,
        })
    }

    /// Get contact profile information
    async fn get_contact_info(&self, contact_id: &str) -> Result<crate::models::chat::ContactInfo> {
        let page = self.get_page().await?;

        if !self.check_authorization(&page).await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        // Navigate to the contact's chat
        self.navigate_to_chat(&page, contact_id).await?;

        // Click on header to open contact info
        let _ = page
            .evaluate(r#"document.querySelector('#main header')?.click();"#)
            .await;

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        let script = r##"
        (function() {
            const info = {
                name: null,
                push_name: null,
                phone: null,
                avatar_url: null,
                status: null,
                is_business: false,
                business_name: null,
                business_category: null,
                is_blocked: false
            };
            
            // Get name from header
            const nameEl = document.querySelector('#main header span[title]');
            info.name = nameEl?.getAttribute('title') || nameEl?.innerText;
            
            // Get avatar
            const avatar = document.querySelector('#main header img[src*="pps.whatsapp.net"]');
            info.avatar_url = avatar?.src;
            
            // Look for contact info panel
            const panels = document.querySelectorAll('[role="application"], [data-animate-drawer-content="true"]');
            for (const panel of panels) {
                const text = panel.innerText || '';
                
                // Check for business indicators
                if (text.includes('Business account') || text.includes('Catalog')) {
                    info.is_business = true;
                }
                
                // Look for "About" section
                const aboutMatch = text.match(/About\n(.+)/);
                if (aboutMatch) {
                    info.status = aboutMatch[1]?.split('\n')[0];
                }
                
                // Look for phone number
                const phoneMatch = text.match(/(\+\d[\d\s\-]+)/);
                if (phoneMatch) {
                    info.phone = phoneMatch[1];
                }
                
                // Check if blocked
                if (text.includes('Unblock') || text.includes('blocked')) {
                    info.is_blocked = true;
                }
            }
            
            return info;
        })();
        "##;

        #[derive(serde::Deserialize)]
        struct RawContactInfo {
            name: Option<String>,
            push_name: Option<String>,
            phone: Option<String>,
            avatar_url: Option<String>,
            status: Option<String>,
            is_business: bool,
            business_name: Option<String>,
            business_category: Option<String>,
            is_blocked: bool,
        }

        let raw: RawContactInfo =
            page.evaluate(script)
                .await?
                .into_value()
                .unwrap_or(RawContactInfo {
                    name: None,
                    push_name: None,
                    phone: None,
                    avatar_url: None,
                    status: None,
                    is_business: false,
                    business_name: None,
                    business_category: None,
                    is_blocked: false,
                });

        // Close the panel
        let _ = page.evaluate("document.body.click();").await;

        Ok(crate::models::chat::ContactInfo {
            id: contact_id.to_string(),
            name: raw.name,
            push_name: raw.push_name,
            phone: raw.phone,
            avatar_url: raw.avatar_url,
            status: raw.status,
            is_business: raw.is_business,
            business_name: raw.business_name,
            business_category: raw.business_category,
            is_blocked: raw.is_blocked,
        })
    }

    /// Send a reaction (emoji) to a message
    async fn send_reaction(&self, chat_id: &str, message_id: &str, emoji: &str) -> Result<()> {
        let page = self.get_page().await?;

        if !self.check_authorization(&page).await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        // Navigate to the chat
        self.navigate_to_chat(&page, chat_id).await?;

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Find the message and double-click or right-click to open reaction menu
        let script = format!(
            r##"
        (async function() {{
            // Find the message by ID or by searching message bubbles
            const messages = document.querySelectorAll('[data-id="{msg_id}"], [data-testid="msg-container"]');
            
            let targetMsg = null;
            for (const msg of messages) {{
                const dataId = msg.getAttribute('data-id');
                if (dataId && (dataId.includes('{msg_id}') || dataId === '{msg_id}')) {{
                    targetMsg = msg;
                    break;
                }}
            }}
            
            if (!targetMsg) {{
                // Try to find by index or content
                const allMsgs = document.querySelectorAll('.message-in, .message-out, [data-testid="msg-container"]');
                if (allMsgs.length > 0) {{
                    // Default to last message if ID not found
                    targetMsg = allMsgs[allMsgs.length - 1];
                }}
            }}
            
            if (!targetMsg) {{
                return {{ success: false, error: 'Message not found' }};
            }}
            
            // Double-click to show reaction quick menu
            const dblClick = new MouseEvent('dblclick', {{
                bubbles: true,
                cancelable: true,
                view: window
            }});
            targetMsg.dispatchEvent(dblClick);
            
            // Wait for reaction menu
            await new Promise(r => setTimeout(r, 500));
            
            // Look for reaction buttons
            const reactionButtons = document.querySelectorAll('[data-testid="reaction-btn"], [aria-label="React"]');
            
            if (reactionButtons.length > 0) {{
                // Click on the reaction menu
                reactionButtons[0].click();
                await new Promise(r => setTimeout(r, 300));
            }}
            
            // Try to find and click the specific emoji
            const emoji = '{emoji}';
            if (emoji) {{
                // Look for emoji in reaction picker
                const emojiButtons = document.querySelectorAll('[data-emoji="{emoji}"], button[aria-label*="{emoji}"]');
                for (const btn of emojiButtons) {{
                    if (btn.innerText?.includes(emoji) || btn.getAttribute('data-emoji') === emoji) {{
                        btn.click();
                        return {{ success: true }};
                    }}
                }}
                
                // Fallback: try to type the emoji in search
                const searchInput = document.querySelector('[data-testid="emoji-search"], input[placeholder*="Search"]');
                if (searchInput) {{
                    searchInput.value = emoji;
                    searchInput.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    await new Promise(r => setTimeout(r, 300));
                    
                    const firstResult = document.querySelector('[data-testid="emoji-result"]');
                    if (firstResult) {{
                        firstResult.click();
                        return {{ success: true }};
                    }}
                }}
            }}
            
            return {{ success: false, error: 'Could not send reaction' }};
        }})();
        "##,
            msg_id = message_id,
            emoji = emoji
        );

        let result: serde_json::Value = page
            .evaluate(script)
            .await?
            .into_value()
            .unwrap_or_default();

        if result
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            debug!(
                "Sent reaction {} to message {} in {}",
                emoji, message_id, chat_id
            );
            Ok(())
        } else {
            let error = result
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            Err(anyhow::anyhow!("Failed to send reaction: {}", error))
        }
    }

    /// Send a reply to a specific message
    async fn send_reply(&self, chat_id: &str, quoted_message_id: &str, text: &str) -> Result<()> {
        let page = self.get_page().await?;

        if !self.check_authorization(&page).await? {
            return Err(anyhow::anyhow!("Not authorized"));
        }

        // Navigate to the chat
        self.navigate_to_chat(&page, chat_id).await?;

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Find the message and click reply
        let reply_script = format!(
            r##"
        (async function() {{
            // Find the message
            const messages = document.querySelectorAll('[data-id*="{msg_id}"], [data-testid="msg-container"]');
            
            let targetMsg = null;
            for (const msg of messages) {{
                const dataId = msg.getAttribute('data-id');
                if (dataId && dataId.includes('{msg_id}')) {{
                    targetMsg = msg;
                    break;
                }}
            }}
            
            if (!targetMsg) {{
                return {{ success: false, error: 'Message not found' }};
            }}
            
            // Hover over message to show menu
            targetMsg.dispatchEvent(new MouseEvent('mouseover', {{ bubbles: true }}));
            await new Promise(r => setTimeout(r, 200));
            
            // Click the down arrow to open context menu
            const menuBtn = targetMsg.querySelector('[data-testid="down-context"], [data-icon="down-context"]') ||
                            targetMsg.parentElement?.querySelector('[data-testid="down-context"]');
            
            if (menuBtn) {{
                menuBtn.click();
                await new Promise(r => setTimeout(r, 300));
            }} else {{
                // Try right-click
                targetMsg.dispatchEvent(new MouseEvent('contextmenu', {{
                    bubbles: true,
                    cancelable: true,
                    view: window,
                    button: 2
                }}));
                await new Promise(r => setTimeout(r, 300));
            }}
            
            // Find and click "Reply" option
            const menuItems = document.querySelectorAll('[role="menuitem"], [data-testid*="reply"], li[data-animate-dropdown-item]');
            for (const item of menuItems) {{
                if (item.innerText?.toLowerCase().includes('reply')) {{
                    item.click();
                    return {{ success: true }};
                }}
            }}
            
            // Alt: look for reply icon
            const replyIcon = document.querySelector('[data-icon="reply"], [data-testid="reply"]');
            if (replyIcon) {{
                replyIcon.click();
                return {{ success: true }};
            }}
            
            return {{ success: false, error: 'Reply option not found' }};
        }})();
        "##,
            msg_id = quoted_message_id
        );

        let result: serde_json::Value = page
            .evaluate(reply_script)
            .await?
            .into_value()
            .unwrap_or_default();

        if !result
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            let error = result
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return Err(anyhow::anyhow!("Failed to initiate reply: {}", error));
        }

        // Wait for reply context to appear
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Now type and send the message
        self.send_text_only(&page, text).await?;

        debug!(
            "Sent reply to message {} in {}: {}",
            quoted_message_id, chat_id, text
        );
        Ok(())
    }
}
