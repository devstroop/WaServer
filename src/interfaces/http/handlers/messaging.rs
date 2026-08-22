//! Messaging handler — thin, extracted from `handlers/api/chat.rs:288` + `services/whatsapp/chat.rs:911` (part of #7)
//! Flow: DTO validate → `E164Validator` → `SendService` (rate → browser) → response.
//! Browser reached only via `ManagerBrowserAdapter` (`BrowserSendPort`) so unit tests mock it.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
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
    domain::{instance::InstanceId, messaging::MediaType},
    interfaces::http::dto::messaging::{SendMessageRequestDto, SendMessageResponseDto},
    services::{InstanceManager, ManagerBrowserAdapter, ManagerRateAdapter},
};

/// Categorize send errors → HTTP status (mirrors `handlers/api/chat.rs:27`)
fn categorize_send_error(err: &str) -> StatusCode {
    if err.contains("not found") {
        StatusCode::NOT_FOUND
    } else if err.contains("Not authorized") || err.contains("not authorized") {
        StatusCode::UNAUTHORIZED
    } else if err.contains("rate limited") {
        StatusCode::TOO_MANY_REQUESTS
    } else if err.contains("timed out")
        || err.contains("unresponsive")
        || err.contains("busy")
        || err.contains("warmup failed")
        || err.contains("Browser not")
    {
        StatusCode::SERVICE_UNAVAILABLE
    } else if err.contains("no content") {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

/// Build a SendService wired to the manager via ports (#7)
fn build_send_service(manager: Arc<InstanceManager>) -> SendService {
    SendService::new(
        Arc::new(E164Validator),
        Arc::new(ManagerBrowserAdapter {
            manager: manager.clone(),
        }),
        Arc::new(ManagerRateAdapter::new(manager)),
    )
}

/// `POST /api/v1/instances/:instance_id/send?phone=&text=` — thin handler
/// Query `phone` + `text` validated at DTO boundary; business flow in `SendService`.
#[utoipa::path(post, path = "/api/v1/instances/{instance_id}/send", tag = "Messaging", params(("instance_id" = String, Path, description = "Instance ID"), ("phone" = String, Query, description = "Recipient E.164"), ("text" = String, Query, description = "Message text")), responses((status=200, body=SendMessageResponseDto), (status=401, description="Not authorized"), (status=429, description="Rate limited"), (status=503, description="Instance busy")), security(("bearer_auth" = [])))]
pub async fn send_message(
    State(manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
    Query(query): Query<SendMessageRequestDto>,
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
    let resolved = match manager.registry.resolve(&instance_id).await {
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

    let instance_uuid: InstanceId = resolved;

    // 3. Execute use-case: validator → rate limit → browser port
    let service = build_send_service(manager);
    let cmd = SendMessageCommand {
        instance: instance_uuid,
        to: query.phone.clone(),
        text: query.text.clone(),
        media_type: MediaType::None,
        media_path: None,
    };

    match service.send(cmd).await {
        Ok(message_id) => {
            let resp = SendMessageResponseDto {
                success: true,
                message_id,
                phone: query.phone,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => {
            let status = categorize_send_error(&e);
            (status, Json(json!({"error": "send_failed", "message": e}))).into_response()
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
            phone: "123".into(),
            text: None
        }
        .validate()
        .is_err());
    }

    #[test]
    fn test_categorize_send_error() {
        assert_eq!(
            categorize_send_error("instance 'x' not found"),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            categorize_send_error("Not authorized"),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            categorize_send_error("rate limited: 60 msgs"),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            categorize_send_error("is busy"),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(categorize_send_error("no content"), StatusCode::BAD_REQUEST);
        assert_eq!(
            categorize_send_error("weird"),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_send_service_type_builds() {
        // Rate adapter window behavior verified in messaging_ports tests; here ensure type builds
        let _ = std::any::type_name::<SendService>();
    }
}
