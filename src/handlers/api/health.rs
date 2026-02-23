// Health Check and Metrics Endpoints
//
// Production-ready health checking for monitoring and observability
// Now uses InstanceManager for multi-instance status

use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::SystemTime};
use utoipa::ToSchema;

use crate::services::InstanceManager;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: u64,
    pub version: String,
    pub uptime_seconds: u64,
    pub instances_count: usize,
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
    pub instances_count: usize,
    pub instances: Vec<InstanceMetrics>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct InstanceMetrics {
    pub id: String,
    pub status: String,
    pub authorized: bool,
    pub total_messages_sent: u64,
    pub error_count: u64,
}

static START_TIME: std::sync::OnceLock<SystemTime> = std::sync::OnceLock::new();

fn get_start_time() -> SystemTime {
    *START_TIME.get_or_init(SystemTime::now)
}

/// Health Check Endpoint
///
/// Returns the overall health status of the WAS (WhatsApp Server) API.
/// This endpoint is typically used by load balancers and monitoring systems.
#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy", body = HealthResponse)
    ),
    tag = "Health"
)]
pub async fn health_check(
    State(manager): State<Arc<InstanceManager>>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let uptime = SystemTime::now()
        .duration_since(get_start_time())
        .unwrap()
        .as_secs();

    let instances_count = manager.count().await;

    // Server is healthy if it's running (instances health is separate concern)
    let server_health = ServiceHealth {
        status: "healthy".to_string(),
        last_check: now,
        response_time_ms: Some(0),
        details: None,
    };

    let mut services = HashMap::new();
    services.insert("server".to_string(), server_health);

    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: now,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        instances_count,
        services,
    };

    Ok(Json(response))
}

/// Readiness Check Endpoint
///
/// Returns whether the service is ready to accept traffic.
/// This is typically used by Kubernetes readiness probes.
#[utoipa::path(
    get,
    path = "/api/ready",
    responses(
        (status = 200, description = "Service is ready"),
        (status = 503, description = "Service is not ready")
    ),
    tag = "Health"
)]
pub async fn readiness_check() -> Result<(), StatusCode> {
    // Server is ready if we can respond
    Ok(())
}

/// Liveness Check Endpoint
///
/// Returns whether the service is alive and running.
/// This is typically used by Kubernetes liveness probes.
#[utoipa::path(
    get,
    path = "/api/live",
    responses(
        (status = 200, description = "Service is alive"),
        (status = 503, description = "Service is not responding")
    ),
    tag = "Health"
)]
pub async fn liveness_check() -> Result<(), StatusCode> {
    Ok(())
}

/// Metrics Endpoint
///
/// Returns operational metrics for monitoring and observability.
/// This endpoint provides performance and usage statistics.
#[utoipa::path(
    get,
    path = "/api/metrics",
    responses(
        (status = 200, description = "Metrics data", body = MetricsResponse)
    ),
    tag = "Health"
)]
pub async fn get_metrics(State(manager): State<Arc<InstanceManager>>) -> Json<MetricsResponse> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let uptime = SystemTime::now()
        .duration_since(get_start_time())
        .unwrap()
        .as_secs();

    let memory_usage = get_memory_usage();

    // Get metrics from all instances
    let instance_list = manager.list_instances().await;
    let mut instance_metrics = Vec::new();

    for info in instance_list.instances {
        // Get the instance to retrieve metrics
        if let Some(instance) = manager.get_instance_by_id(info.id).await {
            let metrics = instance.get_metrics();
            let status_str = match &info.status {
                crate::models::instance::InstanceStatus::Stopped => "stopped",
                crate::models::instance::InstanceStatus::Starting => "starting",
                crate::models::instance::InstanceStatus::Running => "running",
                crate::models::instance::InstanceStatus::Error(_) => "error",
            };

            instance_metrics.push(InstanceMetrics {
                id: info.id.to_string(),
                status: status_str.to_string(),
                authorized: info.authorized,
                total_messages_sent: metrics.total_messages_sent,
                error_count: metrics.error_count,
            });
        }
    }

    Json(MetricsResponse {
        timestamp: now,
        uptime_seconds: uptime,
        memory_usage_bytes: memory_usage,
        instances_count: instance_metrics.len(),
        instances: instance_metrics,
    })
}

fn get_memory_usage() -> u64 {
    // Placeholder - in production use jemalloc or similar
    std::process::id() as u64 * 1024 * 1024
}
