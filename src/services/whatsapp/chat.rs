//! Chat/Messaging Service
//!
//! Handles sending text messages and attachments via WhatsApp Web.

use crate::{browser::BrowserService, config::AppConfig};
use anyhow::Result;
use async_trait::async_trait;
use base64::Engine;
use chromiumoxide::page::Page;
use std::path::Path;
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
}

impl ChatService {
    pub fn new(config: Arc<AppConfig>, browser_service: Arc<BrowserService>) -> Self {
        Self {
            config,
            browser_service,
            message_queue: Semaphore::new(1),
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
}

impl ChatService {
    async fn navigate_to_chat(&self, page: &Page, chat_id: &str) -> Result<()> {
        debug!("Navigating to chat: {}", chat_id);

        // Determine the type of chat ID and handle accordingly
        // 1. "919876543210@c.us" -> phone number
        // 2. "120363123456@g.us" -> group JID
        // 3. "name:Contact Name" -> search by name
        // 4. "group:Group_Name" -> search by name
        // 5. Just digits -> phone number

        if chat_id.starts_with("name:") || chat_id.starts_with("group:") {
            // Navigate by searching the name in sidebar
            let search_name = if chat_id.starts_with("name:") {
                chat_id.strip_prefix("name:").unwrap_or(chat_id).to_string()
            } else {
                chat_id
                    .strip_prefix("group:")
                    .unwrap_or(chat_id)
                    .replace('_', " ")
            };
            return self.navigate_by_search(page, &search_name).await;
        }

        if chat_id.ends_with("@g.us") {
            // Group JID - try using Store or search by group name
            // First try to open via Store API
            let store_script = format!(
                r##"(async function() {{
                    try {{
                        if (window.Store && window.Store.Chat) {{
                            const chat = await window.Store.Chat.find('{jid}');
                            if (chat) {{
                                // Open the chat
                                if (window.WAPI && window.WAPI.openChat) {{
                                    await window.WAPI.openChat('{jid}');
                                    return 'ok';
                                }}
                                // Alternative: click the chat in the list
                                const chatEl = document.querySelector('[data-id="{jid}"]');
                                if (chatEl) {{
                                    chatEl.click();
                                    return 'ok';
                                }}
                            }}
                        }}
                    }} catch (e) {{
                        console.log('[WAS] Store navigation failed:', e);
                    }}
                    return 'needs_search';
                }})();"##,
                jid = chat_id
            );

            if let Ok(result) = page.evaluate(store_script).await {
                if result.into_value::<String>().unwrap_or_default() == "ok" {
                    // Wait for chat to load
                    return self
                        .wait_for_element(
                            page,
                            r##"#app #main footer div[aria-placeholder="Type a message"]"##,
                            10000,
                        )
                        .await;
                }
            }

            // Fallback: search by group name (extract from JID if we have it cached)
            // For now, we can't easily get group name from JID without Store access
            return Err(anyhow::anyhow!(
                "Cannot navigate to group {} - please use group name instead",
                chat_id
            ));
        }

        // Phone number - either @c.us suffix or just digits
        let phone_number: String = chat_id
            .replace("@c.us", "")
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();

        if phone_number.is_empty() {
            return Err(anyhow::anyhow!("Invalid chat ID: {}", chat_id));
        }

        debug!("Navigating to phone: {}", phone_number);

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

    /// Navigate to a chat by searching for the name in the sidebar
    async fn navigate_by_search(&self, page: &Page, name: &str) -> Result<()> {
        debug!("Searching for chat: {}", name);

        // Click on search box
        let search_script = r##"(function() {
            // Find and click the search box
            const searchBox = document.querySelector('[data-testid="chat-list-search"]') ||
                             document.querySelector('div[contenteditable="true"][role="textbox"][title="Search input textbox"]') ||
                             document.querySelector('div[contenteditable="true"][data-tab="3"]') ||
                             document.querySelector('[aria-label="Search input textbox"]');
            if (searchBox) {
                searchBox.click();
                searchBox.focus();
                return true;
            }
            // Try clicking the search button first
            const searchBtn = document.querySelector('[data-icon="search"]')?.closest('button') ||
                             document.querySelector('button[aria-label*="Search"]');
            if (searchBtn) {
                searchBtn.click();
                return true;
            }
            return false;
        })();"##;

        if !page
            .evaluate(search_script)
            .await?
            .into_value::<bool>()
            .unwrap_or(false)
        {
            return Err(anyhow::anyhow!("Could not find search box"));
        }

        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

        // Type the search query
        let escaped_name = serde_json::to_string(name).unwrap_or_else(|_| "\"\"".to_string());
        let type_script = format!(
            r##"(function() {{
                const searchBox = document.querySelector('[data-testid="chat-list-search"]') ||
                                 document.querySelector('div[contenteditable="true"][role="textbox"][title="Search input textbox"]') ||
                                 document.querySelector('div[contenteditable="true"][data-tab="3"]') ||
                                 document.querySelector('[aria-label="Search input textbox"]');
                if (searchBox) {{
                    searchBox.focus();
                    document.execCommand('selectAll', false, null);
                    document.execCommand('insertText', false, {});
                    return true;
                }}
                return false;
            }})();"##,
            escaped_name
        );

        if !page
            .evaluate(type_script)
            .await?
            .into_value::<bool>()
            .unwrap_or(false)
        {
            return Err(anyhow::anyhow!("Could not type in search box"));
        }

        // Wait for search results
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        // Click on the first search result that matches
        let escaped_name_for_match =
            serde_json::to_string(name).unwrap_or_else(|_| "\"\"".to_string());
        let click_result_script = format!(
            r##"(function() {{
                const searchName = {};
                const searchNameLower = searchName.toLowerCase();
                
                // Look for chat rows in search results
                const rows = document.querySelectorAll('[aria-label="Chat list"] [role="row"], [role="listitem"]');
                for (const row of rows) {{
                    const nameSpan = row.querySelector('span[title]');
                    const name = nameSpan?.getAttribute('title') || nameSpan?.innerText || row.innerText?.split('\n')[0];
                    if (name && name.toLowerCase().includes(searchNameLower)) {{
                        row.click();
                        return true;
                    }}
                }}
                
                // Also check search results section
                const searchResults = document.querySelectorAll('[data-testid="cell-frame-title"]');
                for (const result of searchResults) {{
                    const name = result.innerText || result.textContent;
                    if (name && name.toLowerCase().includes(searchNameLower)) {{
                        result.closest('[role="row"], [role="listitem"]')?.click() || result.click();
                        return true;
                    }}
                }}
                
                return false;
            }})();"##,
            escaped_name_for_match
        );

        if !page
            .evaluate(click_result_script)
            .await?
            .into_value::<bool>()
            .unwrap_or(false)
        {
            // Clear search and try clicking in chat list
            let _ = page
                .evaluate(
                    r##"(function() {
                const back = document.querySelector('[data-icon="x"]')?.closest('button') ||
                             document.querySelector('button[aria-label*="Cancel"]');
                if (back) back.click();
            })();"##,
                )
                .await;

            return Err(anyhow::anyhow!("Could not find chat: {}", name));
        }

        // Clear search box after navigation
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let _ = page
            .evaluate(
                r##"(function() {
            const back = document.querySelector('[data-icon="x"]')?.closest('button') ||
                         document.querySelector('[data-testid="x"]')?.closest('button');
            if (back) back.click();
        })();"##,
            )
            .await;

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
}
