//! WhatsApp Account Operations API
//!
//! REST API endpoints for WhatsApp account operations (status, QR, logout, profile, privacy).
//! Account ID is a UUID, phone number is the E.164 identifier.
//! These endpoints use path parameter {instance_id}.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    models::account::{
        PrivacySettings, PhoneLinkRequest,
        ProfileInfo, WhatsAppStatusResponse,
        UpdateProfileRequest, UpdatePrivacyRequest,
    },
    services::AccountManager,
};

// === API Handlers ===

/// Get WhatsApp authentication status
///
/// Returns the authentication status and bound phone number for the account.
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}/status",
    tag = "WhatsApp",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)")
    ),
    responses(
        (status = 200, description = "Account status", body = WhatsAppStatusResponse),
        (status = 404, description = "Account not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_account_status(
    State(manager): State<Arc<AccountManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response();
        }
    };
    let info = account.info().await;

    // Convert AccountStatus enum to string
    let status_str = match &info.status {
        crate::models::account::AccountStatus::Stopped => "stopped",
        crate::models::account::AccountStatus::Starting => "starting",
        crate::models::account::AccountStatus::Running => "running",
        crate::models::account::AccountStatus::Error(_) => "error",
    };

    Json(json!(WhatsAppStatusResponse {
        account_id: info.id,
        phone_number: info.phone_number,
        status: status_str.to_string(),
        authorized: info.authorized,
        last_activity: info.last_activity.map(|dt| dt.to_rfc3339()),
    }))
    .into_response()
}

/// Get QR code for WhatsApp Web linking
///
/// Returns a QR code image (base64) for linking WhatsApp on mobile device.
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}/link/qr",
    tag = "WhatsApp",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)")
    ),
    responses(
        (status = 200, description = "QR code"),
        (status = 404, description = "Account not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_qr_code(
    State(manager): State<Arc<AccountManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response();
        }
    };

    // Check if browser is running
    if !account.browser_service().is_running().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "browser_not_running",
                "message": "Account browser is not running. Start it first via POST /api/v1/instances/{instance_id}/start"
            })),
        )
            .into_response();
    }

    match account.auth_service().get_auth_qr_code().await {
        Ok(qr_code) => Json(json!({
            "qr_code": qr_code,
        }))
        .into_response(),
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
/// Initiates phone number linking flow.
#[utoipa::path(
    post,
    path = "/api/v1/instances/{instance_id}/link/phone",
    tag = "WhatsApp",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)")
    ),
    request_body = PhoneLinkRequest,
    responses(
        (status = 200, description = "Phone linking initiated"),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Account bound to different phone"),
        (status = 404, description = "Account not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn link_phone(
    State(manager): State<Arc<AccountManager>>,
    Path(instance_id): Path<String>,
    Json(request): Json<PhoneLinkRequest>,
) -> impl IntoResponse {
    let account = match manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response();
        }
    };

    // Check if browser is running
    if !account.browser_service().is_running().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "browser_not_running",
                "message": "Account browser is not running"
            })),
        )
            .into_response();
    }

    // Validate that the phone number matches the account
    let account_phone = account.phone_number();
    let normalized_request = crate::models::account::validate_phone_number(&request.phone_number);
    
    if let Ok(req_phone) = normalized_request {
        let normalized_account = crate::models::account::validate_phone_number(account_phone)
            .unwrap_or_else(|_| account_phone.to_string());
        
        if req_phone != normalized_account {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "phone_mismatch",
                    "message": format!(
                        "Phone number {} does not match account {}. Use the correct account or create a new one.",
                        req_phone, normalized_account
                    )
                })),
            )
                .into_response();
        }
    }

    match account
        .auth_service()
        .login_with_phone_number(&request.phone_number)
        .await
    {
        Ok(linking_code) => {
            // If we got a linking code, it was successful
            let success = linking_code.is_some();
            if success {
                if let Err(e) = account.on_whatsapp_authenticated(&request.phone_number).await {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(json!({
                            "error": "binding_failed",
                            "message": e.to_string()
                        })),
                    )
                        .into_response();
                }
            }
            Json(json!({
                "success": success,
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
/// Disconnects the WhatsApp Web session for this account.
#[utoipa::path(
    delete,
    path = "/api/v1/instances/{instance_id}/unlink",
    tag = "WhatsApp",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)")
    ),
    responses(
        (status = 200, description = "Unlinked"),
        (status = 404, description = "Account not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn unlink(
    State(manager): State<Arc<AccountManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response();
        }
    };

    match account.auth_service().logout().await {
        Ok(_) => {
            account.invalidate_auth_cache().await;
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

// === Profile Management ===

/// Get profile info
///
/// Returns the WhatsApp profile information (name, about, picture).
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}/profile",
    tag = "WhatsApp",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)")
    ),
    responses(
        (status = 200, description = "Profile info", body = ProfileInfo),
        (status = 404, description = "Account not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_profile(
    State(manager): State<Arc<AccountManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response();
        }
    };

    if !account.browser_service().is_running().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "browser_not_running",
                "message": "Account browser is not running"
            })),
        )
            .into_response();
    }

    // TODO: Implement actual profile fetching from WhatsApp Web
    // For now, return placeholder
    Json(json!(ProfileInfo {
        name: None,
        about: None,
        picture_url: None,
    }))
    .into_response()
}

