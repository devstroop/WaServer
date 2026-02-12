use crate::{
    models::chat::{
        ChatListResponse, ErrorResponse, Message, MessageListResponse, MessageQueryParams,
        SendMessageResponse,
    },
    services::whatsapp::WhatsAppService,
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, error, info, warn};
use utoipa;
use uuid::Uuid;

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
#[utoipa::path(
    get,
    path = "/api/chats",
    responses(
        (status = 200, description = "List of chats", body = ChatListResponse),
        (status = 401, description = "Not authorized - scan QR first", body = ErrorResponse),
        (status = 503, description = "Service unavailable", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Chat"
)]
pub async fn list_chats(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> Result<Json<ChatListResponse>, (StatusCode, Json<ErrorResponse>)> {
    if whatsapp_service.is_busy().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Service is busy, please try again later".to_string(),
            }),
        ));
    }

    let result = whatsapp_service
        .execute_with_busy_flag(async { whatsapp_service.chat_service().get_chat_list().await })
        .await;

    match result {
        Ok(chats) => {
            let total = chats.len();
            info!("Retrieved {} chats from WhatsApp", total);
            Ok(Json(ChatListResponse { chats, total }))
        }
        Err(e) => {
            let error_msg = e.to_string();
            error!("Error listing chats: {}", error_msg);

            if error_msg.contains("Not authorized") {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse { error: error_msg }),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Error listing chats: {}", error_msg),
                    }),
                ))
            }
        }
    }
}

// ============================================================================
// Messages Endpoint
// ============================================================================

/// Get messages from a specific chat
///
/// Retrieves messages from the specified chat. The chat_id can be:
/// - A phone number (e.g., "919876543210")
/// - A contact name (e.g., "John Doe")
/// - A chat ID (e.g., "919876543210@c.us")
///
/// Query parameters:
/// - `limit`: Maximum number of messages (default: 50)
/// - `load_more`: Scroll up to load older messages (default: false)
#[utoipa::path(
    get,
    path = "/api/chats/{chat_id}",
    params(
        ("chat_id" = String, Path, description = "Phone number, contact name, or chat ID"),
        ("limit" = Option<u32>, Query, description = "Maximum messages to retrieve"),
        ("load_more" = Option<bool>, Query, description = "Load older messages")
    ),
    responses(
        (status = 200, description = "List of messages", body = MessageListResponse),
        (status = 401, description = "Not authorized - scan QR first", body = ErrorResponse),
        (status = 404, description = "Chat not found", body = ErrorResponse),
        (status = 503, description = "Service unavailable", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Chat"
)]
pub async fn get_chat_messages(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
    Path(chat_id): Path<String>,
    Query(params): Query<MessageQueryParams>,
) -> Result<Json<MessageListResponse>, (StatusCode, Json<ErrorResponse>)> {
    if whatsapp_service.is_busy().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Service is busy, please try again later".to_string(),
            }),
        ));
    }

    debug!("Getting messages for chat: {}", chat_id);

    let limit = params.limit;
    let load_more = params.load_more.unwrap_or(false);

    let result = whatsapp_service
        .execute_with_busy_flag(async {
            whatsapp_service
                .chat_service()
                .get_messages(&chat_id, limit, load_more)
                .await
        })
        .await;

    match result {
        Ok(response) => {
            info!(
                "Retrieved {} messages from chat {}",
                response.total, chat_id
            );
            Ok(Json(response))
        }
        Err(e) => {
            let error_msg = e.to_string();
            error!("Error getting messages: {}", error_msg);

            if error_msg.contains("Not authorized") {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse { error: error_msg }),
                ))
            } else if error_msg.contains("Invalid phone") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Chat not found or invalid phone number".to_string(),
                    }),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Error getting messages: {}", error_msg),
                    }),
                ))
            }
        }
    }
}

// ============================================================================
// Watch/Poll Endpoint for Incoming Messages
// ============================================================================

