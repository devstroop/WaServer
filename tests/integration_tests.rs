use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

const BASE_URL: &str = "http://localhost:3000";

/// Secret key used by the running server.
/// Override with: WAS_SECRET=<key> cargo test --test integration_tests -- --ignored
fn secret() -> String {
    std::env::var("WAS_SECRET")
        .unwrap_or_else(|_| "change-this-secret-key-in-production".to_string())
}

/// Integration tests for WAS v0.3.0 API.
#[cfg(test)]
mod tests {
    use super::*;

    fn auth_header() -> String {
        format!("Bearer {}", secret())
    }

    /// Generate a unique-enough phone number per call using epoch nanos,
    /// so parallel test threads never collide on the DB's unique index.
    fn unique_phone() -> String {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock went backwards")
            .as_nanos();
        // Keep 7 digits after the +1555 prefix; mix nanos to avoid collisions.
        let suffix = (nanos % 10_000_000) as u64;
        format!("+1555{:07}", suffix)
    }

    // ── Health / infrastructure ────────────────────────────────

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_api_health_check() -> Result<()> {
        let client = Client::new();

        let response = client
            .get(format!("{}/api/health", BASE_URL))
            .send()
            .await?;

        assert!(response.status().is_success());
        let body: Value = response.json().await?;
        assert_eq!(body["status"], "healthy");
        println!("Health check successful: {}", body);
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_readiness_and_liveness() -> Result<()> {
        let client = Client::new();

        let ready = client.get(format!("{}/api/ready", BASE_URL)).send().await?;
        assert!(ready.status().is_success());

        let live = client.get(format!("{}/api/live", BASE_URL)).send().await?;
        assert!(live.status().is_success());

        println!("Readiness and liveness probes OK");
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_metrics_endpoint() -> Result<()> {
        let client = Client::new();

        let response = client
            .get(format!("{}/api/metrics", BASE_URL))
            .send()
            .await?;
        assert!(response.status().is_success());
        println!("Metrics endpoint OK");
        Ok(())
    }

    // ── Authentication ─────────────────────────────────────────

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_unauthorized_request() -> Result<()> {
        let client = Client::new();

        let response = client
            .get(format!("{}/api/v1/instances", BASE_URL))
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
            .get(format!("{}/api/v1/instances", BASE_URL))
            .send()
            .await?;

        assert_eq!(response.status(), 401);
        Ok(())
    }

    // ── Instance management ────────────────────────────────────

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_list_instances() -> Result<()> {
        let client = Client::new();

        let response = client
            .get(format!("{}/api/v1/instances", BASE_URL))
            .header("Authorization", auth_header())
            .send()
            .await?;

        assert!(response.status().is_success());
        let body: Value = response.json().await?;
        assert!(body.get("instances").is_some());
        assert!(body.get("total").is_some());
        println!("Instances list: {}", serde_json::to_string_pretty(&body)?);
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_create_and_get_instance() -> Result<()> {
        let client = Client::new();
        let phone = unique_phone();

        // Create
        let response = client
            .post(format!("{}/api/v1/instances", BASE_URL))
            .header("Authorization", auth_header())
            .json(&serde_json::json!({
                "name": "integration-test",
                "phone_number": phone
            }))
            .send()
            .await?;

        assert!(
            response.status().is_success(),
            "create failed: {}",
            response.status()
        );
        let created: Value = response.json().await?;
        let id = created["id"]
            .as_str()
            .expect("created instance must return an id")
            .to_string();

        // Get status
        let response = client
            .get(format!("{}/api/v1/instances/{}/status", BASE_URL, id))
            .header("Authorization", auth_header())
            .send()
            .await?;

        assert!(response.status().is_success());
        let status: Value = response.json().await?;
        assert_eq!(status["authorized"], false);
        assert_eq!(status["instance_id"], id.as_str());
        println!("Instance status: {}", status);

        // Cleanup
        let _ = client
            .delete(format!("{}/api/v1/instances/{}", BASE_URL, id))
            .header("Authorization", auth_header())
            .query(&[("delete_data", "true")])
            .send()
            .await?;

        Ok(())
    }

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_send_message_unauthorized_instance() -> Result<()> {
        let client = Client::new();

        // Create a fresh instance that has no linked WhatsApp session
        let phone = unique_phone();
        let response = client
            .post(format!("{}/api/v1/instances", BASE_URL))
            .header("Authorization", auth_header())
            .json(&serde_json::json!({
                "name": "send-test",
                "phone_number": phone
            }))
            .send()
            .await?;
        let created: Value = response.json().await?;
        let id = created["id"]
            .as_str()
            .expect("created instance must return an id")
            .to_string();

        // Sending without an authorized session must be a client error
        let response = client
            .post(format!("{}/api/v1/instances/{}/send", BASE_URL, id))
            .header("Authorization", auth_header())
            .query(&[("phone", "1234567890")])
            .query(&[("text", "Hello from Rust WAS Test!")])
            .send()
            .await?;

        assert!(
            response.status().is_client_error(),
            "expected client error for unlinked instance, got {}",
            response.status()
        );

        // Cleanup
        let _ = client
            .delete(format!("{}/api/v1/instances/{}", BASE_URL, id))
            .header("Authorization", auth_header())
            .query(&[("delete_data", "true")])
            .send()
            .await?;

        Ok(())
    }

    // ── Misc ───────────────────────────────────────────────────

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_swagger_ui() -> Result<()> {
        let client = Client::new();

        let response = client.get(format!("{}/api-docs/", BASE_URL)).send().await?;

        assert!(response.status().is_success());
        println!("Swagger UI reachable");
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Only run when server is running
    async fn test_concurrent_requests() -> Result<()> {
        let client = Client::new();
        let mut handles = vec![];

        for i in 0..5 {
            let client = client.clone();
            let handle = tokio::spawn(async move {
                let response = client
                    .get(format!("{}/api/v1/instances", BASE_URL))
                    .header("Authorization", format!("Bearer {}", secret()))
                    .send()
                    .await?;

                let status = response.status();
                println!("Request {} completed with status: {}", i, status);
                Ok::<_, anyhow::Error>(status)
            });
            handles.push(handle);
        }

        for handle in handles {
            let status = handle.await??;
            assert!(status.is_success());
        }

        println!("Concurrent requests test completed successfully");
        Ok(())
    }
}
