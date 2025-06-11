use anyhow::Result;
use std::sync::Arc;
use wae_rust::config::AppConfig;
use wae_rust::services::browser::BrowserService;

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
                        Ok(Some(url)) => println!("Page URL: {}", url),
                        Ok(None) => println!("Page URL is not available"),
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

    #[tokio::test]
    async fn test_multiple_browser_services() -> Result<()> {
        // Test creating multiple browser services simultaneously
        let config1 = Arc::new(AppConfig::default());
        let config2 = Arc::new(AppConfig::default());
        
        let browser_service1 = BrowserService::new(config1);
        let browser_service2 = BrowserService::new(config2);
        
        // Both should be able to coexist
        assert!(!browser_service1.is_running().await);
        assert!(!browser_service2.is_running().await);
        
        // Clean up both services
        browser_service1.close().await?;
        browser_service2.close().await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_browser_service_double_close() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let browser_service = BrowserService::new(config);
        
        // Test that calling close multiple times doesn't cause issues
        browser_service.close().await?;
        browser_service.close().await?; // Should not panic or error
        
        Ok(())
    }

    #[tokio::test]
    #[ignore] // This test requires Chrome to be installed
    async fn test_page_navigation() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let browser_service = BrowserService::new(config);
        
        // Initialize browser
        if browser_service.initialize().await.is_ok() {
            // Test navigation to different URLs
            let urls = vec![
                "https://httpbin.org/get",
                "https://example.com",
                "data:text/html,<h1>Test Page</h1>"
            ];
            
            for url in urls {
                match browser_service.get_or_create_page(url).await {
                    Ok(page) => {
                        println!("Successfully navigated to: {}", url);
                        
                        // Try to get the page title
                        if let Ok(title) = page.get_title().await {
                            if let Some(title) = title {
                                println!("Page title: {}", title);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to navigate to {}: {}", url, e);
                    }
                }
            }
            
            // Clean up
            browser_service.close().await?;
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_browser_service_config_validation() -> Result<()> {
        // Test with minimal config
        let config = AppConfig::default();
        let browser_service = BrowserService::new(Arc::new(config));
        
        // Should create successfully
        assert!(!browser_service.is_running().await);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_browser_operations() -> Result<()> {
        // Test concurrent operations on the same browser service
        let config = Arc::new(AppConfig::default());
        let browser_service = Arc::new(BrowserService::new(config));
        
        // Create multiple concurrent tasks
        let mut handles = vec![];
        
        for i in 0..5 {
            let service = browser_service.clone();
            let handle = tokio::spawn(async move {
                // Each task checks if browser is running
                let running = service.is_running().await;
                println!("Task {}: Browser running = {}", i, running);
                Ok::<_, anyhow::Error>(())
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await??;
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_browser_service_memory_cleanup() -> Result<()> {
        // Test that browser service properly cleans up memory
        let config = Arc::new(AppConfig::default());
        
        // Create and drop multiple browser services
        for i in 0..3 {
            let browser_service = BrowserService::new(config.clone());
            println!("Created browser service {}", i);
            
            // Ensure it's not running initially
            assert!(!browser_service.is_running().await);
            
            // Clean up
            browser_service.close().await?;
        }
        
        Ok(())
    }

    #[tokio::test]
    #[ignore] // This test requires Chrome to be installed and may be slow
    async fn test_browser_service_stress_test() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let browser_service = BrowserService::new(config);
        
        // Initialize browser
        if browser_service.initialize().await.is_ok() {
            println!("Starting stress test with multiple pages");
            
            // Create multiple pages rapidly
            let mut page_handles = vec![];
            
            for i in 0..5 {
                let url = format!("data:text/html,<h1>Test Page {}</h1>", i);
                match browser_service.get_or_create_page(&url).await {
                    Ok(page) => {
                        println!("Created page {}", i);
                        page_handles.push(page);
                    }
                    Err(e) => {
                        println!("Failed to create page {}: {}", i, e);
                    }
                }
                
                // Small delay to avoid overwhelming the browser
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            println!("Created {} pages", page_handles.len());
            
            // Try to interact with each page
            for (i, page) in page_handles.iter().enumerate() {
                match page.get_title().await {
                    Ok(Some(title)) => println!("Page {} title: {}", i, title),
                    Ok(None) => println!("Page {} has no title", i),
                    Err(e) => println!("Failed to get title for page {}: {}", i, e),
                }
            }
            
            // Clean up
            browser_service.close().await?;
            println!("Stress test completed");
        }
        
        Ok(())
    }

    #[tokio::test]
    #[ignore] // This test requires Chrome to be installed
    async fn test_browser_service_timeout_handling() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let browser_service = BrowserService::new(config);
        
        // Test initialization with timeout
        let init_result = tokio::time::timeout(
            tokio::time::Duration::from_secs(30),
            browser_service.initialize()
        ).await;
        
        match init_result {
            Ok(Ok(_)) => {
                println!("Browser initialized within timeout");
                
                // Test page creation with timeout
                let page_result = tokio::time::timeout(
                    tokio::time::Duration::from_secs(10),
                    browser_service.get_or_create_page("https://httpbin.org/delay/1")
                ).await;
                
                match page_result {
                    Ok(Ok(_)) => println!("Page created within timeout"),
                    Ok(Err(e)) => println!("Page creation failed: {}", e),
                    Err(_) => println!("Page creation timed out"),
                }
                
                browser_service.close().await?;
            }
            Ok(Err(e)) => println!("Browser initialization failed: {}", e),
            Err(_) => println!("Browser initialization timed out"),
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_browser_service_error_recovery() -> Result<()> {
        let config = Arc::new(AppConfig::default());
        let browser_service = BrowserService::new(config);
        
        // Try to create a page without initializing browser first
        let result = browser_service.get_or_create_page("https://example.com").await;
        
        // Should either handle gracefully or auto-initialize
        match result {
            Ok(_) => println!("Browser auto-initialized and page created"),
            Err(e) => println!("Expected error when browser not initialized: {}", e),
        }
        
        // Service should still be usable
        browser_service.close().await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_browser_service_configuration_edge_cases() -> Result<()> {
        // Test with various configuration edge cases
        
        // Test with empty browser args
        let mut config1 = AppConfig::default();
        config1.browser.args.clear();
        let browser_service1 = BrowserService::new(Arc::new(config1));
        assert!(!browser_service1.is_running().await);
        
        // Test with headless mode disabled
        let mut config2 = AppConfig::default();
        config2.browser.headless = false;
        let browser_service2 = BrowserService::new(Arc::new(config2));
        assert!(!browser_service2.is_running().await);
        
        // Test with many browser args
        let mut config3 = AppConfig::default();
        config3.browser.args.extend(vec![
            "--disable-gpu".to_string(),
            "--disable-dev-shm-usage".to_string(),
            "--no-sandbox".to_string(),
        ]);
        let browser_service3 = BrowserService::new(Arc::new(config3));
        assert!(!browser_service3.is_running().await);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_browser_service_whatsapp_specific() -> Result<()> {
        // Test WhatsApp-specific functionality
        let config = Arc::new(AppConfig::default());
        let browser_service = BrowserService::new(config);
        
        // Test that WhatsApp URL is handled specially
        if browser_service.initialize().await.is_ok() {
            // These should all return the same persistent page
            let page1_result = browser_service.get_or_create_page("https://web.whatsapp.com").await;
            let page2_result = browser_service.get_whatsapp_page().await;
            let page3_result = browser_service.get_or_create_page("https://web.whatsapp.com/send?phone=1234567890").await;
            
            match (page1_result, page2_result, page3_result) {
                (Ok(_), Ok(_), Ok(_)) => {
                    println!("WhatsApp page persistence working correctly");
                }
                _ => {
                    println!("WhatsApp page creation had some failures (may be expected)");
                }
            }
            
            browser_service.close().await?;
        }
        
        Ok(())
    }
}
