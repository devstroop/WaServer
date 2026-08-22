// Structured Logging and Request Correlation
//
// Production-ready logging utilities with request correlation and metrics

use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::environment::EnvironmentConfig;

/// Correlation ID for request tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationId(pub String);

impl CorrelationId {
    /// Generate a new correlation ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create correlation ID from existing string
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Request metrics for performance tracking
#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub duration_ms: u64,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

/// Initialize logging based on environment configuration
pub fn init_logging(env_config: &EnvironmentConfig) -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

    // Create env filter from configuration
    let env_filter = EnvFilter::try_new(env_config.get_rust_log_filter())
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let registry = Registry::default().with(env_filter);

    if env_config.is_production() {
        // Production: JSON logging
        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(false)
            .with_span_list(false);

        registry.with(json_layer).init();
    } else {
        // Development: Pretty printing
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_level(true);

        registry.with(fmt_layer).init();
    }

    Ok(())
}

/// Log request metrics with correlation ID
pub fn log_request_metrics(metrics: &RequestMetrics, correlation_id: &CorrelationId) {
    if metrics.status_code >= 400 {
        warn!(
            correlation_id = %correlation_id.0,
            method = %metrics.method,
            path = %metrics.path,
            status_code = metrics.status_code,
            duration_ms = metrics.duration_ms,
            user_agent = ?metrics.user_agent,
            ip_address = ?metrics.ip_address,
            "Request completed with error"
        );
    } else {
        info!(
            correlation_id = %correlation_id.0,
            method = %metrics.method,
            path = %metrics.path,
            status_code = metrics.status_code,
            duration_ms = metrics.duration_ms,
            user_agent = ?metrics.user_agent,
            ip_address = ?metrics.ip_address,
            "Request completed"
        );
    }
}
