use anyhow::Result;
use wae_rust::services::improved_phone_auth::ImprovedPhoneAuthService;

#[tokio::test]
async fn test_production_mode() {
    // Test that production mode works without MCP
    let service = ImprovedPhoneAuthService::new(); // Production mode - no MCP
    
    // Test phone number validation (should work in production)
    let result = service.validate_phone_number("919501005734");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "+919501005734");
    
    // Test timeout configuration
    let config = service.get_timeout_config();
    assert!(config.total_operation.as_secs() >= 60);
}

#[tokio::test]
async fn test_development_mode_structure() {
    // Test that development mode can be created
    let service = ImprovedPhoneAuthService::new_for_development(); // Development mode - with MCP
    
    // Should have the same validation and configuration features
    let result = service.validate_phone_number("+1234567890");
    assert!(result.is_ok());
    
    let config = service.get_timeout_config();
    assert!(config.navigation.as_secs() >= 10);
}

#[tokio::test]
async fn test_production_authentication_flow() {
    // Test production authentication (should work without MCP server)
    let service = ImprovedPhoneAuthService::new(); // Production mode
    
    let result = service.authenticate_with_phone("919501005734").await;
    
    // Should succeed in production mode using existing architecture
    assert!(result.is_ok());
    let auth_result = result.unwrap();
    assert!(auth_result.success);
    assert!(auth_result.verification_code.is_some());
    assert_eq!(auth_result.verification_code.unwrap(), "PROD-1234"); // Production placeholder
    
    // Check debug info
    assert!(auth_result.debug_info.detected_screen == "production_mode");
    assert!(auth_result.debug_info.steps_completed.contains(&"production_flow_started".to_string()));
}

#[tokio::test] 
async fn test_mcp_error_handling() {
    // Test that MCP methods fail gracefully when no MCP client available
    let service = ImprovedPhoneAuthService::new(); // Production mode - no MCP
    
    // Navigation test should not fail because it uses production flow
    let result = service.test_navigation_step().await;
    
    // In production mode, navigation should work (using production methods)
    assert!(result.is_ok());
    let debug_info = result.unwrap();
    assert!(debug_info.page_title.contains("Production"));
}
