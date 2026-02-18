//! HTMX Partial Handlers
//!
//! Handlers that return HTML fragments for HTMX dynamic updates.

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    Form,
};
use serde::Deserialize;
use std::sync::Arc;
use std::time::SystemTime;

use crate::services::whatsapp::WhatsAppService;

// =============================================================================
// Template Definitions for Partials
// =============================================================================

#[derive(Template)]
#[template(path = "partials/health_cards.html")]
pub struct HealthCardsTemplate {
    pub health: HealthData,
    pub uptime_formatted: String,
    pub browser_healthy: bool,
    pub whatsapp_connected: bool,
}

#[derive(Clone)]
pub struct HealthData {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Template)]
#[template(path = "partials/auth_panel.html")]
pub struct AuthPanelTemplate {
    pub status: String,
    pub authenticated: bool,
    pub phone_number: Option<String>,
}

#[derive(Template)]
#[template(path = "partials/qr_code.html")]
pub struct QrCodeTemplate {
    pub qrcode: Option<String>,
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "partials/auth_indicator.html")]
pub struct AuthIndicatorTemplate {
    pub authenticated: bool,
    pub status: String,
}

#[derive(Template)]
#[template(path = "partials/chat_list.html")]
pub struct ChatListTemplate {
    pub chats: Vec<ChatInfo>,
}

#[derive(Clone)]
pub struct ChatInfo {
    pub id: String,
    pub name: String,
    pub is_group: bool,
    pub last_message: Option<String>,
    pub last_message_time: Option<String>,
    pub unread_count: u32,
}

#[derive(Template)]
#[template(path = "partials/chat_view.html")]
pub struct ChatViewTemplate {
    pub chat: ChatInfo,
    pub messages: Vec<MessageInfo>,
}

#[derive(Clone)]
pub struct MessageInfo {
    pub id: String,
    pub text: String,
    pub sender_name: String,
    pub time: String,
    pub is_outgoing: bool,
    pub is_read: bool,
}

// =============================================================================
// Partial Handlers
// =============================================================================

