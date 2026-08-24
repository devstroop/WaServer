//! Messaging handler — thin, extracted from `handlers/api/chat.rs:288` + `services/whatsapp/chat.rs:911`
//! Flow: DTO validate → multipart stage (optional) → resolve instance →
//! `SendService` (policy → rate limit → browser port) → response.
//! Browser reached only via `ManagerBrowserAdapter` (`BrowserSendPort`) so unit tests mock it.

use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    application::messaging::{
        policy::E164Validator,
        send::{SendMessageCommand, SendService},
    },
    domain::{
        instance::InstanceId,
        messaging::{MediaType, MessageStatus},
        shared::error::DomainError,
    },
    interfaces::http::dto::messaging::{SendMessageRequestDto, SendMessageResponseDto},
    services::{InstanceManager, ManagerBrowserAdapter},
};

/// Staging base dir for multipart uploads (legacy-compatible path)
pub const STAGING_DIR: &str = "data/attachments/.staging";

/// Categorize send errors → HTTP status (typed DomainError mapping)
fn categorize_send_error(err: &DomainError) -> StatusCode {
    match err {
        DomainError::NotFound { .. } => StatusCode::NOT_FOUND,
        DomainError::PermissionDenied { .. } => StatusCode::UNAUTHORIZED,
        DomainError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
        DomainError::InvalidInput { .. } | DomainError::Validation(_) => StatusCode::BAD_REQUEST,
        DomainError::Conflict { .. } | DomainError::Internal(_) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Map upload filename → domain MediaType via extension
pub fn media_type_for_filename(name: &str) -> MediaType {
    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" => MediaType::Image,
        "mp4" | "mov" | "avi" | "mkv" => MediaType::Video,
        "mp3" | "ogg" | "wav" | "m4a" | "opus" => MediaType::Voice,
        _ => MediaType::Document,
    }
}

/// Deterministic staging path for an uploaded filename
pub fn staging_path_for(filename: &str) -> String {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    format!("{}/{}.{}", STAGING_DIR, uuid::Uuid::new_v4(), ext)
}

async fn stage_upload(
    field: axum::extract::multipart::Field<'_>,
) -> Result<(String, usize), String> {
    let filename = field
        .file_name()
        .map(str::to_string)
        .unwrap_or_else(|| "upload.bin".into());
    let path = staging_path_for(&filename);
    tokio::fs::create_dir_all(STAGING_DIR)
        .await
        .map_err(|e| format!("failed to create attachments directory: {}", e))?;
    let data = field
        .bytes()
        .await
        .map_err(|e| format!("failed to read file data: {}", e))?;
    let len = data.len();
    tokio::fs::write(&path, &data)
        .await
        .map_err(|e| format!("failed to save attachment file: {}", e))?;
    Ok((path, len))
}

/// Build a SendService wired to the manager via ports (#7)
/// Rate limiter is the shared process-wide instance (`InstanceManager.rate_limiter`)
/// so windows survive across requests; limits come from per-instance config.
fn build_send_service(manager: Arc<InstanceManager>) -> SendService {
    SendService::new(
        Arc::new(E164Validator),
        Arc::new(ManagerBrowserAdapter {
            manager: manager.clone(),
        }),
        manager.rate_limiter.clone(),
    )
}

/// `POST /api/v1/instances/:instance_id/send?phone=&text=` + optional multipart `file`
/// Query `phone`/`text`; file staged to disk then sent as media. Business flow in `SendService`.
#[utoipa::path(post, path = "/api/v1/instances/{instance_id}/send", tag = "Messaging", params(("instance_id" = String, Path, description = "Instance ID"), SendMessageRequestDto), request_body(content = SendMessageRequestDto, content_type = "multipart/form-data", description = "Optional file attachment (`file` field)"), responses((status=200, body=SendMessageResponseDto), (status=400, description="Bad request - phone required, at least text or file must be provided", body=crate::models::chat::ErrorResponse), (status=401, description="Not authorized"), (status=429, description="Rate limited"), (status=503, description="Instance busy")), security(("bearer_auth" = [])))]
pub async fn send_message(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
    Query(query): Query<SendMessageRequestDto>,
    multipart: Option<Multipart>,
) -> impl IntoResponse {
    // 1. DTO boundary validation
    if let Err(e) = query.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":"invalid_request","message":e})),
        )
            .into_response();
    }

    // 2. Resolve instance (UUID or phone) via registry
    let instance_uuid: InstanceId = match manager.registry.resolve(&instance_id).await {
        Some(id) => id,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "instance_not_found",
                    "message": format!("Instance '{}' not found", instance_id)
                })),
            )
                .into_response()
        }
    };

    // 3. Stage optional multipart upload before slow operations
    let mut media_path: Option<String> = None;
    let mut media_type = MediaType::None;
    if let Some(mut mp) = multipart {
        while let Ok(Some(field)) = mp.next_field().await {
            if field.name() == Some("file") {
                match stage_upload(field).await {
                    Ok((path, len)) => {
                        tracing::info!(path = %path, bytes = len, "attachment staged");
                        media_type = media_type_for_filename(&path);
                        media_path = Some(path);
                    }
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"error": "send_failed", "message": e})),
                        )
                            .into_response()
                    }
                }
            }
        }
    }

    // 4. At least text or file required (before warmup/rate spend)
    if query.text.is_none() && media_path.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_request", "message": "Either text message or file attachment must be provided"})),
        )
            .into_response();
    }

    // 5. Execute use-case: validator → policy → rate limit → browser port
    let service = build_send_service(manager);
    let cmd = SendMessageCommand {
        instance: instance_uuid,
        to: query.phone.clone(),
        text: query.text.clone(),
        media_type,
        media_path,
    };

    match service.send(cmd).await {
        Ok(message_id) => {
            let resp = SendMessageResponseDto {
                status: MessageStatus::Sent.to_string(),
                success: true,
                message_id,
                phone: query.phone,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => {
            let status = categorize_send_error(&e);
            (
                status,
                Json(json!({"error": "send_failed", "message": e.to_string()})),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dto_validate() {
        let dto = SendMessageRequestDto {
            phone: "+1234567890".into(),
            text: Some("hi".into()),
        };
        assert!(dto.validate().is_ok());
        assert!(SendMessageRequestDto {
            phone: "".into(),
            text: None
        }
        .validate()
        .is_err());
    }

    #[test]
    fn test_categorize_send_error() {
        assert_eq!(
            categorize_send_error(&DomainError::not_found("instance", "x")),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            categorize_send_error(&DomainError::PermissionDenied {
                operation: "send".into()
            }),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            categorize_send_error(&DomainError::RateLimited {
                operation: "send".into(),
                retry_after_seconds: 60
            }),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            categorize_send_error(&DomainError::Validation("bad".into())),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            categorize_send_error(&DomainError::Internal("boom".into())),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_media_type_for_filename() {
        assert_eq!(media_type_for_filename("photo.jpg"), MediaType::Image);
        assert_eq!(media_type_for_filename("clip.MP4"), MediaType::Video);
        assert_eq!(media_type_for_filename("voice-note.ogg"), MediaType::Voice);
        assert_eq!(media_type_for_filename("report.pdf"), MediaType::Document);
        assert_eq!(media_type_for_filename("archive.zip"), MediaType::Document);
        assert_eq!(media_type_for_filename("noext"), MediaType::Document);
    }

    #[test]
    fn test_staging_path_format() {
        let p = staging_path_for("photo.jpeg");
        assert!(p.starts_with("data/attachments/.staging/"));
        assert!(p.ends_with(".jpeg"));
        let other = staging_path_for("photo.jpeg");
        assert_ne!(p, other, "paths must be unique per call");
    }

    #[tokio::test]
    async fn test_send_service_type_builds() {
        // Rate adapter window behavior verified in rate_limiter tests; here ensure type builds
        let _ = std::any::type_name::<SendService>();
    }
}
