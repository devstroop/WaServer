use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// MCP Playwright client for browser automation
#[derive(Debug, Clone)]
pub struct McpPlaywrightClient {
    client: Client,
    base_url: String,
}

/// Response from MCP Playwright browser snapshot
#[derive(Debug, Clone)]
pub struct BrowserSnapshot {
    pub content: String,
    pub elements: HashMap<String, ElementInfo>,
    pub page_title: String,
    pub current_url: String,
}

/// Information about a detected element
#[derive(Debug, Clone)]
pub struct ElementInfo {
    pub ref_id: String,
    pub element_type: String,
    pub text: Option<String>,
    pub visible: bool,
    pub clickable: bool,
}

impl McpPlaywrightClient {
    /// Create a new MCP Playwright client
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.unwrap_or_else(|| "http://localhost:3001".to_string()),
        }
    }

    /// Navigate to a URL
    pub async fn navigate(&self, url: &str) -> Result<()> {
        debug!("🌐 MCP: Navigating to {}", url);

        let payload = json!({
            "method": "tools/call",
            "params": {
                "name": "mcp_playwright_browser_navigate",
                "arguments": {
                    "url": url
                }
            }
        });

        let response = self.send_request(payload).await?;

        if response.get("error").is_some() {
            return Err(anyhow!("Navigation failed: {:?}", response.get("error")));
        }

        info!("✅ MCP: Successfully navigated to {}", url);
        Ok(())
    }

    /// Take a snapshot of the current page
    pub async fn snapshot(&self) -> Result<BrowserSnapshot> {
        debug!("📸 MCP: Taking browser snapshot");

        let payload = json!({
            "method": "tools/call",
            "params": {
                "name": "mcp_playwright_browser_snapshot",
                "arguments": {}
            }
        });

        let response = self.send_request(payload).await?;

        if let Some(error) = response.get("error") {
            return Err(anyhow!("Snapshot failed: {:?}", error));
        }

        // Parse the snapshot response
        let content = response
            .get("result")
            .and_then(|r| r.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        let page_title = response
            .get("result")
            .and_then(|r| r.get("title"))
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let current_url = response
            .get("result")
            .and_then(|r| r.get("url"))
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();

        // Extract elements from the snapshot
        let elements = self.parse_elements_from_snapshot(&content).await?;

        debug!(
            "✅ MCP: Snapshot captured - {} elements detected",
            elements.len()
        );

        Ok(BrowserSnapshot {
            content,
            elements,
            page_title,
            current_url,
        })
    }

    /// Click on an element by reference
    pub async fn click(&self, element_ref: &str, description: &str) -> Result<()> {
        debug!("🖱️  MCP: Clicking element: {}", description);

        let payload = json!({
            "method": "tools/call",
            "params": {
                "name": "mcp_playwright_browser_click",
                "arguments": {
                    "ref": element_ref,
                    "element": description
                }
            }
        });

        let response = self.send_request(payload).await?;

        if let Some(error) = response.get("error") {
            return Err(anyhow!("Click failed: {:?}", error));
        }

        info!("✅ MCP: Successfully clicked: {}", description);
        Ok(())
    }

    /// Type text into an element
    pub async fn type_text(&self, element_ref: &str, text: &str, description: &str) -> Result<()> {
        debug!("⌨️  MCP: Typing into element: {}", description);

        let payload = json!({
            "method": "tools/call",
            "params": {
                "name": "mcp_playwright_browser_type",
                "arguments": {
                    "ref": element_ref,
                    "element": description,
                    "text": text,
                    "submit": false
                }
            }
        });

        let response = self.send_request(payload).await?;

        if let Some(error) = response.get("error") {
            return Err(anyhow!("Type failed: {:?}", error));
        }

        info!("✅ MCP: Successfully typed into: {}", description);
        Ok(())
    }

    /// Wait for text to appear or disappear
    pub async fn wait_for_text(&self, text: &str, timeout_secs: u64) -> Result<bool> {
        debug!("⏳ MCP: Waiting for text: '{}'", text);

        let payload = json!({
            "method": "tools/call",
            "params": {
                "name": "mcp_playwright_browser_wait_for",
                "arguments": {
                    "text": text,
                    "time": timeout_secs
                }
            }
        });

        let response = self.send_request(payload).await?;

        if let Some(error) = response.get("error") {
            warn!("⚠️  MCP: Wait for text failed: {:?}", error);
            return Ok(false);
        }

        info!("✅ MCP: Text found: '{}'", text);
        Ok(true)
    }

    /// Send a request to the MCP server
    async fn send_request(&self, payload: Value) -> Result<Value> {
        let response = self
            .client
            .post(&format!("{}/rpc", self.base_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("MCP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("MCP server returned error: {}", response.status()));
        }

        let json_response: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse MCP response: {}", e))?;

        Ok(json_response)
    }

    /// Parse elements from the accessibility snapshot
    async fn parse_elements_from_snapshot(
        &self,
        content: &str,
    ) -> Result<HashMap<String, ElementInfo>> {
        let mut elements = HashMap::new();

        // For now, create some basic parsing logic
        // In a real implementation, we'd parse the accessibility tree properly

        // Look for common WhatsApp Web elements
        if content.contains("phone number") || content.contains("Phone number") {
            elements.insert(
                "phone_input".to_string(),
                ElementInfo {
                    ref_id: "phone_input".to_string(),
                    element_type: "input".to_string(),
                    text: Some("Phone number input".to_string()),
                    visible: true,
                    clickable: true,
                },
            );
        }

        if content.contains("Next") || content.contains("Continue") {
            elements.insert(
                "next_button".to_string(),
                ElementInfo {
                    ref_id: "next_button".to_string(),
                    element_type: "button".to_string(),
                    text: Some("Next button".to_string()),
                    visible: true,
                    clickable: true,
                },
            );
        }

        if content.contains("verification")
            || content.contains("code")
            || content.contains("Enter code")
        {
            elements.insert(
                "verification_code".to_string(),
                ElementInfo {
                    ref_id: "verification_code".to_string(),
                    element_type: "text".to_string(),
                    text: Some("Verification code".to_string()),
                    visible: true,
                    clickable: false,
                },
            );
        }

        if content.contains("QR") || content.contains("qr") {
            elements.insert(
                "qr_code".to_string(),
                ElementInfo {
                    ref_id: "qr_code".to_string(),
                    element_type: "image".to_string(),
                    text: Some("QR Code".to_string()),
                    visible: true,
                    clickable: false,
                },
            );
        }

        debug!("🔍 MCP: Parsed {} elements from snapshot", elements.len());
        Ok(elements)
    }

    /// Detect the current screen type based on snapshot content
    pub fn detect_screen_type(&self, snapshot: &BrowserSnapshot) -> String {
        let content = &snapshot.content.to_lowercase();

        if content.contains("qr") || content.contains("scan") {
            "qr_screen".to_string()
        } else if content.contains("phone") && content.contains("number") {
            "phone_screen".to_string()
        } else if content.contains("verification") || content.contains("code") {
            "verification_screen".to_string()
        } else if content.contains("chat") || content.contains("messages") {
            "authenticated_screen".to_string()
        } else {
            "unknown_screen".to_string()
        }
    }

    /// Extract verification code from screen
    pub fn extract_verification_code(&self, snapshot: &BrowserSnapshot) -> Option<String> {
        let content = &snapshot.content;

        // Look for patterns like "Enter the 6-digit code: 123456"
        if let Some(code_match) = extract_code_pattern(content) {
            debug!("🔑 MCP: Extracted verification code: {}", code_match);
            return Some(code_match);
        }

        // Check if there's a verification code element
        if let Some(code_element) = snapshot.elements.get("verification_code") {
            if let Some(text) = &code_element.text {
                if let Some(code) = extract_code_from_text(text) {
                    debug!("🔑 MCP: Found verification code in element: {}", code);
                    return Some(code);
                }
            }
        }

        debug!("❌ MCP: No verification code detected");
        None
    }
}

/// Extract verification code using regex patterns
fn extract_code_pattern(text: &str) -> Option<String> {
    // Look for 6-digit codes
    use regex::Regex;

    let patterns = [
        r"\b(\d{6})\b",       // 6 digits
        r"code[:\s]+(\d{6})", // "code: 123456"
        r"(\d{3})\s*(\d{3})", // "123 456"
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.captures(text) {
                if let Some(code) = captures.get(1) {
                    return Some(code.as_str().to_string());
                }
            }
        }
    }

    None
}

/// Extract code from element text
fn extract_code_from_text(text: &str) -> Option<String> {
    // Simple numeric extraction
    let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() >= 4 && digits.len() <= 8 {
        Some(digits)
    } else {
        None
    }
}
