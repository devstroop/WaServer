//! WhatsApp Account Operations API
//!
//! REST API endpoints for WhatsApp account operations (status, QR, logout, profile, privacy).
//! Account ID is a UUID, phone number is the E.164 identifier.
//! These endpoints use path parameter {account_id}.

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
    models::account::{ProfileInfo, UpdateProfileRequest, WhatsAppStatusResponse},
    services::AccountManager,
};

// === API Handlers ===

/// Get WhatsApp authentication status
///
/// Returns the authentication status and bound phone number for the account.
#[utoipa::path(
    get,
    path = "/api/v1/accounts/{account_id}/status",
    tag = "Account",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID)")
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
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_account(&account_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Account '{}' not found", account_id)
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
/// Returns a QR code as a PNG image for linking WhatsApp on mobile device.
#[utoipa::path(
    get,
    path = "/api/v1/accounts/{account_id}/link/qr",
    tag = "Account",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID)")
    ),
    responses(
        (status = 200, description = "QR code PNG image", content_type = "image/png"),
        (status = 404, description = "Account not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_qr_code(
    State(manager): State<Arc<AccountManager>>,
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_account(&account_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Account '{}' not found", account_id)
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
                "message": "Account browser is not running. Start it first via POST /api/v1/accounts/{account_id}/start"
            })),
        )
            .into_response();
    }

    // Check if already authorized
    match account.auth_service().is_authorized().await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "already_authorized",
                    "message": "Account is already authorized. Use /api/v1/accounts/{account_id}/unlink to disconnect first."
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

    match account.auth_service().get_auth_qr_code().await {
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
/// Initiates phone number linking flow using the account's stored phone number.
#[utoipa::path(
    post,
    path = "/api/v1/accounts/{account_id}/link/phone",
    tag = "Account",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID)")
    ),
    responses(
        (status = 200, description = "Phone linking initiated"),
        (status = 404, description = "Account not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn link_phone(
    State(manager): State<Arc<AccountManager>>,
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_account(&account_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Account '{}' not found", account_id)
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

    // Use the phone number stored on the account (set during creation)
    let phone_number = match account.phone_number() {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "no_phone_number",
                    "message": "Account has no phone number configured"
                })),
            )
                .into_response();
        }
    };

    // Check if already authorized
    match account.auth_service().is_authorized().await {
        Ok(true) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "already_authorized",
                    "message": "Account is already authorized. Use /api/v1/accounts/{account_id}/unlink to disconnect first."
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

    match account
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
/// Disconnects the WhatsApp Web session for this account.
#[utoipa::path(
    delete,
    path = "/api/v1/accounts/{account_id}/unlink",
    tag = "Account",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID)")
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
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_account(&account_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Account '{}' not found", account_id)
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
    path = "/api/v1/accounts/{account_id}/profile",
    tag = "Account",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID)")
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
    Path(account_id): Path<String>,
) -> impl IntoResponse {
    let account = match manager.get_account(&account_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Account '{}' not found", account_id)
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
    path = "/api/v1/accounts/{account_id}/profile",
    tag = "Account",
    params(
        ("account_id" = String, Path, description = "Account ID (UUID)")
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
    Path(account_id): Path<String>,
    Json(request): Json<UpdateProfileRequest>,
) -> impl IntoResponse {
    let account = match manager.get_account(&account_id).await {
        Some(acc) => acc,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Account '{}' not found", account_id)
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
