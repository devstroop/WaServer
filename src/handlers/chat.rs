use crate::{
    models::chat::{SendMessageResponse},
    models::auth::ErrorResponse,
    services::whatsapp::WhatsAppService,
};
use axum::{
    extract::{Multipart, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, error, info, warn};
use utoipa;
use uuid::Uuid;

/// Query parameters for sending messages
#[derive(Debug, Deserialize)]
pub struct SendMessageQuery {
    /// Recipient phone number
    pub phone: String,
    /// Message text (optional if sending file)
    pub text: Option<String>,
}

/// Send a message (text or file attachment)
#[utoipa::path(
    post,
    path = "/api/chat/send",
    params(
        ("phone" = String, Query, description = "Recipient phone number"),
        ("text" = Option<String>, Query, description = "Message text (optional if sending file)")
    ),
    request_body(content = String, description = "Multipart form data with optional file attachment", content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Message sent successfully", body = SendMessageResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 503, description = "Service unavailable - too many requests or service busy", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Chat"
)]
pub async fn send_message(
    Query(params): Query<SendMessageQuery>,
    State(whatsapp_service): State<Arc<WhatsAppService>>,
    mut multipart: Multipart,
) -> Result<Json<SendMessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check if service is busy
    if whatsapp_service.is_busy().await {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "Service is busy processing another operation, please try again later".to_string(),
            }),
        ));
    }

    debug!("Processing send message request for phone: {}", params.phone);

    let mut attachment_path: Option<String> = None;
    let mut cleanup_file = false;

    // Process multipart data for file attachments
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
            if name == "file" {
                if let Some(filename) = field.file_name() {
                    debug!("Processing file attachment: {}", filename);
                    
                    // Create temporary file
                    let file_extension = std::path::Path::new(filename)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("tmp");
                    
                    let temp_filename = format!("{}.{}", Uuid::new_v4(), file_extension);
                    let temp_path = format!("./.temp/{}", temp_filename);
                    
                    // Ensure temp directory exists
                    if let Err(e) = fs::create_dir_all("./.temp").await {
                        error!("Failed to create temp directory: {}", e);
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Failed to create temporary directory".to_string(),
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

                    if let Err(e) = fs::write(&temp_path, &data).await {
                        error!("Failed to save temp file: {}", e);
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Failed to save temporary file".to_string(),
                            }),
                        ));
                    }

                    attachment_path = Some(temp_path);
                    cleanup_file = true;
                    info!("File attachment saved: {} bytes", data.len());
                }
            }
        }
    }

    // Validate request
    if attachment_path.is_none() && params.text.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Either text message or file attachment must be provided".to_string(),
            }),
        ));
    }

    // Send the message
    let result = whatsapp_service.execute_with_busy_flag(async {
        whatsapp_service.chat_service().send_message(
            &params.phone,
            params.text.as_deref(),
            attachment_path.as_deref(),
            None, // Use default timeout
        ).await
    }).await;

    // Cleanup temporary file
    if cleanup_file {
        if let Some(ref path) = attachment_path {
            if let Err(e) = fs::remove_file(path).await {
                warn!("Failed to cleanup temp file {}: {}", path, e);
            } else {
                debug!("Cleaned up temp file: {}", path);
            }
        }
    }

    // Handle result
    match result {
        Ok(_) => {
            info!("Message sent successfully to {}", params.phone);
            Ok(Json(SendMessageResponse {
                status: "Message sent successfully".to_string(),
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
                        error: "Operation timed out waiting for the service to become available".to_string(),
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
