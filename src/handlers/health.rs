// Health Check and Metrics Endpoints
//
// Production-ready health checking and metrics for monitoring and observability

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::SystemTime};
use utoipa::ToSchema;

use crate::services::whatsapp::WhatsAppService;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: u64,
    pub version: String,
    pub uptime_seconds: u64,
    pub services: HashMap<String, ServiceHealth>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ServiceHealth {
    pub status: String,
    pub last_check: u64,
    pub response_time_ms: Option<u64>,
    pub details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MetricsResponse {
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub memory_usage_bytes: u64,
    pub whatsapp_connection_status: String,
    pub total_messages_sent: u64,
    pub total_auth_attempts: u64,
    pub error_count: u64,
    pub last_activity: Option<u64>,
}

static START_TIME: std::sync::OnceLock<SystemTime> = std::sync::OnceLock::new();

fn get_start_time() -> SystemTime {
    *START_TIME.get_or_init(|| SystemTime::now())
}

/// Health Check Endpoint
///
/// Returns the overall health status of the WhatsApp Engine API server and its dependencies.
/// This endpoint is typically used by load balancers and monitoring systems.
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy", body = HealthResponse)
    ),
    tag = "Health"
)]
pub async fn health_check(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let uptime = SystemTime::now()
        .duration_since(get_start_time())
        .unwrap()
        .as_secs();

    // Check WhatsApp service health
    let whatsapp_health = check_whatsapp_service_health(&whatsapp_service).await;
    
    // Determine overall status
    let overall_status = if whatsapp_health.status == "healthy" {
        "healthy"
    } else {
        "unhealthy"
    };

    let mut services = HashMap::new();
    services.insert("whatsapp".to_string(), whatsapp_health);

    let response = HealthResponse {
        status: overall_status.to_string(),
        timestamp: now,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        services,
    };

    if overall_status == "healthy" {
        Ok(Json(response))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Readiness Check Endpoint
///
/// Returns whether the service is ready to accept traffic.
/// This is typically used by Kubernetes readiness probes.
#[utoipa::path(
    get,
    path = "/ready",
    responses(
        (status = 200, description = "Service is ready"),
        (status = 503, description = "Service is not ready")
    ),
    tag = "Health"
)]
pub async fn readiness_check(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> Result<(), StatusCode> {
    // Check if the service is initialized and ready
    if whatsapp_service.is_initialized().await {
        Ok(())
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Liveness Check Endpoint
///
/// Returns whether the service is alive and running.
/// This is typically used by Kubernetes liveness probes.
#[utoipa::path(
    get,
    path = "/live",
    responses(
        (status = 200, description = "Service is alive"),
        (status = 503, description = "Service is not responding")
    ),
    tag = "Health"
)]
pub async fn liveness_check() -> Result<(), StatusCode> {
    // Simple liveness check - if we can respond, we're alive
    Ok(())
}

/// Metrics Endpoint
///
/// Returns operational metrics for monitoring and observability.
/// This endpoint provides performance and usage statistics.
#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Metrics data", body = MetricsResponse)
    ),
    tag = "Metrics"
)]
pub async fn get_metrics(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> Json<MetricsResponse> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let uptime = SystemTime::now()
        .duration_since(get_start_time())
        .unwrap()
        .as_secs();

    // Get memory usage (simple approximation)
    let memory_usage = get_memory_usage();

    // Get WhatsApp connection status
    let connection_status = match whatsapp_service.get_auth_status().await {
        Ok(status) => status.status,
        Err(_) => "disconnected".to_string(),
    };

    // Get service metrics (these would be tracked by the service in a real implementation)
    let metrics = whatsapp_service.get_metrics().await;

    Json(MetricsResponse {
        timestamp: now,
        uptime_seconds: uptime,
        memory_usage_bytes: memory_usage,
        whatsapp_connection_status: connection_status,
        total_messages_sent: metrics.total_messages_sent,
        total_auth_attempts: metrics.total_auth_attempts,
        error_count: metrics.error_count,
        last_activity: metrics.last_activity,
    })
}

async fn check_whatsapp_service_health(service: &WhatsAppService) -> ServiceHealth {
    let start = SystemTime::now();
    
    match service.health_check().await {
        Ok(_) => {
            let response_time = SystemTime::now()
                .duration_since(start)
                .unwrap()
                .as_millis() as u64;
            
            ServiceHealth {
                status: "healthy".to_string(),
                last_check: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                response_time_ms: Some(response_time),
                details: None,
            }
        }
        Err(e) => ServiceHealth {
            status: "unhealthy".to_string(),
            last_check: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            response_time_ms: None,
            details: Some(e.to_string()),
        },
    }
}

fn get_memory_usage() -> u64 {
    // This is a simple approximation. In production, you might want to use
    // a more sophisticated memory tracking solution like `jemalloc` or `tikv-jemallocator`
    // For now, we'll return a placeholder value
    std::process::id() as u64 * 1024 * 1024 // Placeholder
}
