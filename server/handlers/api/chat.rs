//! Chat Handlers - Uses path parameter {instance_id}
//!
//! All chat and message routes use path parameter /api/v1/instances/{instance_id}/... to identify which WhatsApp instance to use.

use std::sync::Arc;

use crate::{
    models::chat::{
        ChatListResponse, ErrorResponse, MessageListResponse, MessageQueryParams,
        SendMessageParams, SendMessageResponse,
    },
    services::InstanceManager,
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use tokio::fs;
use tracing::{debug, error, info, warn};
use utoipa;
use uuid::Uuid;

// ============================================================================
// Error Handling Utilities
// ============================================================================

/// Categorize errors and return appropriate HTTP status and message
fn categorize_error(error_msg: &str) -> (StatusCode, String) {
    if error_msg.contains("Not authorized") || error_msg.contains("not authorized") {
        (StatusCode::UNAUTHORIZED, error_msg.to_string())
    } else if error_msg.contains("timed out") || error_msg.contains("unresponsive") {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Browser operation timed out. The browser may be unresponsive.".to_string(),
        )
    } else if error_msg.contains("is busy") {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Instance is busy with another operation".to_string(),
        )
    } else if error_msg.contains("Browser not") || error_msg.contains("not initialized") {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Browser not available. Please restart the instance.".to_string(),
        )
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, error_msg.to_string())
    }
}

// ============================================================================
// Chat List Endpoint
// ============================================================================

/// List all visible chats/conversations
///
/// Returns a list of chats from the WhatsApp sidebar with:
/// - Contact/group name
/// - Last message preview
/// - Timestamp
/// - Unread count
// Hidden from Swagger - internal use only
// #[utoipa::path(
//     get,
//     path = "/api/v1/instances/{instance_id}/chats",
//     params(
//         ("instance_id" = String, Path, description = "Instance ID (UUID)")
//     ),
//     responses(
//         (status = 200, description = "List of chats", body = ChatListResponse),
//         (status = 404, description = "Instance not found", body = ErrorResponse),
//         (status = 401, description = "Not authorized - scan QR first", body = ErrorResponse),
//         (status = 503, description = "Service unavailable", body = ErrorResponse),
//         (status = 500, description = "Internal server error", body = ErrorResponse)
//     ),
//     security(
//         ("bearer_auth" = [])
//     ),
//     tag = "Messaging"
// )]
pub async fn list_chats(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
) -> Result<Json<ChatListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let instance = match manager.get_instance(&instance_id).await {
        Some(acc) => acc,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(format!(
                    "Instance '{}' not found",
                    instance_id
                ))),
            ));
        }
    };

    // Ensure browser is warm (auto-warms if sleeping)
    if let Err(e) = instance.ensure_warm().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(format!(
                "Failed to warm up instance: {}",
                e
            ))),
        ));
    }

    if instance.is_busy().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "Instance is busy, please try again later".to_string(),
            )),
        ));
    }

    let result = instance
        .execute_with_busy_flag(async { instance.chat_service().get_chat_list().await })
        .await;

    match result {
        Ok(chats) => {
            let total = chats.len();
            info!("Instance {} - Retrieved {} chats", instance.id, total);
            Ok(Json(ChatListResponse { chats, total }))
        }
        Err(e) => {
            let error_msg = e.to_string();
            error!(
                "Instance {} - Error listing chats: {}",
                instance.id, error_msg
            );
            let (status, msg) = categorize_error(&error_msg);
            Err((status, Json(ErrorResponse::new(msg))))
        }
    }
}

// ============================================================================
// Messages Endpoint
// ============================================================================

