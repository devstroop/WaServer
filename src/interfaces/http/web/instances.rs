//! Instances management — list, create, detail, link (QR), config (#31)
//!
//! Web routes are thin adapters over the same `InstanceManager` flows as the
//! JSON API; all mutations ride CSRF-protected form posts.

use askama::Template;
use axum::{
    extract::{Form, Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use base64::Engine;
use serde::Deserialize;

use crate::{
    application::messaging::send::SendMessageCommand,
    domain::{
        instance::{CreateInstanceRequest, UpdateInstanceConfigRequest},
        messaging::MediaType,
        shared::error::DomainError,
    },
    models::instance::{validate_phone_number, InstanceStatus},
    services::StatusSnapshot,
};

use super::pages::{error_fragment, VERSION};

fn badge(status: &InstanceStatus) -> (&'static str, &'static str) {
    match status {
        InstanceStatus::Active => ("active", "dt-badge--success"),
        InstanceStatus::WarmingUp => ("warming up", "dt-badge--warning"),
        InstanceStatus::Sleeping => ("sleeping", "dt-badge--neutral"),
        InstanceStatus::Error(_) => ("error", "dt-badge--danger"),
    }
}

// ---------------------------------------------------------------------------
// List page + polled table
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "web/pages/instances.html")]
pub struct InstancesPageTemplate {
    pub title: String,
    pub version: &'static str,
    pub csrf: String,
    pub username: String,
    pub role: String,
    pub admin: bool,
}

/// `GET /app/instances`
pub async fn list_page(
    super::pages::WebSessionExt(session): super::pages::WebSessionExt,
) -> Response {
    let tpl = InstancesPageTemplate {
        title: "Instances — WAS Admin".into(),
        version: VERSION,
        csrf: session.csrf_token,
        username: session.username.clone(),
        role: session.role.clone(),
        admin: session.admin,
    };
    html_page(tpl)
}

#[derive(Template)]
#[template(path = "web/_instance_table.html")]
pub struct InstanceTableTemplate {
    pub instances: Vec<InstanceRow>,
}

pub struct InstanceRow {
    pub id: String,
    pub label: String,
    pub phone: String,
    pub status: &'static str,
    pub badge_class: &'static str,
}

/// `GET /app/fragments/instances` — polled table fragment
pub async fn table_fragment(State(app): State<super::pages::AppState>) -> Response {
    let snapshots = app.manager.list_status_snapshots().await;
    let tpl = InstanceTableTemplate {
        instances: snapshots.iter().map(row_from_snapshot).collect(),
    };
    (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response()
}

fn row_from_snapshot(s: &StatusSnapshot) -> InstanceRow {
    let (status, badge_class) = badge(&s.status);
    InstanceRow {
        id: s.id.to_string(),
        label: s.name.clone().unwrap_or_else(|| s.id.to_string()),
        phone: s.phone.clone().unwrap_or_else(|| "—".into()),
        status,
        badge_class,
    }
}

// ---------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateInstanceForm {
    pub phone_number: String,
    #[serde(default)]
    pub instance_name: Option<String>,
    #[serde(default)]
    pub idle_timeout: Option<String>,
}

#[derive(Template)]
#[template(path = "web/_toast.html")]
pub struct ToastFragment {
    pub tone: &'static str,
    pub message: String,
}

