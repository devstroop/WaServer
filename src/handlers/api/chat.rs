//! Chat Handlers - Now uses AccountManager with X-Account-Id
//!
//! All chat routes require X-Account-Id header to identify which WhatsApp account to use.

use crate::{
    middleware::CurrentAccount,
    models::chat::{
        ChatListResponse, ErrorResponse, Message, MessageListResponse, MessageQueryParams,
        SendMessageResponse,
    },
};
use axum::{
    extract::{Multipart, Path, Query},
    http::StatusCode,
    response::Json,
    Extension,
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
            "Account is busy with another operation".to_string(),
        )
    } else if error_msg.contains("Browser not") || error_msg.contains("not initialized") {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Browser not available. Please restart the account.".to_string(),
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
#[utoipa::path(
    get,
    path = "/api/v1/chats",
    responses(
        (status = 200, description = "List of chats", body = ChatListResponse),
        (status = 400, description = "Missing X-Account-Id header", body = ErrorResponse),
        (status = 401, description = "Not authorized - scan QR first", body = ErrorResponse),
        (status = 503, description = "Service unavailable", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "WhatsApp - Chat"
)]
pub async fn list_chats(
    Extension(current): Extension<CurrentAccount>,
) -> Result<Json<ChatListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let account = current.0;

    // Check if browser is running
    if !account.browser_service().is_running().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Browser not running. Start account first via POST /api/admin/accounts/{id}/start".to_string(),
            }),
        ));
    }

    if account.is_busy().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Account is busy, please try again later".to_string(),
            }),
        ));
    }

    let result = account
        .execute_with_busy_flag(async { account.chat_service().get_chat_list().await })
        .await;

    match result {
        Ok(chats) => {
            let total = chats.len();
            info!("Account {} - Retrieved {} chats", account.id, total);
            Ok(Json(ChatListResponse { chats, total }))
        }
        Err(e) => {
            let error_msg = e.to_string();
            error!("Account {} - Error listing chats: {}", account.id, error_msg);
            let (status, msg) = categorize_error(&error_msg);
            Err((status, Json(ErrorResponse { error: msg })))
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
    path = "/api/v1/chats/{chat_id}",
    params(
        ("chat_id" = String, Path, description = "Phone number, contact name, or chat ID"),
        ("limit" = Option<u32>, Query, description = "Maximum messages to retrieve"),
        ("load_more" = Option<bool>, Query, description = "Load older messages")
    ),
    responses(
        (status = 200, description = "List of messages", body = MessageListResponse),
        (status = 400, description = "Missing X-Account-Id header", body = ErrorResponse),
        (status = 401, description = "Not authorized - scan QR first", body = ErrorResponse),
        (status = 404, description = "Chat not found", body = ErrorResponse),
        (status = 503, description = "Service unavailable", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "WhatsApp - Chat"
)]
pub async fn get_chat_messages(
    Extension(current): Extension<CurrentAccount>,
    Path(chat_id): Path<String>,
    Query(params): Query<MessageQueryParams>,
) -> Result<Json<MessageListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let account = current.0;

    // Check if browser is running
    if !account.browser_service().is_running().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Browser not running. Start account first.".to_string(),
            }),
        ));
    }

    if account.is_busy().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Account is busy, please try again later".to_string(),
            }),
        ));
    }

    debug!("Account {} - Getting messages for chat: {}", account.id, chat_id);

    let limit = params.limit;
    let load_more = params.load_more.unwrap_or(false);

    let result = account
        .execute_with_busy_flag(async {
            account
                .chat_service()
                .get_messages(&chat_id, limit, load_more)
                .await
        })
        .await;

    match result {
        Ok(response) => {
            info!(
                "Account {} - Retrieved {} messages from chat {}",
                account.id, response.total, chat_id
            );
            Ok(Json(response))
        }
        Err(e) => {
            let error_msg = e.to_string();
            error!("Account {} - Error getting messages: {}", account.id, error_msg);

            if error_msg.contains("Invalid phone") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: "Chat not found or invalid phone number".to_string(),
                    }),
                ))
            } else {
                let (status, msg) = categorize_error(&error_msg);
                Err((status, Json(ErrorResponse { error: msg })))
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
    path = "/api/v1/chats/events",
    responses(
        (status = 200, description = "New incoming messages", body = Vec<crate::models::chat::MessageInfo>),
        (status = 400, description = "Missing X-Account-Id header", body = ErrorResponse),
        (status = 401, description = "Not authorized", body = ErrorResponse),
        (status = 503, description = "Service unavailable", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "WhatsApp - Chat"
)]
pub async fn watch_messages(
    Extension(current): Extension<CurrentAccount>,
) -> Result<Json<Vec<crate::models::chat::MessageInfo>>, (StatusCode, Json<ErrorResponse>)> {
    let account = current.0;

    if !account.browser_service().is_running().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Browser not running".to_string(),
            }),
        ));
    }

    if account.is_busy().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Account is busy".to_string(),
            }),
        ));
    }

    let result = account
        .execute_with_busy_flag(async { account.chat_service().watch_messages().await })
        .await;

    match result {
        Ok(messages) => {
            debug!("Account {} - Watch found {} new messages", account.id, messages.len());
            Ok(Json(messages))
        }
        Err(e) => {
            let error_msg = e.to_string();
            let (status, msg) = categorize_error(&error_msg);
            Err((status, Json(ErrorResponse { error: msg })))
        }
    }
}