/// Get messages from a phone number
///
/// Retrieves messages from a conversation with the specified phone number.
///
/// Query parameters:
/// - `limit`: Maximum number of messages (default: 50)
/// - `load_more`: Scroll up to load older messages (default: false)
// Hidden from Swagger - internal use only
// #[utoipa::path(
//     get,
//     path = "/api/v1/instances/{instance_id}/messages/{phone}",
//     params(
//         ("instance_id" = String, Path, description = "Instance ID (UUID)"),
//         ("phone" = String, Path, description = "Phone number (e.g., 919876543210)"),
//         ("limit" = Option<u32>, Query, description = "Maximum messages to retrieve"),
//         ("load_more" = Option<bool>, Query, description = "Load older messages")
//     ),
//     responses(
//         (status = 200, description = "List of messages", body = MessageListResponse),
//         (status = 404, description = "Instance or phone not found", body = ErrorResponse),
//         (status = 401, description = "Not authorized - scan QR first", body = ErrorResponse),
//         (status = 503, description = "Service unavailable", body = ErrorResponse),
//         (status = 500, description = "Internal server error", body = ErrorResponse)
//     ),
//     security(
//         ("bearer_auth" = [])
//     ),
//     tag = "Messaging"
// )]
pub async fn get_chat_messages(
    State(manager): State<Arc<InstanceManager>>,
    Path((instance_id, phone)): Path<(String, String)>,
    Query(params): Query<MessageQueryParams>,
) -> Result<Json<MessageListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let instance = match manager.get_instance(&instance_id).await {
        Some(acc) => acc,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(format!(
                    "Instance '{}' not found",
                    instance_id
                ))),
            ));
        }
    };

    // Ensure browser is warm (auto-warms if sleeping)
    if let Err(e) = instance.ensure_warm().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(format!(
                "Failed to warm up instance: {}",
                e
            ))),
        ));
    }

    if instance.is_busy().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "Instance is busy, please try again later".to_string(),
            )),
        ));
    }

    debug!(
        "Instance {} - Getting messages for phone: {}",
        instance.id, phone
    );

    let limit = params.limit;
    let load_more = params.load_more.unwrap_or(false);

    let result = instance
        .execute_with_busy_flag(async {
            instance
                .chat_service()
                .get_messages(&phone, limit, load_more)
                .await
        })
        .await;

    match result {
        Ok(response) => {
            info!(
                "Instance {} - Retrieved {} messages from phone {}",
                instance.id, response.total, phone
            );
            Ok(Json(response))
        }
        Err(e) => {
            let error_msg = e.to_string();
            error!(
                "Instance {} - Error getting messages: {}",
                instance.id, error_msg
            );

            if error_msg.contains("Invalid phone") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse::new(
                        "Chat not found or invalid phone number".to_string(),
                    )),
                ))
            } else {
                let (status, msg) = categorize_error(&error_msg);
                Err((status, Json(ErrorResponse::new(msg))))
            }
        }
    }
}

// ============================================================================
// Send Message Endpoint
// ============================================================================

