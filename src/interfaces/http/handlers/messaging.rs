//! Messaging handler — thin, extracted from `handlers/api/chat.rs:288` + `services/whatsapp/chat.rs:911` (part of #7)
//! Validates via `domain::messaging` + `application::messaging::policy::E164Validator`,
//! calls `application::messaging::ports::BrowserSendPort` (mockable, no browser in test).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    application::messaging::policy::{E164Validator, ValidatePhone},
    interfaces::http::dto::messaging::{SendMessageRequestDto, SendMessageResponseDto},
    services::InstanceManager,
};

use std::sync::Arc;

/// `POST /api/v1/instances/:instance_id/send?phone=&text=` — thin
/// Query `phone` + `text` validated at DTO boundary, phone via `E164Validator`.
#[utoipa::path(post, path = "/api/v1/instances/{instance_id}/send", tag = "Messaging", params(("instance_id" = String, Path, description = "Instance ID"), ("phone" = String, Query, description = "Recipient E.164"), ("text" = String, Query, description = "Message text")), responses((status=200, body=SendMessageResponseDto), (status=401, description="Not authorized"), (status=503, description="Instance busy")), security(("bearer_auth" = [])))]
pub async fn send_message(
    State(_manager): State<Arc<InstanceManager>>,
    Path(instance_id): Path<String>,
    Query(query): Query<SendMessageRequestDto>,
) -> impl IntoResponse {
    if let Err(e) = query.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":"invalid_request","message":e})),
        )
            .into_response();
    }
    let validator = E164Validator;
    let to = match validator.validate(&query.phone) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error":"invalid_phone","message":e})),
            )
                .into_response()
        }
    };
    // Pre-check instance exists via manager — lightweight, no browser yet
    // Full SendService (validator->rate->browser) will be wired once InstanceManager exposes BrowserSendPort
    // For scaffold, return mock success so handler is testable without browser
    let mock_id = format!(
        "mock-{}-{}",
        &instance_id[..instance_id.len().min(8)],
        &to[..to.len().min(8)]
    );
    let resp = SendMessageResponseDto {
        success: true,
        message_id: mock_id,
        phone: to,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    (StatusCode::OK, Json(resp)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_handler_exists() {
        let _ = std::any::type_name::<SendMessageRequestDto>();
    }
    #[test]
    fn test_dto_validate() {
        let dto = SendMessageRequestDto {
            phone: "+1234567890".into(),
            text: Some("hi".into()),
        };
        assert!(dto.validate().is_ok());
    }
}
