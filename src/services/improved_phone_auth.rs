use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn, error};

/// Improved phone authentication service using MCP Playwright
pub struct ImprovedPhoneAuthService {
    page_url: String,
    timeout_config: TimeoutConfig,
}

#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub navigation: Duration,
    pub element_wait: Duration,
    pub code_detection: Duration,
    pub total_operation: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            navigation: Duration::from_secs(15),
            element_wait: Duration::from_secs(10),
            code_detection: Duration::from_secs(30),
            total_operation: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhoneAuthResult {
    pub success: bool,
    pub verification_code: Option<String>,
    pub error_message: Option<String>,
    pub debug_info: PhoneAuthDebugInfo,
}

#[derive(Debug, Clone)]
pub struct PhoneAuthDebugInfo {
    pub current_url: String,
    pub page_title: String,
    pub detected_screen: String,
    pub steps_completed: Vec<String>,
    pub error_details: Option<String>,
}

impl ImprovedPhoneAuthService {
    pub fn new() -> Self {
        Self {
            page_url: "https://web.whatsapp.com".to_string(),
            timeout_config: TimeoutConfig::default(),
        }
    }

    /// Main phone authentication method with improved error handling
    pub async fn authenticate_with_phone(&self, phone_number: &str) -> Result<PhoneAuthResult> {
        info!("🔐 Starting improved phone authentication for: {}", phone_number);
        
        let mut debug_info = PhoneAuthDebugInfo {
            current_url: String::new(),
            page_title: String::new(),
            detected_screen: "unknown".to_string(),
            steps_completed: Vec::new(),
            error_details: None,
        };

        // Use timeout for better error handling
        let result = timeout(
            self.timeout_config.total_operation,
            self.perform_authentication(phone_number, &mut debug_info)
        ).await;

        match result {
            Ok(Ok(verification_code)) => {
                info!("✅ Phone authentication successful, code extracted: {}", verification_code);
                Ok(PhoneAuthResult {
                    success: true,
                    verification_code: Some(verification_code),
                    error_message: None,
                    debug_info,
                })
            }
            Ok(Err(e)) => {
                error!("❌ Phone authentication failed: {}", e);
                debug_info.error_details = Some(e.to_string());
                Ok(PhoneAuthResult {
                    success: false,
                    verification_code: None,
                    error_message: Some(e.to_string()),
                    debug_info,
                })
            }
            Err(_) => {
                error!("⏰ Phone authentication timed out after {:?}", self.timeout_config.total_operation);
                debug_info.error_details = Some("Operation timeout".to_string());
                Ok(PhoneAuthResult {
                    success: false,
                    verification_code: None,
                    error_message: Some("Authentication timeout".to_string()),
                    debug_info,
                })
            }
        }
    }

    async fn perform_authentication(&self, phone_number: &str, debug_info: &mut PhoneAuthDebugInfo) -> Result<String> {
        // For now, let's implement a simplified version that can be tested
        // The real Playwright integration will be done via MCP tools in tests
        
        debug_info.steps_completed.push("phone_auth_started".to_string());
        debug_info.current_url = self.page_url.clone();
        
        let formatted_phone = self.format_phone_number(phone_number);
        info!("📱 Processing phone authentication for: {}", formatted_phone);
        
        // Simulate the authentication steps for now
        // This will be replaced with real browser automation in integration tests
        debug_info.steps_completed.push("phone_formatted".to_string());
        debug_info.steps_completed.push("ready_for_browser_automation".to_string());
        
        // Return a test code for now - in real implementation this will come from the browser
        Ok("TEST-ABCD".to_string())
    }

    fn format_phone_number(&self, phone: &str) -> String {
        if phone.starts_with('+') {
            phone.to_string()
        } else {
            format!("+{}", phone)
        }
    }

    /// Get debug information about the authentication process
    pub fn get_timeout_config(&self) -> &TimeoutConfig {
        &self.timeout_config
    }

    /// Validate phone number format
    pub fn validate_phone_number(&self, phone: &str) -> Result<String> {
        let formatted = self.format_phone_number(phone);
        
        // Basic validation
        if formatted.len() < 8 || formatted.len() > 15 {
            return Err(anyhow::anyhow!("Invalid phone number length"));
        }
        
        if !formatted.chars().skip(1).all(|c| c.is_ascii_digit()) {
            return Err(anyhow::anyhow!("Phone number contains invalid characters"));
        }
        
        Ok(formatted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_phone_number_formatting() {
        let service = ImprovedPhoneAuthService::new();
        
        assert_eq!(service.format_phone_number("1234567890"), "+1234567890");
        assert_eq!(service.format_phone_number("+1234567890"), "+1234567890");
        assert_eq!(service.format_phone_number("919501005734"), "+919501005734");
    }

    #[tokio::test]
    async fn test_timeout_configuration() {
        let service = ImprovedPhoneAuthService::new();
        
        assert_eq!(service.timeout_config.navigation, Duration::from_secs(15));
        assert_eq!(service.timeout_config.total_operation, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_phone_auth_structure() {
        let service = ImprovedPhoneAuthService::new();
        
        // This test ensures our structure is correct
        // Real tests will be added once Playwright integration is complete
        assert!(!service.page_url.is_empty());
    }
}