/// Watch for new incoming messages
///
/// Returns any unread messages visible in the chat list.
/// Useful for polling new messages without navigating to each chat.
#[utoipa::path(
    get,
    path = "/api/chats/events",
    responses(
        (status = 200, description = "New incoming messages", body = Vec<crate::models::chat::MessageInfo>),
        (status = 401, description = "Not authorized", body = ErrorResponse),
        (status = 503, description = "Service unavailable", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Chat"
)]
pub async fn watch_messages(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> Result<Json<Vec<crate::models::chat::MessageInfo>>, (StatusCode, Json<ErrorResponse>)> {
    if whatsapp_service.is_busy().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Service is busy".to_string(),
            }),
        ));
    }

    let result = whatsapp_service
        .execute_with_busy_flag(async { whatsapp_service.chat_service().watch_messages().await })
        .await;

    match result {
        Ok(messages) => {
            debug!("Watch found {} new messages", messages.len());
            Ok(Json(messages))
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Not authorized") {
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse { error: error_msg }),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Error watching messages: {}", error_msg),
                    }),
                ))
            }
        }
    }
}

// ============================================================================
// Send Message Endpoint (existing)
// ============================================================================

/// Send a message (text or file attachment)
///
/// This endpoint accepts multipart form data with the following fields:
/// - `phone` (string, required): Recipient phone number  
/// - `text` (string, optional): Message text content
/// - `file` (file, optional): File attachment
///
/// **Note:** At least one of `text` or `file` must be provided.
#[utoipa::path(
    post,
    path = "/api/messages",
    responses(
        (status = 200, description = "Message sent successfully", body = SendMessageResponse),
        (status = 400, description = "Bad request - Phone is required and either text or file must be provided", body = ErrorResponse),
        (status = 503, description = "Service unavailable - too many requests or service busy", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Chat"
)]
pub async fn send_message(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
    mut multipart: Multipart,
) -> Result<Json<SendMessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check if service is busy
    if whatsapp_service.is_busy().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Service is busy processing another operation, please try again later"
                    .to_string(),
            }),
        ));
    }

    let mut phone: Option<String> = None;
    let mut text: Option<String> = None;
    let mut attachment_path: Option<String> = None;
    let mut original_filename: Option<String> = None;

    // Process multipart data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Error processing multipart data: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid multipart data".to_string(),
            }),
        )
    })? {
        if let Some(name) = field.name() {
            match name {
                "phone" => {
                    phone = Some(field.text().await.map_err(|e| {
                        error!("Error reading phone field: {}", e);
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorResponse {
                                error: "Invalid phone field".to_string(),
                            }),
                        )
                    })?);
                }
                "text" => {
                    text = Some(field.text().await.map_err(|e| {
                        error!("Error reading text field: {}", e);
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorResponse {
                                error: "Invalid text field".to_string(),
                            }),
                        )
                    })?);
                }
                "file" => {
                    let filename = field.file_name().map(|s| s.to_string());
                    if let Some(filename) = filename {
                        debug!("Processing file attachment: {}", filename);
                        original_filename = Some(filename.clone());

                        // Store in attachments directory (will be organized by phone after we have it)
                        let file_extension = std::path::Path::new(&filename)
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("bin");

                        // Use UUID + original extension for unique filename
                        let unique_filename = format!("{}.{}", Uuid::new_v4(), file_extension);

                        // Temporary staging path until we know the phone number
                        let staging_path = format!("data/attachments/.staging/{}", unique_filename);

                        // Ensure staging directory exists
                        if let Err(e) = fs::create_dir_all("data/attachments/.staging").await {
                            error!("Failed to create staging directory: {}", e);
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ErrorResponse {
                                    error: "Failed to create attachments directory".to_string(),
                                }),
                            ));
                        }

                        // Save file data
                        let data = field.bytes().await.map_err(|e| {
                            error!("Error reading file data: {}", e);
                            (
                                StatusCode::BAD_REQUEST,
                                Json(ErrorResponse {
                                    error: "Failed to read file data".to_string(),
                                }),
                            )
                        })?;

                        if let Err(e) = fs::write(&staging_path, &data).await {
                            error!("Failed to save attachment file: {}", e);
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ErrorResponse {
                                    error: "Failed to save attachment file".to_string(),
                                }),
                            ));
                        }

                        attachment_path = Some(staging_path);
                        info!("File attachment saved: {} bytes ({})", data.len(), filename);
                    }
                }
                _ => {
                    // Skip unknown fields
                    debug!("Skipping unknown field: {}", name);
                }
            }
        }
    }

    // Validate required fields
    let phone = phone.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Phone number is required".to_string(),
            }),
        )
    })?;

    debug!("Processing send message request for phone: {}", phone);

    // Validate that at least text or file is provided
    if attachment_path.is_none() && text.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Either text message or file attachment must be provided".to_string(),
            }),
        ));
    }

    // Move attachment from staging to phone-specific directory
    let final_attachment_path = if let Some(ref staging_path) = attachment_path {
        // Create phone-specific directory: data/attachments/{phone}/{date}/
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let phone_dir = format!("data/attachments/{}/{}", phone.replace("+", ""), today);

        if let Err(e) = fs::create_dir_all(&phone_dir).await {
            error!("Failed to create phone attachments directory: {}", e);
            // Continue with staging path if we can't move it
            Some(staging_path.clone())
        } else {
            // Extract filename from staging path
            let filename = std::path::Path::new(staging_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("attachment");

            let final_path = format!("{}/{}", phone_dir, filename);

            // Move file from staging to final location
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
    let result = whatsapp_service
        .execute_with_busy_flag(async {
            whatsapp_service
                .chat_service()
                .send_message(
                    &phone,
                    text.as_deref(),
                    final_attachment_path.as_deref(),
                    None, // Use default timeout
                )
                .await
        })
        .await;

    // Log attachment info (attachments are now persisted, not cleaned up)
    if let Some(ref path) = final_attachment_path {
        if let Some(ref orig_name) = original_filename {
            debug!("Attachment persisted: {} -> {}", orig_name, path);
        }
    }

    // Handle result
    match result {
        Ok(_) => {
            // Generate a message ID for tracking
            let msg_id = Uuid::new_v4().to_string();
            info!("Message sent successfully to {} (id: {})", phone, msg_id);
            Ok(Json(SendMessageResponse {
                status: "Message sent successfully".to_string(),
                message_id: msg_id,
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();
            error!("Error sending message: {}", error_msg);

            if error_msg.contains("Not authorized") {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse { error: error_msg }),
                ))
            } else if error_msg.contains("busy") {
                Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ErrorResponse {
                        error: "Service is busy, please try again later".to_string(),
                    }),
                ))
            } else if error_msg.contains("timeout") || error_msg.contains("Timed out") {
                Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(ErrorResponse {
                        error: "Operation timed out waiting for the service to become available"
                            .to_string(),
                    }),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Error sending WhatsApp message: {}", error_msg),
                    }),
                ))
            }
        }
    }
}

// ============================================================================
// Message by ID Endpoint
// ============================================================================

/// Get a specific message by ID
///
/// Returns full message details including status
#[utoipa::path(
    get,
    path = "/api/messages/{message_id}",
    params(
        ("message_id" = String, Path, description = "Message ID to retrieve")
    ),
    responses(
        (status = 200, description = "Message details", body = Message),
        (status = 404, description = "Message not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Messages"
)]
pub async fn get_message(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
    Path(message_id): Path<String>,
) -> Result<Json<Message>, (StatusCode, Json<ErrorResponse>)> {
    let db = whatsapp_service.database();

    match db.get_message(&message_id) {
        Ok(Some(msg)) => Ok(Json(Message::from(msg))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Message not found".to_string(),
            }),
        )),
        Err(e) => {
            error!("Failed to get message: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to get message: {}", e),
                }),
            ))
        }
    }
}