/// Update profile
///
/// Updates WhatsApp profile information. All fields are optional - only provided fields are updated.
#[utoipa::path(
    put,
    path = "/api/v1/instances/{instance_id}/profile",
    tag = "WhatsApp",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)")
    ),
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated"),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Account not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_profile(
    State(manager): State<Arc<AccountManager>>,
    Path(instance_id): Path<String>,
    Json(request): Json<UpdateProfileRequest>,
) -> impl IntoResponse {
    let account = match manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response();
        }
    };

    if !account.browser_service().is_running().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "browser_not_running",
                "message": "Account browser is not running"
            })),
        )
            .into_response();
    }

    // TODO: Implement actual profile update via WhatsApp Web automation
    let mut updated = Vec::new();
    if request.name.is_some() {
        updated.push("name");
    }
    if request.about.is_some() {
        updated.push("about");
    }
    if request.picture.is_some() {
        updated.push("picture");
    }

    Json(json!({
        "success": true,
        "message": "Profile update not yet implemented",
        "fields_requested": updated,
        "profile": request
    }))
    .into_response()
}

// === Privacy Settings ===

/// Get privacy settings
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}/privacy",
    tag = "WhatsApp",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)")
    ),
    responses(
        (status = 200, description = "Privacy settings", body = PrivacySettings),
        (status = 404, description = "Account not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_privacy(
    State(manager): State<Arc<AccountManager>>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let _account = match manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response();
        }
    };

    // TODO: Implement actual privacy settings fetching from WhatsApp Web
    Json(json!(PrivacySettings::default())).into_response()
}

/// Update privacy settings
///
/// Updates WhatsApp privacy settings. All fields are optional - only provided fields are updated.
#[utoipa::path(
    put,
    path = "/api/v1/instances/{instance_id}/privacy",
    tag = "WhatsApp",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)")
    ),
    request_body = UpdatePrivacyRequest,
    responses(
        (status = 200, description = "Privacy settings updated"),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Account not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_privacy(
    State(manager): State<Arc<AccountManager>>,
    Path(instance_id): Path<String>,
    Json(request): Json<UpdatePrivacyRequest>,
) -> impl IntoResponse {
    let _account = match manager.get_account(&instance_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response();
        }
    };

    // TODO: Implement actual privacy update via WhatsApp Web automation
    let mut updated = Vec::new();
    if request.last_seen.is_some() {
        updated.push("last_seen");
    }
    if request.online.is_some() {
        updated.push("online");
    }
    if request.profile_photo.is_some() {
        updated.push("profile_photo");
    }
    if request.about.is_some() {
        updated.push("about");
    }
    if request.read_receipts.is_some() {
        updated.push("read_receipts");
    }
    if request.groups.is_some() {
        updated.push("groups");
    }

    Json(json!({
        "success": true,
        "message": "Privacy update not yet implemented",
        "fields_requested": updated,
        "privacy": request
    }))
    .into_response()
}