// ============================================================================
// Send Message Endpoint
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
    path = "/api/v1/messages",
    responses(
        (status = 200, description = "Message sent successfully", body = SendMessageResponse),
        (status = 400, description = "Bad request - Phone is required and either text or file must be provided", body = ErrorResponse),
        (status = 503, description = "Service unavailable - browser not running or account busy", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "WhatsApp - Chat"
)]
pub async fn send_message(
    Extension(current): Extension<CurrentAccount>,
    mut multipart: Multipart,
) -> Result<Json<SendMessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    let account = current.0;

    // Check if browser is running
    if !account.browser_service().is_running().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Browser not running. Start account first via POST /api/admin/accounts/{id}/start".to_string(),
            }),
        ));
    }

    // Check if account is busy
    if account.is_busy().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Account is busy processing another operation, please try again later"
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
                                Json(ErrorResponse {
                                    error: "Failed to create attachments directory".to_string(),
                                }),
                            ));
                        }

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

    debug!("Account {} - Processing send message request for phone: {}", account.id, phone);

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
    let result = account
        .execute_with_busy_flag(async {
            account
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
            account.track_message_sent();
            info!("Account {} - Message sent successfully to {} (id: {})", account.id, phone, msg_id);
            Ok(Json(SendMessageResponse {
                status: "Message sent successfully".to_string(),
                message_id: msg_id,
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();
            account.track_error();
            error!("Account {} - Error sending message: {}", account.id, error_msg);
            
            let (status, msg) = categorize_error(&error_msg);
            Err((status, Json(ErrorResponse { error: msg })))
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
    path = "/api/v1/messages/{message_id}",
    params(
        ("message_id" = String, Path, description = "Message ID to retrieve")
    ),
    responses(
        (status = 200, description = "Message details", body = Message),
        (status = 400, description = "Missing X-Account-Id header", body = ErrorResponse),
        (status = 404, description = "Message not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "WhatsApp - Messages"
)]
pub async fn get_message(
    Extension(current): Extension<CurrentAccount>,
    Path(message_id): Path<String>,
) -> Result<Json<Message>, (StatusCode, Json<ErrorResponse>)> {
    let account = current.0;
    let db = account.database();

    match db.get_message(&message_id) {
        Ok(Some(msg)) => Ok(Json(Message::from(msg))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Message not found".to_string(),
            }),
        )),
        Err(e) => {
            error!("Account {} - Failed to get message: {}", account.id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to get message: {}", e),
                }),
            ))
        }
    }
}
