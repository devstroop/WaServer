//! Global send console — instance selector + phone/text/file via SendService (#59)

use askama::Template;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    application::messaging::send::SendMessageCommand,
    domain::{messaging::MediaType, shared::error::DomainError},
};

use super::pages::{AppState, WebSessionExt, VERSION};

#[derive(Template)]
#[template(path = "web/pages/send.html")]
pub struct SendPageTemplate {
    pub title: String,
    pub version: &'static str,
    pub csrf: String,
    pub username: String,
    pub role: String,
    pub admin: bool,
    pub instances: Vec<SendInstanceOption>,
}

pub struct SendInstanceOption {
    pub id: String,
    pub label: String,
    pub status: &'static str,
    pub badge_class: &'static str,
}

/// `GET /app/send`
pub async fn page(State(app): State<AppState>, session: WebSessionExt) -> Response {
    let snapshots = app.manager.list_status_snapshots().await;
    let instances = snapshots
        .iter()
        .map(|s| {
            let (status, badge_class) = match s.status {
                crate::models::instance::InstanceStatus::Active => ("active", "dt-badge--success"),
                crate::models::instance::InstanceStatus::WarmingUp => {
                    ("warming up", "dt-badge--warning")
                }
                crate::models::instance::InstanceStatus::Sleeping => {
                    ("sleeping", "dt-badge--neutral")
                }
                crate::models::instance::InstanceStatus::Error(_) => ("error", "dt-badge--danger"),
            };
            SendInstanceOption {
                id: s.id.to_string(),
                label: s
                    .name
                    .clone()
                    .unwrap_or_else(|| s.phone.clone().unwrap_or_else(|| s.id.to_string())),
                status,
                badge_class,
            }
        })
        .collect();

    let tpl = SendPageTemplate {
        title: "Send — WAS Admin".into(),
        version: VERSION,
        csrf: session.csrf_token.clone(),
        username: session.username.clone(),
        role: session.role.clone(),
        admin: session.admin,
        instances,
    };
    (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response()
}

#[derive(Template)]
#[template(path = "web/_send_feedback.html")]
pub struct SendFeedbackTemplate {
    pub tone: &'static str,
    pub message: String,
}

fn send_error_fragment(err: &DomainError) -> Response {
    let (tone, message) = match err {
        DomainError::RateLimited {
            retry_after_seconds,
            ..
        } => (
            "warning",
            format!("Rate limited — try again in {retry_after_seconds}s."),
        ),
        DomainError::NotFound { .. } => ("danger", "Instance not found.".to_string()),
        DomainError::Validation(msg) | DomainError::InvalidInput { reason: msg, .. } => {
            ("danger", msg.clone())
        }
        DomainError::PermissionDenied { .. } => {
            ("danger", "Not authorized for this instance.".to_string())
        }
        other => ("danger", other.to_string()),
    };
    let tpl = SendFeedbackTemplate { tone, message };
    (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response()
}

/// `POST /app/send` — multipart with `instance_id`, `phone`, `text`, `file`
pub async fn send_post(State(app): State<AppState>, mut multipart: Multipart) -> Response {
    let mut instance_id = String::new();
    let mut phone = String::new();
    let mut text: Option<String> = None;
    let mut media_path: Option<String> = None;
    let mut media_type = MediaType::None;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("instance_id") => {
                instance_id = field.text().await.unwrap_or_default().trim().to_string();
            }
            Some("phone") => {
                phone = field.text().await.unwrap_or_default().trim().to_string();
            }
            Some("text") => match field.text().await {
                Ok(t) if !t.trim().is_empty() => text = Some(t),
                _ => {}
            },
            Some("file") => {
                let has_content = field.file_name().map(|f| !f.is_empty()).unwrap_or(false);
                if has_content {
                    match super::super::handlers::messaging::stage_upload(field).await {
                        Ok((path, len)) => {
                            tracing::info!(path = %path, bytes = len, "global send attachment staged");
                            media_type =
                                super::super::handlers::messaging::media_type_for_filename(&path);
                            media_path = Some(path);
                        }
                        Err(e) => {
                            return send_error_fragment(&DomainError::Internal(e));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if instance_id.is_empty() {
        return send_error_fragment(&DomainError::Validation("Select an instance.".into()));
    }
    if text.is_none() && media_path.is_none() {
        return send_error_fragment(&DomainError::Validation(
            "Either a message or a file attachment is required.".into(),
        ));
    }
    if app.manager.get_instance(&instance_id).await.is_none() {
        return send_error_fragment(&DomainError::not_found("instance", &instance_id));
    }

    let service = super::super::handlers::messaging::build_send_service(app.manager.clone());
    let cmd = SendMessageCommand {
        instance: match uuid::Uuid::parse_str(&instance_id) {
            Ok(u) => u,
            Err(_) => {
                return send_error_fragment(&DomainError::not_found("instance", &instance_id))
            }
        },
        to: phone,
        text,
        media_type,
        media_path: media_path.clone(),
    };

    match service.send(cmd).await {
        Ok(message_id) => {
            if let Some(path) = &media_path {
                let _ = tokio::fs::remove_file(path).await;
            }
            let tpl = SendFeedbackTemplate {
                tone: "success",
                message: format!("Sent ✓ (message {message_id})"),
            };
            (
                StatusCode::OK,
                axum::response::Html(tpl.render().unwrap_or_default()),
            )
                .into_response()
        }
        Err(e) => send_error_fragment(&e),
    }
}
