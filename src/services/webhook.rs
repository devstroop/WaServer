//! Webhook Service
//!
//! Fires HTTP callbacks on message events (receive-only by default).

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::config::WebhookConfig;

/// Webhook event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    MessageReceived,
}

impl std::fmt::Display for WebhookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookEvent::MessageReceived => write!(f, "message.received"),
        }
    }
}

impl WebhookEvent {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "message.received" | "message_received" => Some(WebhookEvent::MessageReceived),
            "*" => Some(WebhookEvent::MessageReceived), // Wildcard matches all
            _ => None,
        }
    }
}

/// Payload sent to webhook endpoints
#[derive(Debug, Clone, Serialize)]
pub struct WebhookPayload {
    pub event: String,
    pub timestamp: i64,
    pub data: WebhookMessageData,
}

/// Message data included in webhook payload
#[derive(Debug, Clone, Serialize)]
pub struct WebhookMessageData {
    pub id: String,
    pub from: String,
    pub to: String,
    pub text: Option<String>,
    pub message_type: String,
    pub timestamp: Option<String>,
    pub contact_name: Option<String>,
    pub is_group: bool,
}

/// Webhook endpoint configuration (parsed from config)
#[derive(Debug, Clone)]
pub struct WebhookEndpoint {
    pub url: String,
    pub events: Vec<WebhookEvent>,
    pub secret: Option<String>,
    pub headers: std::collections::HashMap<String, String>,
}

/// Webhook service for firing HTTP callbacks
pub struct WebhookService {
    config: WebhookConfig,
    client: Client,
    endpoints: Vec<WebhookEndpoint>,
    sender: Option<mpsc::Sender<(WebhookEvent, WebhookPayload)>>,
}

impl WebhookService {
    /// Create new webhook service from config
    pub fn new(config: WebhookConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .unwrap_or_default();

        // Parse endpoints from config
        let endpoints = config
            .endpoints
            .iter()
            .map(|ep| {
                let events = ep
                    .events
                    .iter()
                    .filter_map(|e| WebhookEvent::from_str(e))
                    .collect();
                WebhookEndpoint {
                    url: ep.url.clone(),
                    events,
                    secret: ep.secret.clone(),
                    headers: ep.headers.clone().unwrap_or_default(),
                }
            })
            .collect();

        Self {
            config,
            client,
            endpoints,
            sender: None,
        }
    }

    /// Start background worker for async webhook delivery
    pub fn start_worker(mut self) -> Arc<Self> {
        if !self.config.enabled {
            info!("Webhooks disabled");
            return Arc::new(self);
        }

        let (tx, mut rx) = mpsc::channel::<(WebhookEvent, WebhookPayload)>(100);
        self.sender = Some(tx);

        let service = Arc::new(self);
        let service_clone = service.clone();

        // Spawn background worker
        tokio::spawn(async move {
            info!("Webhook worker started");
            while let Some((event, payload)) = rx.recv().await {
                service_clone.deliver_webhook(&event, &payload).await;
            }
            info!("Webhook worker stopped");
        });

        service
    }

    /// Queue a webhook for delivery (non-blocking)
    pub async fn fire(&self, event: WebhookEvent, data: WebhookMessageData) {
        if !self.config.enabled {
            return;
        }

        let payload = WebhookPayload {
            event: event.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            data,
        };

        if let Some(sender) = &self.sender {
            if let Err(e) = sender.send((event, payload)).await {
                error!("Failed to queue webhook: {}", e);
            }
        } else {
            // Fallback: deliver synchronously
            self.deliver_webhook(&event, &payload).await;
        }
    }

    /// Deliver webhook to all matching endpoints
    async fn deliver_webhook(&self, event: &WebhookEvent, payload: &WebhookPayload) {
        for endpoint in &self.endpoints {
            // Check if endpoint subscribes to this event
            if !endpoint.events.contains(event) && !endpoint.events.is_empty() {
                continue;
            }

            debug!("Delivering webhook to {}", endpoint.url);

            // Serialize payload
            let body = match serde_json::to_string(payload) {
                Ok(b) => b,
                Err(e) => {
                    error!("Failed to serialize webhook payload: {}", e);
                    continue;
                }
            };

            // Build request
            let mut request = self
                .client
                .post(&endpoint.url)
                .header("Content-Type", "application/json")
                .header("User-Agent", "WhatsApp-Engine/0.2.0");

            // Add HMAC signature if secret is configured
            if let Some(secret) = &endpoint.secret {
                let signature = Self::compute_signature(secret, &body);
                request = request.header("X-Webhook-Signature", format!("sha256={}", signature));
            }

            // Add custom headers
            for (key, value) in &endpoint.headers {
                request = request.header(key, value);
            }

            // Send with retries
            let mut attempts = 0;
            let max_retries = self.config.retry_count;

            loop {
                attempts += 1;
                match request.try_clone().unwrap().body(body.clone()).send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            debug!(
                                "Webhook delivered to {} (status: {})",
                                endpoint.url,
                                response.status()
                            );
                            break;
                        } else {
                            warn!(
                                "Webhook to {} returned status {} (attempt {}/{})",
                                endpoint.url,
                                response.status(),
                                attempts,
                                max_retries + 1
                            );
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Webhook to {} failed: {} (attempt {}/{})",
                            endpoint.url,
                            e,
                            attempts,
                            max_retries + 1
                        );
                    }
                }

                if attempts > max_retries {
                    error!(
                        "Webhook to {} failed after {} attempts",
                        endpoint.url,
                        attempts
                    );
                    break;
                }

                // Wait before retry
                tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
            }
        }
    }

    /// Compute HMAC-SHA256 signature
    fn compute_signature(secret: &str, body: &str) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(body.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// Check if webhooks are enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get configured endpoints count
    pub fn endpoints_count(&self) -> usize {
        self.endpoints.len()
    }
}

impl Default for WebhookService {
    fn default() -> Self {
        Self::new(WebhookConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_event_from_str() {
        assert_eq!(
            WebhookEvent::from_str("message.received"),
            Some(WebhookEvent::MessageReceived)
        );
        assert_eq!(
            WebhookEvent::from_str("*"),
            Some(WebhookEvent::MessageReceived)
        );
        assert_eq!(WebhookEvent::from_str("unknown"), None);
    }

    #[test]
    fn test_hmac_signature() {
        let signature = WebhookService::compute_signature("secret", "test body");
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // SHA256 hex = 64 chars
    }
}