fn toast(tone: &'static str, message: &str) -> Response {
    let tpl = ToastFragment {
        tone,
        message: message.to_string(),
    };
    (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response()
}

/// `POST /app/instances` — create from dialog form; toast + refresh list on success
pub async fn create_post(
    State(app): State<super::pages::AppState>,
    Form(form): Form<CreateInstanceForm>,
) -> Response {
    let request = CreateInstanceRequest {
        phone_number: form.phone_number.trim().to_string(),
        instance_name: form
            .instance_name
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "unknown".to_string()),
        browser: None,
        idle_timeout: form
            .idle_timeout
            .as_deref()
            .and_then(|s| s.trim().parse::<u64>().ok()),
    };
    // Validate phone shape early for a friendly message (manager re-validates)
    if validate_phone_number(&request.phone_number).is_err() {
        return (
            StatusCode::BAD_REQUEST,
            axum::response::Html(error_fragment("Phone must be E.164 (e.g. +15551234567).")),
        )
            .into_response();
    }
    match app.manager.create_instance(request).await {
        Ok(resp) => {
            tracing::info!(id = %resp.id, "web created instance");
            toast("success", "Instance created.")
        }
        Err(e) => {
            let msg = e.to_string();
            let text = if msg.contains("already exists") {
                "A phone number can only be registered once."
            } else if msg.contains("Invalid") {
                "Phone must be E.164 (e.g. +15551234567)."
            } else {
                &msg
            };
            (
                StatusCode::BAD_REQUEST,
                axum::response::Html(error_fragment(text)),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// Detail page + status fragment
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "web/pages/instance_detail.html")]
pub struct InstanceDetailTemplate {
    pub title: String,
    pub version: &'static str,
    pub csrf: String,
    pub username: String,
    pub role: String,
    pub admin: bool,
    pub id: String,
    pub label: String,
    pub phone: String,
    pub status: &'static str,
    pub badge_class: &'static str,
    pub authorized: bool,
    pub messages_per_minute: u32,
    pub idle_timeout: u64,
}

/// `GET /app/instances/:id`
pub async fn detail_page(
    State(app): State<super::pages::AppState>,
    super::pages::WebSessionExt(session): super::pages::WebSessionExt,
    Path(id): Path<String>,
) -> Response {
    let Some(snap) = find_snapshot(&app, &id).await else {
        return not_found_page(&session);
    };
    let (status, badge_class) = badge(&snap.status);
    let config = app.manager.registry.get_config(snap.id).await;
    let tpl = InstanceDetailTemplate {
        title: format!(
            "{} — WAS Admin",
            snap.name.clone().unwrap_or_else(|| snap.id.to_string())
        ),
        version: VERSION,
        csrf: session.csrf_token.clone(),
        username: session.username.clone(),
        role: session.role.clone(),
        admin: session.admin,
        id: snap.id.to_string(),
        label: snap.name.clone().unwrap_or_else(|| snap.id.to_string()),
        phone: snap.phone.clone().unwrap_or_else(|| "—".into()),
        status,
        badge_class,
        authorized: matches!(snap.status, InstanceStatus::Active),
        messages_per_minute: config
            .as_ref()
            .map(|c| c.rate_limits.messages_per_minute)
            .unwrap_or(60),
        idle_timeout: config.as_ref().map(|c| c.idle_timeout).unwrap_or(300),
    };
    html_page(tpl)
}

#[derive(Template)]
#[template(path = "web/_instance_status.html")]
pub struct StatusFragmentTemplate {
    pub id: String,
    pub label: String,
    pub phone: String,
    pub status: &'static str,
    pub badge_class: &'static str,
}

/// `GET /app/fragments/instances/:id/status` — polled badge on the detail page
pub async fn status_fragment(
    State(app): State<super::pages::AppState>,
    Path(id): Path<String>,
) -> Response {
    match find_snapshot(&app, &id).await {
        Some(snap) => {
            let (status, badge_class) = badge(&snap.status);
            let tpl = StatusFragmentTemplate {
                id: snap.id.to_string(),
                label: snap.name.unwrap_or_else(|| snap.id.to_string()),
                phone: snap.phone.unwrap_or_else(|| "—".into()),
                status,
                badge_class,
            };
            (
                StatusCode::OK,
                axum::response::Html(tpl.render().unwrap_or_default()),
            )
                .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            axum::response::Html(error_fragment("Instance not found.")),
        )
            .into_response(),
    }
}

async fn find_snapshot(app: &super::pages::AppState, id: &str) -> Option<StatusSnapshot> {
    let uuid = uuid::Uuid::parse_str(id).ok()?;
    app.manager
        .list_status_snapshots()
        .await
        .into_iter()
        .find(|s| s.id == uuid)
}

fn not_found_page(session: &super::pages::SessionView) -> Response {
    #[derive(Template)]
    #[template(path = "web/pages/instances.html")]
    struct NotFoundPage {
        title: String,
        version: &'static str,
        csrf: String,
        username: String,
        role: String,
        admin: bool,
    }
    html_page(NotFoundPage {
        title: "Not found — WAS Admin".into(),
        version: VERSION,
        csrf: session.csrf_token.clone(),
        username: session.username.clone(),
        role: session.role.clone(),
        admin: session.admin,
    })
}

// ---------------------------------------------------------------------------
// Link (QR) flow — polled panel; self-swaps to linked state
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "web/_link_panel.html")]
pub struct LinkPanelTemplate {
    pub id: String,
    pub linked: bool,
    pub status: &'static str,
}

