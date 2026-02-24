//! Chat/Messaging Service
//!
//! Handles sending text messages and attachments via WhatsApp Web.
//! Based on proven .NET implementation patterns.

use crate::{
    browser::BrowserService,
    config::AppConfig,
};
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

        let script = r##"
        (function() {
            const chats = [];
            const chatRows = document.querySelectorAll('[data-testid="cell-frame-container"], div[role="listitem"], div[role="row"]');
            
            chatRows.forEach(row => {
                try {
                    // Try to get the chat data
                    const nameEl = row.querySelector('[data-testid="cell-frame-title"] span') ||
                                   row.querySelector('span[title]') ||
                                   row.querySelector('[dir="auto"]');
                    
                    if (!nameEl) return;
                    
                    const name = nameEl.innerText || nameEl.getAttribute('title') || '';
                    if (!name || name === 'Loading…') return;
                    
                    // Get last message - look for the message preview text under the chat name
                    let lastMsg = null;
                    
                    // Method 1: WhatsApp's last message status container
                    const lastMsgContainer = row.querySelector('[data-testid="last-msg-status"]');
                    if (lastMsgContainer) {
                        const parent = lastMsgContainer.closest('div');
                        if (parent) {
                            // Get text excluding status icons
                            const textSpans = parent.querySelectorAll('span');
                            for (const span of textSpans) {
                                if (!span.querySelector('[data-icon]') && span.innerText?.trim()) {
                                    lastMsg = span.innerText.trim();
                                    break;
                                }
                            }
                        }
                    }
                    
                    // Method 2: Look for the second row of text (first is name, second is message)
                    if (!lastMsg) {
                        const cellContainer = row.querySelector('[data-testid="cell-frame-container"]') || row;
                        const spans = cellContainer.querySelectorAll('span[dir="ltr"], span[dir="auto"]');
                        // Find spans that aren't the name and aren't timestamps
                        for (const span of spans) {
                            const text = span.innerText?.trim();
                            if (text && text !== name && !text.match(/^\d{1,2}:\d{2}/) && 
                                !text.match(/^(Yesterday|Today)/) && text.length > 0) {
                                // Skip if this is likely the name element
                                if (span.closest('[data-testid="cell-frame-title"]')) continue;
                                lastMsg = text;
                                break;
                            }
                        }
                    }
                    
                    // Method 3: Look for message text in secondary content area
                    if (!lastMsg) {
                        const secondaryEl = row.querySelector('[data-testid="cell-frame-secondary"]');
                        if (secondaryEl) {
                            lastMsg = secondaryEl.innerText?.trim() || null;
                        }
                    }
                    
                    // Get timestamp
                    const timeEl = row.querySelector('[data-testid="cell-frame-primary-detail"]') ||
                                   row.querySelectorAll('[dir="auto"]')[1];
                    const timestamp = timeEl ? timeEl.innerText : null;
                    
                    // Get unread count
                    const unreadEl = row.querySelector('[data-testid="icon-unread-count"]') ||
                                     row.querySelector('span[aria-label*="unread"]');
                    let unreadCount = 0;
                    if (unreadEl) {
                        const text = unreadEl.innerText || unreadEl.getAttribute('aria-label') || '';
                        const match = text.match(/\d+/);
                        unreadCount = match ? parseInt(match[0]) : 0;
                    }
                    
                    // Check if group (has group icon or multiple participants indicator)
                    const isGroup = row.querySelector('[data-icon="default-group"]') !== null ||
                                    name.includes('group') ||
                                    row.querySelector('[data-testid="group"]') !== null;
                    
                    // Get avatar URL
                    const avatarEl = row.querySelector('img[src*="pps.whatsapp.net"]');
                    const avatarUrl = avatarEl ? avatarEl.src : null;
                    
                    // Try to extract chat ID from multiple sources
                    let chatId = null;
                    
                    // Method 1: data-id attribute on row or parent
                    let el = row;
                    for (let i = 0; i < 5 && el; i++) {
                        const dataId = el.getAttribute('data-id');
                        if (dataId && dataId.includes('@')) {
                            chatId = dataId;
                            break;
                        }
                        el = el.parentElement;
                    }
                    
                    // Method 2: Look for data-id in child elements (deeper search)
                    if (!chatId) {
                        const allWithId = row.querySelectorAll('[data-id]');
                        for (const child of allWithId) {
                            const childDataId = child.getAttribute('data-id');
                            if (childDataId && childDataId.includes('@')) {
                                chatId = childDataId;
                                break;
                            }
                        }
                    }
                    
                    // Method 3: Look for phone in aria-label or title attributes
                    if (!chatId) {
                        const allElements = row.querySelectorAll('[aria-label], [title]');
                        for (const el of allElements) {
                            const attr = el.getAttribute('aria-label') || el.getAttribute('title') || '';
                            // Match phone patterns like +919876543210 or 919876543210
                            const phoneMatch = attr.match(/\+?(\d{10,15})/);
                            if (phoneMatch) {
                                chatId = phoneMatch[1] + '@c.us';
                                break;
                            }
                        }
                    }
                    
                    // Method 4: Extract phone number from name (handles formats like "+91 97389 68141")
                    if (!chatId) {
                        // Remove all non-digit characters except leading +
                        const cleanName = name.replace(/[^\d+]/g, '').replace(/^\+/, '');
                        if (cleanName.length >= 10 && cleanName.length <= 15 && /^\d+$/.test(cleanName)) {
                            chatId = cleanName + '@c.us';
                        }
                    }
                    
                    // Method 5: Use the name as a fallback identifier
                    // But prefix it to indicate it's a name-based ID
                    if (!chatId) {
                        chatId = 'name:' + name;
                    }
                    
                    chats.push({
                        id: chatId,
                        name: name,
                        last_message: lastMsg,
                        timestamp: timestamp,
                        unread_count: unreadCount,
                        is_group: isGroup,
                        avatar_url: avatarUrl
                    });
                } catch(e) {
                    // Skip problematic rows
                }
            });
            
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
}
