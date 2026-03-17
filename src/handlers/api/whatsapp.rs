//! WhatsApp Instance Operations API
//!
//! REST API endpoints for WhatsApp instance operations (status, QR, linking, unlinking).
//! Instance ID is a UUID, phone number is the E.164 identifier.
//! These endpoints use path parameter {instance_id}.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use base64::Engine;
use serde_json::json;

use crate::{
    models::instance::WhatsAppStatusResponse,
    services::InstanceManager,
};

// === API Handlers ===

/// Get WhatsApp authentication status
///
/// Returns the authentication status and bound phone number for the instance.
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}/status",
    tag = "WhatsApp",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)")
    ),
    responses(
        (status = 200, description = "Instance status", body = WhatsAppStatusResponse),
        (status = 404, description = "Instance not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_instance_status(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let instance = match manager.get_instance(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "instance_not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response();
        }
    };
    let info = instance.info().await;

    // Convert InstanceStatus enum to string
    let status_str = match &info.status {
        crate::models::instance::InstanceStatus::Sleeping => "sleeping",
        crate::models::instance::InstanceStatus::WarmingUp => "warming_up",
        crate::models::instance::InstanceStatus::Active => "active",
        crate::models::instance::InstanceStatus::Error(_) => "error",
    };

    Json(json!(WhatsAppStatusResponse {
        instance_id: info.id,
        phone_number: info.phone_number,
        status: status_str.to_string(),
        authorized: info.authorized,
    }))
    .into_response()
}

/// Get QR code for WhatsApp Web linking
///
/// Returns a QR code as a PNG image for linking WhatsApp on mobile device.
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}/link/qr",
    tag = "WhatsApp",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)")
    ),
    responses(
        (status = 200, description = "QR code PNG image", content_type = "image/png"),
        (status = 404, description = "Instance not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_qr_code(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let instance = match manager.get_instance(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "instance_not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response();
        }
    };

    // Ensure browser is warm (auto-warms if sleeping)
    if let Err(e) = instance.ensure_warm().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "warmup_failed",
                "message": format!("Failed to warm up instance: {}", e)
            })),
        )
            .into_response();
    }

    // Check if already authorized
    match instance.auth_service().is_authorized().await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "already_authorized",
                    "message": "Instance is already authorized. Use /api/v1/instances/{instance_id}/unlink to disconnect first."
                })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "auth_check_failed",
                    "message": e.to_string()
                })),
            )
                .into_response();
        }
        Ok(false) => {} // Continue to QR code generation
    }

    match instance.auth_service().get_auth_qr_code().await {
        Ok(qr_base64) => match base64::engine::general_purpose::STANDARD.decode(&qr_base64) {
            Ok(png_bytes) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/png")],
                png_bytes,
            )
                .into_response(),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "qr_decode_failed",
                    "message": "Failed to decode QR code image data"
                })),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "qr_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Link via phone number
///
/// Initiates phone number linking flow using the instance's stored phone number.
#[utoipa::path(
    post,
    path = "/api/v1/instances/{instance_id}/link/phone",
    tag = "WhatsApp",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)")
    ),
    responses(
        (status = 200, description = "Phone linking initiated"),
        (status = 404, description = "Instance not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn link_phone(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let instance = match manager.get_instance(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "instance_not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response();
        }
    };

    // Ensure browser is warm (auto-warms if sleeping)
    if let Err(e) = instance.ensure_warm().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "warmup_failed",
                "message": format!("Failed to warm up instance: {}", e)
            })),
        )
            .into_response();
    }

    // Use the phone number stored on the instance (set during creation)
    let phone_number = match instance.phone_number() {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "no_phone_number",
                    "message": "Instance has no phone number configured"
                })),
            )
                .into_response();
        }
    };

    // Check if already authorized
    match instance.auth_service().is_authorized().await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "already_authorized",
                    "message": "Instance is already authorized. Use /api/v1/instances/{instance_id}/unlink to disconnect first."
                })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "auth_check_failed",
                    "message": e.to_string()
                })),
            )
                .into_response();
        }
        Ok(false) => {} // Continue to phone linking
    }

    match instance
        .auth_service()
        .login_with_phone_number(&phone_number)
        .await
    {
        Ok(linking_code) => {
            let success = linking_code.is_some();
            Json(json!({
                "success": success,
                "phone_number": phone_number,
                "linking_code": linking_code,
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "link_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Unlink WhatsApp Web
///
/// Disconnects the WhatsApp Web session for this instance.
#[utoipa::path(
    delete,
    path = "/api/v1/instances/{instance_id}/unlink",
    tag = "WhatsApp",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)")
    ),
    responses(
        (status = 200, description = "Unlinked"),
        (status = 404, description = "Instance not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn unlink(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let instance = match manager.get_instance(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "instance_not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response();
        }
    };

    match instance.auth_service().logout().await {
        Ok(_) => {
            instance.invalidate_auth_cache().await;
            Json(json!({
                "success": true,
                "message": "WhatsApp Web session unlinked"
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "unlink_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}
