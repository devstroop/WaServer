//! WhatsApp Instance Operations API
//!
//! REST API endpoints for WhatsApp instance operations (status, QR, logout, profile, privacy).
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

#[allow(unused_imports)]
use crate::{
    models::chat::{
        ContactInfo, GroupInfo, MarkReadResponse, PresenceInfo, ReactionRequest,
        ReplyMessageRequest, TypingRequest, TypingResponse,
    },
    models::instance::{ProfileInfo, UpdateProfileRequest, WhatsAppStatusResponse},
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

// === Profile Management ===

/// Get profile info
///
/// Returns the WhatsApp profile information (name, about, picture).
// Hidden from Swagger - not yet implemented
// #[utoipa::path(
//     get,
//     path = "/api/v1/instances/{instance_id}/profile",
//     tag = "WhatsApp",
//     params(
//         ("instance_id" = String, Path, description = "Instance ID (UUID)")
//     ),
//     responses(
//         (status = 200, description = "Profile info", body = ProfileInfo),
//         (status = 404, description = "Instance not found"),
//         (status = 503, description = "Browser not running"),
//     ),
//     security(
//         ("bearer_auth" = [])
//     )
// )]
pub async fn get_profile(
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
// Hidden from Swagger - not yet implemented
// #[utoipa::path(
//     put,
//     path = "/api/v1/instances/{instance_id}/profile",
//     tag = "WhatsApp",
//     params(
//         ("instance_id" = String, Path, description = "Instance ID (UUID)")
//     ),
//     request_body = UpdateProfileRequest,
//     responses(
//         (status = 200, description = "Profile updated"),
//         (status = 400, description = "Invalid request"),
//         (status = 404, description = "Instance not found"),
//         (status = 503, description = "Browser not running"),
//     ),
//     security(
//         ("bearer_auth" = [])
//     )
// )]
pub async fn update_profile(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
    Json(request): Json<UpdateProfileRequest>,
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

// === New Chat Features ===

/// Send typing indicator to a chat
///
/// Sends a typing state (composing/paused) to indicate the user is typing.
#[utoipa::path(
    post,
    path = "/api/v1/instances/{instance_id}/messages/{phone}/typing",
    tag = "Chat",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)"),
        ("phone" = String, Path, description = "Phone number (e.g., 919876543210)")
    ),
    request_body = TypingRequest,
    responses(
        (status = 200, description = "Typing indicator sent", body = TypingResponse),
        (status = 404, description = "Instance not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn send_typing(
    State(manager): State<Arc<InstanceManager>>,
    Path((instance_id, phone)): Path<(String, String)>,
    Json(request): Json<TypingRequest>,
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

    let chat_service = instance.chat_service();
    match chat_service.send_typing(&phone, request.state).await {
        Ok(()) => Json(TypingResponse {
            success: true,
            chat_id: phone.clone(),
            state: request.state,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "typing_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Mark messages as read in a chat
///
/// Marks all messages in the specified chat as read.
#[utoipa::path(
    post,
    path = "/api/v1/instances/{instance_id}/messages/{phone}/read",
    tag = "Chat",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)"),
        ("phone" = String, Path, description = "Phone number (e.g., 919876543210)")
    ),
    responses(
        (status = 200, description = "Messages marked as read", body = MarkReadResponse),
        (status = 404, description = "Instance not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn mark_read(
    State(manager): State<Arc<InstanceManager>>,
    Path((instance_id, phone)): Path<(String, String)>,
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

    let chat_service = instance.chat_service();
    match chat_service.mark_read(&phone).await {
        Ok(count) => Json(MarkReadResponse {
            success: true,
            chat_id: phone.clone(),
            messages_read: count,
        })
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "mark_read_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Get presence/online status for a contact
///
/// Returns the online status and last seen time for a contact.
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}/contacts/{contact_id}/presence",
    tag = "Contact",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)"),
        ("contact_id" = String, Path, description = "Contact ID (phone number)")
    ),
    responses(
        (status = 200, description = "Presence information", body = PresenceInfo),
        (status = 404, description = "Instance or contact not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_presence(
    State(manager): State<Arc<InstanceManager>>,
    Path((instance_id, contact_id)): Path<(String, String)>,
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

    let chat_service = instance.chat_service();
    match chat_service.get_presence(&contact_id).await {
        Ok(presence) => Json(presence).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "presence_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Get group information
///
/// Returns detailed information about a WhatsApp group including participants.
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}/groups/{group_id}",
    tag = "Group",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)"),
        ("group_id" = String, Path, description = "Group ID")
    ),
    responses(
        (status = 200, description = "Group information", body = GroupInfo),
        (status = 404, description = "Instance or group not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_group_info(
    State(manager): State<Arc<InstanceManager>>,
    Path((instance_id, group_id)): Path<(String, String)>,
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

    let chat_service = instance.chat_service();
    match chat_service.get_group_info(&group_id).await {
        Ok(info) => Json(info).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "group_info_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Get contact information
///
/// Returns detailed profile information for a contact.
#[utoipa::path(
    get,
    path = "/api/v1/instances/{instance_id}/contacts/{contact_id}",
    tag = "Contact",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)"),
        ("contact_id" = String, Path, description = "Contact ID (phone number)")
    ),
    responses(
        (status = 200, description = "Contact information", body = ContactInfo),
        (status = 404, description = "Instance or contact not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_contact_info(
    State(manager): State<Arc<InstanceManager>>,
    Path((instance_id, contact_id)): Path<(String, String)>,
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

    let chat_service = instance.chat_service();
    match chat_service.get_contact_info(&contact_id).await {
        Ok(info) => Json(info).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "contact_info_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Send a reaction to a message
///
/// Adds an emoji reaction to a specific message.
#[utoipa::path(
    post,
    path = "/api/v1/instances/{instance_id}/messages/{phone}/{message_id}/react",
    tag = "Message",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)"),
        ("phone" = String, Path, description = "Phone number"),
        ("message_id" = String, Path, description = "Message ID to react to")
    ),
    request_body = ReactionRequest,
    responses(
        (status = 200, description = "Reaction sent"),
        (status = 404, description = "Instance, chat, or message not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn send_reaction(
    State(manager): State<Arc<InstanceManager>>,
    Path((instance_id, phone, message_id)): Path<(String, String, String)>,
    Json(request): Json<ReactionRequest>,
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

    let chat_service = instance.chat_service();
    match chat_service
        .send_reaction(&phone, &message_id, &request.emoji)
        .await
    {
        Ok(()) => Json(json!({
            "success": true,
            "phone": phone,
            "message_id": message_id,
            "emoji": request.emoji
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "reaction_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}

/// Reply to a specific message
///
/// Sends a reply to a specific message (quoted reply).
#[utoipa::path(
    post,
    path = "/api/v1/instances/{instance_id}/messages/{phone}/{message_id}/reply",
    tag = "Message",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)"),
        ("phone" = String, Path, description = "Phone number"),
        ("message_id" = String, Path, description = "Message ID to reply to")
    ),
    request_body = ReplyMessageRequest,
    responses(
        (status = 200, description = "Reply sent"),
        (status = 404, description = "Instance, chat, or message not found"),
        (status = 503, description = "Browser not running"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn send_reply(
    State(manager): State<Arc<InstanceManager>>,
    Path((instance_id, phone, message_id)): Path<(String, String, String)>,
    Json(request): Json<ReplyMessageRequest>,
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

    let chat_service = instance.chat_service();
    match chat_service
        .send_reply(&phone, &message_id, &request.text)
        .await
    {
        Ok(()) => Json(json!({
            "success": true,
            "phone": phone,
            "quoted_message_id": message_id,
            "text": request.text
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "reply_failed",
                "message": e.to_string()
            })),
        )
            .into_response(),
    }
}