/// `GET /app/fragments/instances/:id/link` — poll every 2s while unlinked.
/// When linked, renders a static card without hx attributes (polling stops).
pub async fn link_fragment(
    State(app): State<super::pages::AppState>,
    Path(id): Path<String>,
) -> Response {
    let account = app.manager.get_instance(&id).await;
    let Some(account) = account else {
        return (
            StatusCode::NOT_FOUND,
            axum::response::Html(error_fragment("Instance not found.")),
        )
            .into_response();
    };

    // Authorized check requires a running browser (same as API QR endpoint).
    // ensure_warm is a no-op when already warm.
    let linked = match account.ensure_warm().await {
        Ok(()) => matches!(account.info().await.status, InstanceStatus::Active),
        Err(_) => false,
    };

    let tpl = LinkPanelTemplate {
        id: account.id.to_string(),
        linked,
        status: if linked { "linked" } else { "waiting for scan" },
    };
    (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response()
}

/// `GET /app/fragments/instances/:id/qr.png` — proxied QR image bytes
pub async fn qr_png(State(app): State<super::pages::AppState>, Path(id): Path<String>) -> Response {
    let Some(account) = app.manager.get_instance(&id).await else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    if let Err(e) = account.ensure_warm().await {
        return (StatusCode::SERVICE_UNAVAILABLE, e.to_string()).into_response();
    }
    use axum::body::Body;
    match account.auth_service().get_auth_qr_code().await {
        Ok(qr_base64) => match base64::engine::general_purpose::STANDARD.decode(&qr_base64) {
            Ok(png_bytes) => (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "image/png".to_string()),
                    (header::CACHE_CONTROL, "no-store".to_string()),
                ],
                Body::from(png_bytes),
            )
                .into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "bad QR payload").into_response(),
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Actions — warmup / reset / delete / screenshot / config
// ---------------------------------------------------------------------------

/// `POST /app/instances/:id/warmup`
pub async fn warmup_post(
    State(app): State<super::pages::AppState>,
    Path(id): Path<String>,
) -> Response {
    match app.manager.get_instance(&id).await {
        Some(account) => match account.warmup().await {
            Ok(()) => toast("success", "Warmup complete."),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("already warming up") {
                    toast("info", "Already warming up.")
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::response::Html(error_fragment(&format!("Warmup failed: {msg}"))),
                    )
                        .into_response()
                }
            }
        },
        None => (
            StatusCode::NOT_FOUND,
            axum::response::Html(error_fragment("Instance not found.")),
        )
            .into_response(),
    }
}

