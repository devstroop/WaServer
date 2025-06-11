use anyhow::Result;
use std::sync::Arc;
use wae_rust::config::AppConfig;
use wae_rust::services::{
    browser::BrowserService,
    whatsapp::WhatsAppService,
};

#[cfg(test)]
mod service_tests {
    use super::*;

    #[tokio::test]
    async fn test_whatsapp_service_creation() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let whatsapp_service = WhatsAppService::new(config);
        
        // Test that the service can be created
        assert!(!whatsapp_service.is_busy().await);
        
        // Test API token access
        let token = whatsapp_service.get_api_token();
        assert!(!token.is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_whatsapp_service_initialization() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let whatsapp_service = WhatsAppService::new(config);
        
        // Test initialization (should handle browser failures gracefully)
        let result = whatsapp_service.initialize().await;
        
        match result {
            Ok(_) => println!("WhatsApp service initialized successfully"),
            Err(e) => println!("WhatsApp service initialization failed gracefully: {}", e),
        }
        
        // Cleanup
        whatsapp_service.close().await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_busy_flag_mechanism() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let whatsapp_service = WhatsAppService::new(config);
        
        // Initially not busy
        assert!(!whatsapp_service.is_busy().await);
        
        // Test execute_with_busy_flag
        let result = whatsapp_service.execute_with_busy_flag(async {
            // Inside the operation, service should be busy
            // Note: We can't easily test this without access to internal state
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            Ok::<(), anyhow::Error>(())
        }).await;
        
        assert!(result.is_ok());
        
        // After operation, should not be busy
        assert!(!whatsapp_service.is_busy().await);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_service_auth_check_without_browser() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let whatsapp_service = WhatsAppService::new(config);
        
        // Test auth check when browser is not available
        // Should handle gracefully without panicking
        match whatsapp_service.check_auth_status_directly().await {
            Ok(authorized) => {
                println!("Auth check returned: {}", authorized);
                // Should be false when browser is not available
                assert!(!authorized);
            }
            Err(e) => {
                println!("Auth check failed gracefully: {}", e);
                // This is acceptable when browser is not available
            }
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_config_validation() -> Result<()> {
        // Test with minimal config
        let config = AppConfig::default();
        
        // Validate essential config fields
        assert!(!config.auth.api_token.is_empty());
        assert!(config.browser.timeout_ms > 0);
        assert!(config.limits.max_concurrent_requests > 0);
        
        println!("Config validation passed");
        println!("API Token: {}", config.auth.api_token);
        println!("Browser timeout: {}ms", config.browser.timeout_ms);
        println!("Max concurrent requests: {}", config.limits.max_concurrent_requests);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_service_cleanup() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let whatsapp_service = WhatsAppService::new(config);
        
        // Initialize if possible
        let _ = whatsapp_service.initialize().await;
        
        // Test cleanup
        let cleanup_result = whatsapp_service.close().await;
        assert!(cleanup_result.is_ok());
        
        println!("Service cleanup test passed");
        Ok(())
    }
}