/// Health cards partial (for dashboard)
pub async fn health_cards(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> impl IntoResponse {
    let _now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Get uptime (simplified - using app start time)
    static START_TIME: std::sync::OnceLock<SystemTime> = std::sync::OnceLock::new();
    let start = *START_TIME.get_or_init(SystemTime::now);
    let uptime_seconds = SystemTime::now()
        .duration_since(start)
        .unwrap_or_default()
        .as_secs();

    // Check services
    let browser_healthy = whatsapp_service.health_check().await.is_ok();
    
    let whatsapp_connected = match whatsapp_service.get_auth_status().await {
        Ok(status) => status.authenticated,
        Err(_) => false,
    };

    let uptime_formatted = format_uptime(uptime_seconds);

    let template = HealthCardsTemplate {
        health: HealthData {
            status: if browser_healthy { "healthy".to_string() } else { "unhealthy".to_string() },
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds,
        },
        uptime_formatted,
        browser_healthy,
        whatsapp_connected,
    };

    Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
}

/// Auth panel partial
pub async fn auth_panel(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> impl IntoResponse {
    match whatsapp_service.get_auth_status().await {
        Ok(status) => {
            let template = AuthPanelTemplate {
                status: status.status.clone(),
                authenticated: status.authenticated,
                phone_number: status.phone_number,
            };
            Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
        }
        Err(_) => {
            let template = AuthPanelTemplate {
                status: "checking".to_string(),
                authenticated: false,
                phone_number: None,
            };
            Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
        }
    }
}

/// QR code partial
pub async fn qr_code(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> impl IntoResponse {
    match whatsapp_service
        .execute_with_busy_flag(async { 
            whatsapp_service.auth_service().get_auth_qr_code().await 
        })
        .await
    {
        Ok(qr) => {
            let template = QrCodeTemplate {
                qrcode: Some(qr),
                error: None,
            };
            Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
        }
        Err(e) => {
            let template = QrCodeTemplate {
                qrcode: None,
                error: Some(e.to_string()),
            };
            Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
        }
    }
}

/// Auth indicator partial (header status)
pub async fn auth_indicator(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> impl IntoResponse {
    match whatsapp_service.get_auth_status().await {
        Ok(status) => {
            let template = AuthIndicatorTemplate {
                authenticated: status.authenticated,
                status: status.status,
            };
            Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
        }
        Err(_) => {
            let template = AuthIndicatorTemplate {
                authenticated: false,
                status: "checking".to_string(),
            };
            Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
        }
    }
}

/// Phone pairing partial
#[derive(Deserialize)]
pub struct PhonePairForm {
    phone: String,
}

pub async fn phone_pair(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
    Form(form): Form<PhonePairForm>,
) -> impl IntoResponse {
    match whatsapp_service
        .execute_with_busy_flag(async {
            whatsapp_service
                .auth_service()
                .login_with_phone_number(&form.phone)
                .await
        })
        .await
    {
        Ok(code) => {
            let formatted_code = code.map(|c| c.replace(",", "")).unwrap_or_default();
            Html(format!(
                r#"<div class="p-4 bg-success bg-opacity-10 rounded text-center">
                    <p class="text-secondary small mb-2">Enter this code on your phone:</p>
                    <p class="fs-3 font-monospace fw-bold" style="color: var(--was-green); letter-spacing: 0.1em;">{}</p>
                </div>"#,
                formatted_code
            ))
        }
        Err(e) => {
            Html(format!(
                r#"<div class="p-4 bg-danger bg-opacity-10 rounded text-center">
                    <p class="small text-danger">{}</p>
                </div>"#,
                e
            ))
        }
    }
}

/// Chat list partial
#[derive(Deserialize, Default)]
pub struct ChatListQuery {
    search: Option<String>,
}

pub async fn chat_list(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
    Query(query): Query<ChatListQuery>,
) -> impl IntoResponse {
    match whatsapp_service.chat_service().get_chat_list().await {
        Ok(chat_list) => {
            let mut chats: Vec<ChatInfo> = chat_list.into_iter().map(|c| {
                ChatInfo {
                    id: c.id,
                    name: c.name,
                    is_group: c.is_group,
                    last_message: c.last_message,
                    last_message_time: c.timestamp,
                    unread_count: c.unread_count,
                }
            }).collect();

            // Filter by search if provided
            if let Some(search) = &query.search {
                let search_lower = search.to_lowercase();
                chats.retain(|c| c.name.to_lowercase().contains(&search_lower));
            }

            let template = ChatListTemplate { chats };
            Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
        }
        Err(e) => {
            Html(format!(
                r#"<div class="d-flex flex-column align-items-center justify-content-center text-secondary" style="height: 16rem;">
                    <i class="bi bi-exclamation-circle fs-1 mb-3 opacity-50"></i>
                    <p class="small">Error loading chats: {}</p>
                </div>"#,
                e
            ))
        }
    }
}

/// Chat view partial
pub async fn chat_view(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
    Path(chat_id): Path<String>,
) -> impl IntoResponse {
    // Get chat messages
    match whatsapp_service.chat_service().get_messages(&chat_id, None, false).await {
        Ok(message_list) => {
            let messages: Vec<MessageInfo> = message_list.messages.into_iter().map(|m| {
                MessageInfo {
                    id: m.id,
                    text: m.text.unwrap_or_default(),
                    sender_name: m.sender.unwrap_or_else(|| "Unknown".to_string()),
                    time: m.timestamp.unwrap_or_else(|| "--:--".to_string()),
                    is_outgoing: m.from_me,
                    is_read: m.status.as_deref() == Some("read"),
                }
            }).collect();

            let chat = ChatInfo {
                id: chat_id.clone(),
                name: message_list.chat_name.unwrap_or(chat_id.clone()),
                is_group: false, // TODO: detect from chat info
                last_message: None,
                last_message_time: None,
                unread_count: 0,
            };

            let template = ChatViewTemplate { chat, messages };
            Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
        }
        Err(e) => {
            Html(format!(
                r#"<div class="d-flex flex-column align-items-center justify-content-center h-100 text-secondary">
                    <i class="bi bi-exclamation-circle fs-1 mb-3 opacity-50"></i>
                    <p class="small">Error loading messages: {}</p>
                </div>"#,
                e
            ))
        }
    }
}

/// Link device card partial (shown when not authenticated)
pub async fn link_device_card(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> impl IntoResponse {
    let authenticated = match whatsapp_service.get_auth_status().await {
        Ok(status) => status.authenticated,
        Err(_) => false,
    };

    if authenticated {
        Html("".to_string())
    } else {
        Html(r#"
        <div id="link-device-card" class="col-md-6">
            <div class="action-card h-100" style="border-style: dashed !important;">
                <h5><i class="bi bi-phone text-warning"></i> Link Your Device</h5>
                <p>Scan the QR code or enter your phone number to connect WhatsApp</p>
                <a href="/auth" class="btn btn-was">Link Device</a>
            </div>
        </div>
        "#.to_string())
    }
}

/// Connected account card partial (shown when authenticated)
pub async fn connected_account(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> impl IntoResponse {
    match whatsapp_service.get_auth_status().await {
        Ok(status) if status.authenticated => {
            let phone = status.phone_number.unwrap_or_else(|| "WhatsApp User".to_string());
            Html(format!(r#"
            <div id="connected-account" class="action-card" style="border-color: var(--was-green); background: rgba(37, 211, 102, 0.05);">
                <h5 style="color: var(--was-green-dark);">Connected Account</h5>
                <div class="d-flex align-items-center gap-3 mt-3">
                    <div class="rounded-circle d-flex align-items-center justify-content-center" style="width: 64px; height: 64px; background: rgba(37, 211, 102, 0.2);">
                        <i class="bi bi-phone fs-3" style="color: var(--was-green);"></i>
                    </div>
                    <div>
                        <p class="fw-semibold fs-5 mb-0">{}</p>
                        <p class="text-secondary mb-0">Device linked and ready</p>
                    </div>
                </div>
            </div>
            "#, phone))
        }
        _ => Html(r#"<div id="connected-account"></div>"#.to_string()),
    }
}

/// Server info partial
pub async fn server_info() -> impl IntoResponse {
    Html(format!(r#"
    <div>
        <div class="d-flex justify-content-between mb-2">
            <span class="text-secondary">Version</span>
            <span class="font-monospace small">{}</span>
        </div>
        <div class="d-flex justify-content-between">
            <span class="text-secondary">Build</span>
            <span class="font-monospace small">Release</span>
        </div>
    </div>
    "#, 
    env!("CARGO_PKG_VERSION")
    ))
}

/// Session controls partial
pub async fn session_controls(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
) -> impl IntoResponse {
    let authenticated = match whatsapp_service.get_auth_status().await {
        Ok(status) => status.authenticated,
        Err(_) => false,
    };

    if authenticated {
        Html(r#"
        <div class="d-flex align-items-center gap-2" style="color: var(--was-green);">
            <i class="bi bi-check-circle-fill"></i>
            <span>Device Connected</span>
        </div>
        "#.to_string())
    } else {
        Html(r#"
        <div class="d-flex align-items-center gap-3">
            <div class="d-flex align-items-center gap-2 text-warning">
                <i class="bi bi-exclamation-circle-fill"></i>
                <span>Not Connected</span>
            </div>
            <a href="/auth" class="btn btn-was btn-sm">Connect Device</a>
        </div>
        "#.to_string())
    }
}

// =============================================================================
// Helpers
// =============================================================================

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    
    if parts.is_empty() {
        "< 1m".to_string()
    } else {
        parts.join(" ")
    }
}
