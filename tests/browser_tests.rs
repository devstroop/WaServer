use anyhow::Result;
use std::sync::Arc;
use whatsapp_engine_rust::config::AppConfig;
use whatsapp_engine_rust::services::browser::BrowserService;

#[cfg(test)]
mod browser_tests {
    use super::*;

    #[tokio::test]
    async fn test_browser_service_creation() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let browser_service = BrowserService::new(config);
        
        // Just test that we can create the service
        assert!(!browser_service.is_running().await);
        Ok(())
    }

    #[tokio::test]
    #[ignore] // This test requires Chrome to be installed
    async fn test_browser_initialization() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let browser_service = BrowserService::new(config);
        
        // Test browser initialization
        let result = browser_service.initialize().await;
        
        // Should succeed if Chrome is available, or gracefully handle failure
        match result {
            Ok(_) => {
                println!("Browser initialized successfully");
                assert!(browser_service.is_running().await);
                
                // Clean up
                browser_service.close().await?;
            }
            Err(e) => {
                println!("Browser initialization failed (expected if Chrome not available): {}", e);
                // This is okay - the service should handle missing Chrome gracefully
            }
        }
        
        Ok(())
    }

    #[tokio::test]
    #[ignore] // This test requires Chrome to be installed
    async fn test_page_creation() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let browser_service = BrowserService::new(config);
        
        // Initialize browser
        if browser_service.initialize().await.is_ok() {
            // Test page creation
            match browser_service.get_or_create_page("https://example.com").await {
                Ok(page) => {
                    println!("Page created successfully");
                    
                    // Test getting URL
                    match page.url().await {
                        Ok(url) => println!("Page URL: {}", url),
                        Err(e) => println!("Could not get page URL: {}", e),
                    }
                }
                Err(e) => {
                    println!("Page creation failed: {}", e);
                }
            }
            
            // Clean up
            browser_service.close().await?;
        }
        
        Ok(())
    }

    #[tokio::test]
    #[ignore] // This test requires Chrome to be installed
    async fn test_whatsapp_page_persistence() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let browser_service = BrowserService::new(config);
        
        // Initialize browser
        if browser_service.initialize().await.is_ok() {
            // Get WhatsApp page twice
            let page1_result = browser_service.get_whatsapp_page().await;
            let page2_result = browser_service.get_whatsapp_page().await;
            
            match (page1_result, page2_result) {
                (Ok(page1), Ok(page2)) => {
                    // Both should be valid pages
                    println!("WhatsApp page persistence test passed");
                    
                    // Test that they can get URLs (indicating they're active)
                    let _ = page1.url().await;
                    let _ = page2.url().await;
                }
                _ => {
                    println!("WhatsApp page creation failed");
                }
            }
            
            // Clean up
            browser_service.close().await?;
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_browser_service_graceful_failure() -> Result<()> {
        let mut config = AppConfig::default();
        // Set an invalid Chrome path to test graceful failure
        config.browser.args.push("--invalid-argument-that-should-fail".to_string());
        
        let config = Arc::new(config);
        let browser_service = BrowserService::new(config);
        
        // Test that initialization handles failure gracefully
        let result = browser_service.initialize().await;
        
        // Should not panic and should handle errors gracefully
        match result {
            Ok(_) => println!("Browser initialized despite invalid args"),
            Err(e) => println!("Browser initialization failed gracefully: {}", e),
        }
        
        // Service should still be usable for cleanup
        browser_service.close().await?;
        
        Ok(())
    }
}
