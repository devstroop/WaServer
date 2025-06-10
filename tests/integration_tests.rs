use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

const BASE_URL: &str = "http://localhost:3000";
const API_TOKEN: &str = "test-api-token-123456789";

/// Integration tests for WhatsApp Engine API
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_auth_status() -> Result<()> {
        let client = Client::new();
        
        let response = client
            .get(&format!("{}/api/auth/status", BASE_URL))
            .header("Authorization", format!("Bearer {}", API_TOKEN))
            .send()
            .await?;

        assert!(response.status().is_success());
        
        let body: Value = response.json().await?;
        assert!(body.get("authorized").is_some());
        
        println!("Auth status response: {}", serde_json::to_string_pretty(&body)?);
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_qr_code() -> Result<()> {
        let client = Client::new();
        
        let response = client
            .get(&format!("{}/api/auth/qrcode", BASE_URL))
            .header("Authorization", format!("Bearer {}", API_TOKEN))
            .send()
            .await?;

        // Should succeed even if not authorized
        assert!(response.status().is_success() || response.status().is_client_error());
        
        let body: Value = response.json().await?;
        println!("QR code response: {}", serde_json::to_string_pretty(&body)?);
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_phone_auth() -> Result<()> {
        let client = Client::new();
        
        let response = client
            .post(&format!("{}/api/auth/phone/1234567890", BASE_URL))
            .header("Authorization", format!("Bearer {}", API_TOKEN))
            .send()
            .await?;

        // Should return some response (may fail if not ready for phone auth)
        assert!(response.status().is_success() || response.status().is_client_error());
        
        let body: Value = response.json().await?;
        println!("Phone auth response: {}", serde_json::to_string_pretty(&body)?);
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_send_message() -> Result<()> {
        let client = Client::new();
        
        // First check if we're authorized
        let auth_response = client
            .get(&format!("{}/api/auth/status", BASE_URL))
            .header("Authorization", format!("Bearer {}", API_TOKEN))
            .send()
            .await?;

        let auth_body: Value = auth_response.json().await?;
        let is_authorized = auth_body.get("authorized")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        println!("Authorization status: {}", is_authorized);

        // Send a test message
        let response = client
            .post(&format!("{}/api/chat/send", BASE_URL))
            .header("Authorization", format!("Bearer {}", API_TOKEN))
            .query(&[("phone", "1234567890")])
            .query(&[("text", "Hello from Rust WhatsApp Engine Test!")])
            .send()
            .await?;

        // If not authorized, should return 400
        if !is_authorized {
            assert!(response.status().is_client_error());
        }
        
        let body: Value = response.json().await?;
        println!("Send message response: {}", serde_json::to_string_pretty(&body)?);
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_logout() -> Result<()> {
        let client = Client::new();
        
        let response = client
            .post(&format!("{}/api/auth/logout", BASE_URL))
            .header("Authorization", format!("Bearer {}", API_TOKEN))
            .send()
            .await?;

        assert!(response.status().is_success() || response.status().is_client_error());
        
        let body: Value = response.json().await?;
        println!("Logout response: {}", serde_json::to_string_pretty(&body)?);
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_unauthorized_request() -> Result<()> {
        let client = Client::new();
        
        let response = client
            .get(&format!("{}/api/auth/status", BASE_URL))
            .header("Authorization", "Bearer invalid-token")
            .send()
            .await?;

        assert_eq!(response.status(), 401);
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_missing_auth_header() -> Result<()> {
        let client = Client::new();
        
        let response = client
            .get(&format!("{}/api/auth/status", BASE_URL))
            .send()
            .await?;

        assert_eq!(response.status(), 401);
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_api_health_check() -> Result<()> {
        let client = Client::new();
        
        // Test if server is responding
        let response = client
            .get(&format!("{}/docs", BASE_URL))
            .send()
            .await?;

        assert!(response.status().is_success());
        println!("Health check successful - server is running");
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Only run when server is running  
    async fn test_concurrent_requests() -> Result<()> {
        let client = Client::new();
        
        // Send multiple concurrent auth status requests
        let mut handles = vec![];
        
        for i in 0..5 {
            let client = client.clone();
            let handle = tokio::spawn(async move {
                let response = client
                    .get(&format!("{}/api/auth/status", BASE_URL))
                    .header("Authorization", format!("Bearer {}", API_TOKEN))
                    .send()
                    .await?;
                
                let status = response.status();
                println!("Request {} completed with status: {}", i, status);
                Ok::<_, anyhow::Error>(status)
            });
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        for handle in handles {
            let status = handle.await??;
            assert!(status.is_success());
        }
        
        println!("Concurrent requests test completed successfully");
        Ok(())
    }
}

/// Helper function to start the server for testing
#[allow(dead_code)]
async fn start_test_server() -> Result<()> {
    // This would be used to start the server programmatically
    // For now, tests assume server is already running
    Ok(())
}
