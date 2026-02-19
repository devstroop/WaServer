//! WhatsApp Account Operations API
//!
//! REST API endpoints for WhatsApp account operations (status, QR, logout, profile, privacy).
//! Account ID is the phone number in E.164 format.
//! These endpoints REQUIRE X-Account-Id header.

use std::sync::Arc;

use axum::{
    extract::Request,
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde_json::json;

use crate::{
    middleware::CurrentAccount,
    models::account::{
        PrivacySettings, PhoneLinkRequest,
        ProfileInfo, WhatsAppStatusResponse,
        UpdateProfileNameRequest, UpdateProfileAboutRequest,
        UpdatePrivacyLastSeenRequest, UpdatePrivacyOnlineRequest,
        UpdatePrivacyProfilePhotoRequest, UpdatePrivacyAboutRequest,
        UpdatePrivacyReadReceiptsRequest, UpdatePrivacyGroupsRequest,
    },
    services::WhatsAppAccount,
};

// === Helper to extract account ===

#[allow(dead_code)]
fn get_account(request: &Request) -> Option<Arc<WhatsAppAccount>> {
    request
        .extensions()
        .get::<CurrentAccount>()
        .map(|ca| ca.0.clone())
}

// === API Handlers ===

/// Get WhatsApp account status
///
/// Returns the authentication status and bound phone number for the account.
#[utoipa::path(
    get,
    path = "/api/v1/account/status",
    tag = "WhatsAppAccount",
    responses(
        (status = 200, description = "Account status", body = WhatsAppStatusResponse),
        (status = 400, description = "Missing X-Account-Id header"),
        (status = 404, description = "Account not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_account_status(
    Extension(current): Extension<CurrentAccount>,
) -> impl IntoResponse {
    let account = current.0;
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
        status: status_str.to_string(),
        authorized: info.authorized,
        last_activity: info.last_activity.map(|dt| dt.to_rfc3339()),
    }))
}