/// `POST /app/instances/:id/reset` — unlink WhatsApp session data
pub async fn reset_post(
    State(app): State<super::pages::AppState>,
    Path(id): Path<String>,
) -> Response {
    match app.manager.reset_account(&id).await {
        Ok(()) => toast("success", "Instance reset. Re-link required."),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::response::Html(error_fragment(&format!("Reset failed: {e}"))),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct DeleteForm {
    #[serde(default)]
    pub delete_data: Option<String>,
}

/// `POST /app/instances/:id/delete` — then redirect back to the list
pub async fn delete_post(
    State(app): State<super::pages::AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Form(form): Form<DeleteForm>,
) -> Response {
    let delete_data = form.delete_data.is_some();
    match app.manager.delete_instance(&id, delete_data).await {
        Ok(_) => {
            tracing::info!(%id, "web deleted instance");
            super::session::redirect(&headers, "/app/instances")
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            axum::response::Html(error_fragment(&format!("Delete failed: {e}"))),
        )
            .into_response(),
    }
}

/// `GET /app/instances/:id/shot` — small fragment embedding the live screenshot.
/// The img src busts cache with a timestamp so repeat clicks refresh the image.
pub async fn shot_fragment(
    State(app): State<super::pages::AppState>,
    Path(id): Path<String>,
) -> Response {
    // 404 fast when missing
    if app.manager.get_instance(&id).await.is_none() {
        return (
            StatusCode::NOT_FOUND,
            axum::response::Html(error_fragment("Instance not found.")),
        )
            .into_response();
    }
    let ts = chrono::Utc::now().timestamp_millis();
    let html = format!(
        r#"<img src="/app/fragments/instances/{id}/screenshot.png?ts={ts}" alt="Screenshot of {id}" style="max-width:100%; border-radius:8px;" loading="lazy">"#
    );
    (StatusCode::OK, axum::response::Html(html)).into_response()
}

/// `GET /app/fragments/instances/:id/screenshot.png` — proxied screenshot bytes
pub async fn screenshot_png(
    State(app): State<super::pages::AppState>,
    Path(id): Path<String>,
) -> Response {
    let Some(account) = app.manager.get_instance(&id).await else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    if let Err(e) = account.ensure_warm().await {
        return (StatusCode::SERVICE_UNAVAILABLE, e.to_string()).into_response();
    }
    use axum::body::Body;
    match account.browser_service().screenshot().await {
        Ok(png_data) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "image/png".to_string()),
                (header::CACHE_CONTROL, "no-store".to_string()),
            ],
            Body::from(png_data),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
pub struct ConfigForm {
    pub messages_per_minute: Option<String>,
    pub idle_timeout: Option<String>,
}

/// `POST /app/instances/:id/config` — partial config update from the form
pub async fn config_post(
    State(app): State<super::pages::AppState>,
    Path(id): Path<String>,
    Form(form): Form<ConfigForm>,
) -> Response {
    let Some(account) = app.manager.get_instance(&id).await else {
        return (
            StatusCode::NOT_FOUND,
            axum::response::Html(error_fragment("Instance not found.")),
        )
            .into_response();
    };

    let mpm = form
        .messages_per_minute
        .as_deref()
        .and_then(|s| s.trim().parse::<u32>().ok());
    let idle = form
        .idle_timeout
        .as_deref()
        .and_then(|s| s.trim().parse::<u64>().ok());
    if mpm.is_none() && idle.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            axum::response::Html(error_fragment("Nothing to update.")),
        )
            .into_response();
    }

    let update = UpdateInstanceConfigRequest {
        instance_name: None,
        idle_timeout: idle,
        browser: None,
        rate_limits: mpm.map(|m| crate::domain::instance::UpdateRateLimits {
            messages_per_minute: Some(m),
            requests_per_minute: None,
            message_cooldown_ms: None,
        }),
    };

    match account.update_config_typed(update).await {
        Ok((_, restart_required)) => {
            let note = if restart_required {
                " Restart the instance for some changes to apply."
            } else {
                ""
            };
            toast("success", &format!("Configuration saved.{note}"))
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            axum::response::Html(error_fragment(&format!("Config error: {e}"))),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------------------
// Messaging console — send text/media through SendService (#32)
// ---------------------------------------------------------------------------

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

/// `POST /app/instances/:id/send` — multipart form: phone, text, optional file.
/// Delegates to the same SendService flow as the JSON API
/// (validator → policy → rate limit → browser port).
pub async fn send_post(
    State(app): State<super::pages::AppState>,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Response {
    // Resolve instance first so 404s are cheap
    if app.manager.get_instance(&id).await.is_none() {
        return send_error_fragment(&DomainError::not_found("instance", &id));
    }

    let mut phone = String::new();
    let mut text: Option<String> = None;
    let mut media_path: Option<String> = None;
    let mut media_type = MediaType::None;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
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
                            tracing::info!(path = %path, bytes = len, "web attachment staged");
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

    if text.is_none() && media_path.is_none() {
        return send_error_fragment(&DomainError::Validation(
            "Either a message or a file attachment is required.".into(),
        ));
    }

    let service = super::super::handlers::messaging::build_send_service(app.manager.clone());
    let cmd = SendMessageCommand {
        instance: match uuid::Uuid::parse_str(&id) {
            Ok(u) => u,
            Err(_) => return send_error_fragment(&DomainError::not_found("instance", &id)),
        },
        to: phone,
        text,
        media_type,
        media_path,
    };

    match service.send(cmd).await {
        Ok(message_id) => {
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

fn html_page<T: askama::Template>(tpl: T) -> Response {
    (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response()
}
