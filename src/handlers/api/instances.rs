//! Instances Management API
//!
//! REST API endpoints for managing WhatsApp instances (create, list, get, delete).
//! Instance IDs are UUIDs. Lookup by phone number is also supported.
//! All endpoints require secret key authentication.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    models::instance::{
        CreateInstanceRequest, DeleteInstanceQuery, DeleteInstanceResponse, InstanceActionResponse,
        ListInstancesQuery, UpdateInstanceConfigRequest,
    },
    services::InstanceManager,
};

// === API Handlers ===

/// List all instances
#[utoipa::path(
    get,
    path = "/api/v1/instances",
    tag = "Instances",
    responses(
        (status = 200, description = "List of instances", body = InstanceListResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_instances(
    State(manager): State<Arc<InstanceManager>>,
    Query(_query): Query<ListInstancesQuery>,
) -> impl IntoResponse {
    let accounts = manager.list_instances().await;
    Json(accounts)
}

/// Create a new instance
#[utoipa::path(
    post,
    path = "/api/v1/instances",
    tag = "Instances",
    request_body = CreateInstanceRequest,
    responses(
        (status = 201, description = "Account created", body = CreateInstanceResponse),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Instance already exists"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_instance(
    State(manager): State<Arc<InstanceManager>>,
    Json(request): Json<CreateInstanceRequest>,
) -> impl IntoResponse {
    match manager.create_instance(request).await {
        Ok(response) => (StatusCode::CREATED, Json(json!(response))).into_response(),
        Err(e) => {
            let error_msg = e.to_string();
            let status = if error_msg.contains("already exists") {
                StatusCode::CONFLICT
            } else if error_msg.contains("Invalid") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(json!({
                    "error": "create_failed",
                    "message": error_msg
                })),
            )
                .into_response()
        }
    }
}

/// Get instance info
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Account info", body = InstanceInfo),
        (status = 404, description = "Instance not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_instance(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    match manager.get_instance(&instance_id).await {
        Some(account) => {
            let info = account.info().await;
            Json(json!(info)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "not_found",
                "message": format!("Instance '{}' not found", instance_id)
            })),
        )
            .into_response(),
    }
}

/// Delete an instance
#[utoipa::path(
    delete,
    path = "/api/v1/instances/{instance_id}",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)"),
        ("delete_data" = bool, Query, description = "Delete all instance data")
    ),
    responses(
        (status = 200, description = "Account deleted", body = DeleteInstanceResponse),
        (status = 404, description = "Instance not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_instance(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
    Query(query): Query<DeleteInstanceQuery>,
) -> impl IntoResponse {
    match manager
        .delete_instance(&instance_id, query.delete_data)
        .await
    {
        Ok(instance_id) => Json(json!(DeleteInstanceResponse {
            message: if query.delete_data {
                "Instance and all data deleted".to_string()
            } else {
                "Instance deleted, data preserved".to_string()
            },
            instance_id,
            data_deleted: query.delete_data,
        }))
        .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "delete_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Warmup an instance's browser (pre-warm for faster response)
///
/// Instances auto-warm on demand, but this endpoint lets you pre-warm
/// an instance before sending requests to it.
#[utoipa::path(
    post,
    path = "/api/v1/instances/{instance_id}/warmup",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Account warmed up", body = InstanceActionResponse),
        (status = 404, description = "Instance not found"),
        (status = 409, description = "Instance already warming up"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn warmup_instance(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_instance(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response()
        }
    };

    let instance_id = account.id;

    match account.warmup().await {
        Ok(()) => Json(json!(InstanceActionResponse {
            message: "Instance warmed up successfully".to_string(),
            instance_id,
        }))
        .into_response(),
        Err(e) => {
            let error_msg = e.to_string();
            let status = if error_msg.contains("already warming up") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (
                status,
                Json(json!({
                    "error": "warmup_failed",
                    "message": error_msg
                })),
            )
                .into_response()
        }
    }
}

/// Get live screenshot of instance's browser
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}/screenshot",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "PNG screenshot", content_type = "image/png"),
        (status = 404, description = "Instance not found"),
        (status = 503, description = "Browser not active"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn screenshot(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_instance(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response()
        }
    };

    if let Err(e) = account.ensure_warm().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "warmup_failed",
                "message": format!("Failed to warm up instance: {}", e)
            })),
        )
            .into_response();
    }

    match account.browser_service().screenshot().await {
        Ok(png_data) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "image/png")],
            png_data,
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "screenshot_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Get instance configuration
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}/config",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Instance configuration", body = InstanceConfig),
        (status = 404, description = "Instance not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_instance_config(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_instance(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response()
        }
    };

    let config = account.get_config().await;
    Json(config).into_response()
}

/// Update instance configuration
#[utoipa::path(
    put,
    path = "/api/v1/instances/{instance_id}/config",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)")
    ),
    request_body = UpdateInstanceConfigRequest,
    responses(
        (status = 200, description = "Configuration updated", body = InstanceConfig),
        (status = 404, description = "Instance not found"),
        (status = 400, description = "Invalid configuration"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_instance_config(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
    Json(request): Json<UpdateInstanceConfigRequest>,
) -> impl IntoResponse {
    let account = match manager.get_instance(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response()
        }
    };

    match account.update_config(request).await {
        Ok(config) => Json(json!({
            "message": "Configuration updated successfully",
            "config": config,
            "restart_required": true
        }))
        .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "config_update_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Reset an instance
///
/// Stops the browser and wipes all session data (chrome profile, sessions, media).
/// The instance itself is preserved — only runtime data is cleared.
/// Use this to start fresh without deleting and re-creating the instance.
#[utoipa::path(
    delete,
    path = "/api/v1/instances/{instance_id}/reset",
    tag = "Instances",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID or phone number)")
    ),
    responses(
        (status = 200, description = "Instance reset", body = InstanceActionResponse),
        (status = 404, description = "Instance not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn reset_instance(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let account_ref = match manager.get_instance(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response()
        }
    };

    let id = account_ref.id;

    match account_ref.reset().await {
        Ok(()) => Json(json!(InstanceActionResponse {
            message: "Instance reset — all session data cleared".to_string(),
            instance_id: id,
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "reset_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}