/// Get QR code for WhatsApp Web linking
///
/// Returns a QR code image (base64) for linking WhatsApp on mobile device.
#[utoipa::path(
    get,
    path = "/api/v1/account/qr",
    tag = "WhatsAppAccount",
    responses(
        (status = 200, description = "QR code"),
        (status = 400, description = "Missing X-Account-Id header"),
        (status = 404, description = "Account not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_qr_code(
    Extension(current): Extension<CurrentAccount>,
) -> impl IntoResponse {
    let account = current.0;

    // Check if browser is running
    if !account.browser_service().is_running().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "browser_not_running",
                "message": "Account browser is not running. Start it first via POST /api/v1/accounts/{id}/start"
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
    path = "/api/v1/account/phone",
    tag = "WhatsAppAccount",
    request_body = PhoneLinkRequest,
    responses(
        (status = 200, description = "Phone linking initiated"),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Account bound to different phone"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn link_phone(
    Extension(current): Extension<CurrentAccount>,
    Json(request): Json<PhoneLinkRequest>,
) -> impl IntoResponse {
    let account = current.0;

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

/// Logout from WhatsApp Web
///
/// Logs out from WhatsApp Web session.
#[utoipa::path(
    post,
    path = "/api/v1/account/logout",
    tag = "WhatsAppAccount",
    responses(
        (status = 200, description = "Logged out"),
        (status = 400, description = "Missing X-Account-Id header"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn logout(
    Extension(current): Extension<CurrentAccount>,
) -> impl IntoResponse {
    let account = current.0;

    match account.auth_service().logout().await {
        Ok(_) => {
            account.invalidate_auth_cache().await;
            Json(json!({
                "success": true,
                "message": "Logged out from WhatsApp Web"
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "logout_failed",
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
    path = "/api/v1/account/profile",
    tag = "WhatsAppAccount",
    responses(
        (status = 200, description = "Profile info", body = ProfileInfo),
        (status = 400, description = "Missing X-Account-Id header"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_profile(
    Extension(current): Extension<CurrentAccount>,
) -> impl IntoResponse {
    let account = current.0;

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

/// Update profile name
#[utoipa::path(
    put,
    path = "/api/v1/account/profile/name",
    tag = "WhatsAppAccount",
    request_body = UpdateProfileNameRequest,
    responses(
        (status = 200, description = "Name updated"),
        (status = 400, description = "Invalid request"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_profile_name(
    Extension(current): Extension<CurrentAccount>,
    Json(request): Json<UpdateProfileNameRequest>,
) -> impl IntoResponse {
    let account = current.0;

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

    // TODO: Implement actual name update via WhatsApp Web automation
    Json(json!({
        "success": true,
        "message": "Profile name update not yet implemented",
        "name": request.name
    }))
    .into_response()
}

/// Update profile about
#[utoipa::path(
    put,
    path = "/api/v1/account/profile/about",
    tag = "WhatsAppAccount",
    request_body = UpdateProfileAboutRequest,
    responses(
        (status = 200, description = "About updated"),
        (status = 400, description = "Invalid request"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_profile_about(
    Extension(current): Extension<CurrentAccount>,
    Json(request): Json<UpdateProfileAboutRequest>,
) -> impl IntoResponse {
    let _account = current.0;

    // TODO: Implement actual about update via WhatsApp Web automation
    Json(json!({
        "success": true,
        "message": "Profile about update not yet implemented",
        "about": request.about
    }))
    .into_response()
}

/// Update profile picture
#[utoipa::path(
    put,
    path = "/api/v1/account/profile/picture",
    tag = "WhatsAppAccount",
    responses(
        (status = 200, description = "Picture updated"),
        (status = 400, description = "Invalid request"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_profile_picture(
    Extension(current): Extension<CurrentAccount>,
    // TODO: Accept multipart form data for image upload
) -> impl IntoResponse {
    let _account = current.0;

    // TODO: Implement actual picture update via WhatsApp Web automation
    Json(json!({
        "success": true,
        "message": "Profile picture update not yet implemented"
    }))
    .into_response()
}

// === Privacy Settings ===

/// Get privacy settings
#[utoipa::path(
    get,
    path = "/api/v1/account/privacy",
    tag = "WhatsAppAccount",
    responses(
        (status = 200, description = "Privacy settings", body = PrivacySettings),
        (status = 400, description = "Missing X-Account-Id header"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_privacy(
    Extension(current): Extension<CurrentAccount>,
) -> impl IntoResponse {
    let _account = current.0;

    // TODO: Implement actual privacy settings fetching from WhatsApp Web
    Json(json!(PrivacySettings::default())).into_response()
}

/// Update last seen privacy
#[utoipa::path(
    put,
    path = "/api/v1/account/privacy/last-seen",
    tag = "WhatsAppAccount",
    request_body = UpdatePrivacyLastSeenRequest,
    responses(
        (status = 200, description = "Last seen privacy updated"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_privacy_last_seen(
    Extension(current): Extension<CurrentAccount>,
    Json(request): Json<UpdatePrivacyLastSeenRequest>,
) -> impl IntoResponse {
    let _account = current.0;

    // TODO: Implement
    Json(json!({
        "success": true,
        "message": "Last seen privacy update not yet implemented",
        "visibility": request.visibility
    }))
    .into_response()
}

/// Update online privacy
#[utoipa::path(
    put,
    path = "/api/v1/account/privacy/online",
    tag = "WhatsAppAccount",
    request_body = UpdatePrivacyOnlineRequest,
    responses(
        (status = 200, description = "Online privacy updated"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_privacy_online(
    Extension(current): Extension<CurrentAccount>,
    Json(request): Json<UpdatePrivacyOnlineRequest>,
) -> impl IntoResponse {
    let _account = current.0;

    Json(json!({
        "success": true,
        "message": "Online privacy update not yet implemented",
        "visibility": request.visibility
    }))
    .into_response()
}

/// Update profile photo privacy
#[utoipa::path(
    put,
    path = "/api/v1/account/privacy/profile-photo",
    tag = "WhatsAppAccount",
    request_body = UpdatePrivacyProfilePhotoRequest,
    responses(
        (status = 200, description = "Profile photo privacy updated"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_privacy_profile_photo(
    Extension(current): Extension<CurrentAccount>,
    Json(request): Json<UpdatePrivacyProfilePhotoRequest>,
) -> impl IntoResponse {
    let _account = current.0;

    Json(json!({
        "success": true,
        "message": "Profile photo privacy update not yet implemented",
        "visibility": request.visibility
    }))
    .into_response()
}

/// Update about privacy
#[utoipa::path(
    put,
    path = "/api/v1/account/privacy/about",
    tag = "WhatsAppAccount",
    request_body = UpdatePrivacyAboutRequest,
    responses(
        (status = 200, description = "About privacy updated"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_privacy_about(
    Extension(current): Extension<CurrentAccount>,
    Json(request): Json<UpdatePrivacyAboutRequest>,
) -> impl IntoResponse {
    let _account = current.0;

    Json(json!({
        "success": true,
        "message": "About privacy update not yet implemented",
        "visibility": request.visibility
    }))
    .into_response()
}

/// Update read receipts privacy
#[utoipa::path(
    put,
    path = "/api/v1/account/privacy/read-receipts",
    tag = "WhatsAppAccount",
    request_body = UpdatePrivacyReadReceiptsRequest,
    responses(
        (status = 200, description = "Read receipts updated"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_privacy_read_receipts(
    Extension(current): Extension<CurrentAccount>,
    Json(request): Json<UpdatePrivacyReadReceiptsRequest>,
) -> impl IntoResponse {
    let _account = current.0;

    Json(json!({
        "success": true,
        "message": "Read receipts update not yet implemented",
        "enabled": request.enabled
    }))
    .into_response()
}

/// Update groups privacy
#[utoipa::path(
    put,
    path = "/api/v1/account/privacy/groups",
    tag = "WhatsAppAccount",
    request_body = UpdatePrivacyGroupsRequest,
    responses(
        (status = 200, description = "Groups privacy updated"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_privacy_groups(
    Extension(current): Extension<CurrentAccount>,
    Json(request): Json<UpdatePrivacyGroupsRequest>,
) -> impl IntoResponse {
    let _account = current.0;

    Json(json!({
        "success": true,
        "message": "Groups privacy update not yet implemented",
        "permission": request.permission
    }))
    .into_response()
}