/// Send a message (text or file attachment)
///
/// Send a text message and/or file attachment to a phone number.
/// - `phone` (query param, required): Recipient phone number
/// - `text` (query param, optional): Message text or caption for file
/// - `file` (multipart body, optional): File attachment
///
/// **Note:** At least one of `text` or `file` must be provided.
#[utoipa::path(
    post,
    path = "/api/v1/instances/{instance_id}/send",
    params(
        ("instance_id" = String, Path, description = "Instance ID (UUID)"),
        SendMessageParams
    ),
    request_body(
        content_type = "multipart/form-data",
        content = SendMessageRequest,
        description = "Optional file attachment"
    ),
    responses(
        (status = 200, description = "Message sent successfully", body = SendMessageResponse),
        (status = 400, description = "Bad request - phone required, at least text or file must be provided", body = ErrorResponse),
        (status = 404, description = "Instance not found", body = ErrorResponse),
        (status = 503, description = "Service unavailable - browser not active or instance busy", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Messaging"
)]
pub async fn send_message(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
    Query(params): Query<SendMessageParams>,
    multipart: Option<Multipart>,
) -> Result<Json<SendMessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    let phone = params.phone;
    let text = params.text;
    let mut attachment_path: Option<String> = None;
    let mut original_filename: Option<String> = None;

    // Process multipart data if present (for file uploads)
    if let Some(mut mp) = multipart {
        while let Some(field) = mp.next_field().await.map_err(|e| {
            error!("Error processing multipart data: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Invalid multipart data".to_string())),
            )
        })? {
            if let Some(name) = field.name() {
                if name == "file" {
                    let filename = field.file_name().map(|s| s.to_string());
                    if let Some(filename) = filename {
                        debug!("Processing file attachment: {}", filename);
                        original_filename = Some(filename.clone());

                        let file_extension = std::path::Path::new(&filename)
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("bin");

                        let unique_filename = format!("{}.{}", Uuid::new_v4(), file_extension);
                        let staging_path = format!("data/attachments/.staging/{}", unique_filename);

                        if let Err(e) = fs::create_dir_all("data/attachments/.staging").await {
                            error!("Failed to create staging directory: {}", e);
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ErrorResponse::new(
                                    "Failed to create attachments directory".to_string(),
                                )),
                            ));
                        }

                        let data = field.bytes().await.map_err(|e| {
                            error!("Error reading file data: {}", e);
                            (
                                StatusCode::BAD_REQUEST,
                                Json(ErrorResponse::new("Failed to read file data".to_string())),
                            )
                        })?;

                        if let Err(e) = fs::write(&staging_path, &data).await {
                            error!("Failed to save attachment file: {}", e);
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ErrorResponse::new(
                                    "Failed to save attachment file".to_string(),
                                )),
                            ));
                        }

                        attachment_path = Some(staging_path);
                        info!("File attachment saved: {} bytes ({})", data.len(), filename);
                    }
                }
            }
        }
    }

    // Validate that at least text or file is provided (before slow operations)
    if attachment_path.is_none() && text.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "Either text message or file attachment must be provided".to_string(),
            )),
        ));
    }

    // Now get instance and warmup (slow operations start here)
    let instance = match manager.get_instance(&instance_id).await {
        Some(acc) => acc,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(format!(
                    "Instance '{}' not found",
                    instance_id
                ))),
            ));
        }
    };

    // Ensure browser is warm (auto-warms if sleeping)
    if let Err(e) = instance.ensure_warm().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(format!(
                "Failed to warm up instance: {}",
                e
            ))),
        ));
    }

    // Check if instance is busy
    if instance.is_busy().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "Instance is busy processing another operation, please try again later".to_string(),
            )),
        ));
    }

    // phone from URL path is the recipient
    debug!(
        "Instance {} - Processing send message request for chat: {}",
        instance.id, phone
    );

    // Move attachment from staging to phone-specific directory
    let final_attachment_path = if let Some(ref staging_path) = attachment_path {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let phone_dir = format!("data/attachments/{}/{}", phone.replace("+", ""), today);

        if let Err(e) = fs::create_dir_all(&phone_dir).await {
            error!("Failed to create phone attachments directory: {}", e);
            Some(staging_path.clone())
        } else {
            let filename = std::path::Path::new(staging_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("attachment");

            let final_path = format!("{}/{}", phone_dir, filename);

            match fs::rename(staging_path, &final_path).await {
                Ok(_) => {
                    info!("Attachment moved to: {}", final_path);
                    Some(final_path)
                }
                Err(e) => {
                    warn!("Failed to move attachment, using staging path: {}", e);
                    Some(staging_path.clone())
                }
            }
        }
    } else {
        None
    };

    // Send the message
    let result = instance
        .execute_with_busy_flag(async {
            instance
                .chat_service()
                .send_message(
                    &phone,
                    text.as_deref(),
                    final_attachment_path.as_deref(),
                    None,
                )
                .await
        })
        .await;

    if let Some(ref path) = final_attachment_path {
        if let Some(ref orig_name) = original_filename {
            debug!("Attachment persisted: {} -> {}", orig_name, path);
        }
    }

    match result {
        Ok(_) => {
            let msg_id = Uuid::new_v4().to_string();
            instance.track_message_sent();
            info!(
                "Instance {} - Message sent successfully to {} (id: {})",
                instance.id, phone, msg_id
            );
            Ok(Json(SendMessageResponse {
                status: "Message sent successfully".to_string(),
                message_id: msg_id,
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();
            instance.track_error();
            error!(
                "Instance {} - Error sending message: {}",
                instance.id, error_msg
            );

            let (status, msg) = categorize_error(&error_msg);
            Err((status, Json(ErrorResponse::new(msg))))
        }
    }
}
