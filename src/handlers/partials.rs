//! HTMX Partial Handlers
//!
//! Handlers that return HTML fragments for HTMX dynamic updates.
//! Uses AccountManager for multi-account support - shows first available account by default.
//!
//! - Debug: Uses minijinja for hot-reloading templates from disk
//! - Release: Uses pre-compiled askama templates for performance

use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    Form,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;

use crate::services::whatsapp::AccountManager;
use crate::models::account::AccountStatus;

// =============================================================================
// Release Mode: Compiled Askama Templates
// =============================================================================

#[cfg(not(debug_assertions))]
mod compiled {
    use askama::Template;
    use super::*;

    #[derive(Template)]
    #[template(path = "partials/health_cards.html")]
    pub struct HealthCardsTemplate {
        pub health: HealthData,
        pub uptime_formatted: String,
        pub browser_healthy: bool,
        pub whatsapp_connected: bool,
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

    #[derive(Template)]
    #[template(path = "partials/chat_view.html")]
    pub struct ChatViewTemplate {
        pub chat: ChatInfo,
        pub messages: Vec<MessageInfo>,
    }
}

// =============================================================================
// Data Types (shared between debug and release)
// =============================================================================

#[derive(Clone, Serialize)]
pub struct HealthData {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Clone, Serialize)]
pub struct ChatInfo {
    pub id: String,
    pub name: String,
    pub is_group: bool,
    pub last_message: Option<String>,
    pub last_message_time: Option<String>,
    pub unread_count: u32,
}

#[derive(Clone, Serialize)]
pub struct MessageInfo {
    pub id: String,
    pub text: String,
    pub sender_name: String,
    pub time: String,
    pub is_outgoing: bool,
    pub is_read: bool,
}

// =============================================================================
// Debug Mode: Runtime Template Rendering
// =============================================================================

#[cfg(debug_assertions)]
fn render_partial<T: Serialize>(template_path: &str, ctx: T) -> String {
    use crate::utils::templates::render_template;

    match render_template(template_path, ctx) {
        Ok(html) => html,
        Err(e) => format!(
            r#"<div class="alert alert-danger"><strong>Template Error:</strong> {}</div>"#,
            e
        ),
    }
}

// =============================================================================
// Helper: Get first available account
// =============================================================================

async fn get_first_account(manager: &AccountManager) -> Option<Arc<crate::services::whatsapp::WhatsAppAccount>> {
    // Try to get first running account
    let response = manager.list_accounts().await;
    for info in &response.accounts {
        if matches!(info.status, AccountStatus::Running) {
            if let Some(account) = manager.get_account_by_id(info.id).await {
                return Some(account);
            }
        }
    }
    // Fallback to first account if any
    if let Some(info) = response.accounts.first() {
        return manager.get_account_by_id(info.id).await;
    }
    None
}

// =============================================================================
// Partial Handlers
// =============================================================================

/// Health cards partial (for dashboard)
pub async fn health_cards(
    State(manager): State<Arc<AccountManager>>,
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

    // Check services - use first available account
    let (browser_healthy, whatsapp_connected) = if let Some(account) = get_first_account(&manager).await {
        let healthy = account.health_check().await.is_ok();
        let connected = match account.get_auth_status().await {
            Ok(status) => status.authenticated,
            Err(_) => false,
        };
        (healthy, connected)
    } else {
        // No accounts - show as unhealthy
        (false, false)
    };

    let uptime_formatted = format_uptime(uptime_seconds);
    let health = HealthData {
        status: if browser_healthy { "healthy".to_string() } else { "unhealthy".to_string() },
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds,
    };

    #[cfg(debug_assertions)]
    {
        use serde_json::json;
        Html(render_partial("partials/health_cards.html", json!({
            "health": health,
            "uptime_formatted": uptime_formatted,
            "browser_healthy": browser_healthy,
            "whatsapp_connected": whatsapp_connected,
        })))
    }
    #[cfg(not(debug_assertions))]
    {
        use askama::Template;
        let template = compiled::HealthCardsTemplate {
            health,
            uptime_formatted,
            browser_healthy,
            whatsapp_connected,
        };
        Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
    }
}

/// Auth panel partial
pub async fn auth_panel(
    State(manager): State<Arc<AccountManager>>,
) -> impl IntoResponse {
    let (status, authenticated, phone_number) = if let Some(account) = get_first_account(&manager).await {
        match account.get_auth_status().await {
            Ok(s) => (s.status.clone(), s.authenticated, s.phone_number),
            Err(_) => ("no_account".to_string(), false, None),
        }
    } else {
        ("no_account".to_string(), false, None)
    };
    
    #[cfg(debug_assertions)]
    {
        use serde_json::json;
        Html(render_partial("partials/auth_panel.html", json!({
            "status": status,
            "authenticated": authenticated,
            "phone_number": phone_number,
        })))
    }
    #[cfg(not(debug_assertions))]
    {
        use askama::Template;
        let template = compiled::AuthPanelTemplate {
            status,
            authenticated,
            phone_number,
        };
        Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
    }
}

/// QR code partial
pub async fn qr_code(
    State(manager): State<Arc<AccountManager>>,
) -> impl IntoResponse {
    let (qrcode, error) = if let Some(account) = get_first_account(&manager).await {
        match account
            .execute_with_busy_flag(async { 
                account.auth_service().get_auth_qr_code().await 
            })
            .await
        {
            Ok(qr) => (Some(qr), None),
            Err(e) => (None, Some(e.to_string())),
        }
    } else {
        (None, Some("No account available. Create an account first.".to_string()))
    };

    #[cfg(debug_assertions)]
    {
        use serde_json::json;
        Html(render_partial("partials/qr_code.html", json!({
            "qrcode": qrcode,
            "error": error,
        })))
    }
    #[cfg(not(debug_assertions))]
    {
        use askama::Template;
        let template = compiled::QrCodeTemplate { qrcode, error };
        Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
    }
}

/// Auth indicator partial (header status)
pub async fn auth_indicator(
    State(manager): State<Arc<AccountManager>>,
) -> impl IntoResponse {
    let (authenticated, status) = if let Some(account) = get_first_account(&manager).await {
        match account.get_auth_status().await {
            Ok(s) => (s.authenticated, s.status),
            Err(_) => (false, "no_account".to_string()),
        }
    } else {
        (false, "no_account".to_string())
    };

    #[cfg(debug_assertions)]
    {
        use serde_json::json;
        Html(render_partial("partials/auth_indicator.html", json!({
            "authenticated": authenticated,
            "status": status,
        })))
    }
    #[cfg(not(debug_assertions))]
    {
        use askama::Template;
        let template = compiled::AuthIndicatorTemplate { authenticated, status };
        Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
    }
}

/// Phone pairing partial
#[derive(Deserialize)]
pub struct PhonePairForm {
    phone: String,
}

pub async fn phone_pair(
    State(manager): State<Arc<AccountManager>>,
    Form(form): Form<PhonePairForm>,
) -> impl IntoResponse {
    let Some(account) = get_first_account(&manager).await else {
        return Html(r#"<div class="p-4 bg-danger bg-opacity-10 rounded text-center">
            <p class="small text-danger">No account available. Create an account first.</p>
        </div>"#.to_string());
    };
    
    match account
        .execute_with_busy_flag(async {
            account
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
    State(manager): State<Arc<AccountManager>>,
    Query(query): Query<ChatListQuery>,
) -> impl IntoResponse {
    let Some(account) = get_first_account(&manager).await else {
        return Html(r#"<div class="d-flex flex-column align-items-center justify-content-center text-secondary" style="height: 16rem;">
            <i class="bi bi-exclamation-circle icon-3xl mb-3 opacity-50"></i>
            <p class="text-sm">No account available. Create an account first.</p>
        </div>"#.to_string());
    };
    
    match account.chat_service().get_chat_list().await {
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

            #[cfg(debug_assertions)]
            {
                use serde_json::json;
                Html(render_partial("partials/chat_list.html", json!({ "chats": chats })))
            }
            #[cfg(not(debug_assertions))]
            {
                use askama::Template;
                let template = compiled::ChatListTemplate { chats };
                Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
            }
        }
        Err(e) => {
            Html(format!(
                r#"<div class="d-flex flex-column align-items-center justify-content-center text-secondary" style="height: 16rem;">
                    <i class="bi bi-exclamation-circle icon-3xl mb-3 opacity-50"></i>
                    <p class="text-sm">Error loading chats: {}</p>
                </div>"#,
                e
            ))
        }
    }
}

/// Chat view partial
pub async fn chat_view(
    State(manager): State<Arc<AccountManager>>,
    Path(chat_id): Path<String>,
) -> impl IntoResponse {
    let Some(account) = get_first_account(&manager).await else {
        return Html(r#"<div class="d-flex flex-column align-items-center justify-content-center h-100 text-secondary">
            <i class="bi bi-exclamation-circle icon-3xl mb-3 opacity-50"></i>
            <p class="text-sm">No account available</p>
        </div>"#.to_string());
    };
    
    // Get chat messages
    match account.chat_service().get_messages(&chat_id, None, false).await {
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

            #[cfg(debug_assertions)]
            {
                use serde_json::json;
                Html(render_partial("partials/chat_view.html", json!({
                    "chat": chat,
                    "messages": messages,
                })))
            }
            #[cfg(not(debug_assertions))]
            {
                use askama::Template;
                let template = compiled::ChatViewTemplate { chat, messages };
                Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
            }
        }
        Err(e) => {
            Html(format!(
                r#"<div class="d-flex flex-column align-items-center justify-content-center h-100 text-secondary">
                    <i class="bi bi-exclamation-circle icon-3xl mb-3 opacity-50"></i>
                    <p class="text-sm">Error loading messages: {}</p>
                </div>"#,
                e
            ))
        }
    }
}

/// Link device card partial (shown when not authenticated)
pub async fn link_device_card(
    State(manager): State<Arc<AccountManager>>,
) -> impl IntoResponse {
    let authenticated = if let Some(account) = get_first_account(&manager).await {
        match account.get_auth_status().await {
            Ok(status) => status.authenticated,
            Err(_) => false,
        }
    } else {
        false
    };

    if authenticated {
        Html("".to_string())
    } else {
        Html(r#"
        <div id="link-device-card" class="col-md-6">
            <div class="action-card h-100" style="border-style: dashed !important;">
                <h5><i class="bi bi-phone icon-lg text-warning"></i> Link Your Device</h5>
                <p class="text-sm">Scan the QR code or enter your phone number to connect WhatsApp</p>
                <a href="/auth" class="btn btn-was">Link Device</a>
            </div>
        </div>
        "#.to_string())
    }
}

/// Connected account card partial (shown when authenticated)
pub async fn connected_account(
    State(manager): State<Arc<AccountManager>>,
) -> impl IntoResponse {
    if let Some(account) = get_first_account(&manager).await {
        match account.get_auth_status().await {
            Ok(status) if status.authenticated => {
                let phone = status.phone_number.unwrap_or_else(|| "WhatsApp User".to_string());
                return Html(format!(r#"
                <div id="connected-account" class="action-card" style="border-color: var(--was-green); background: rgba(37, 211, 102, 0.05);">
                    <h5 style="color: var(--was-green-dark);">Connected Account</h5>
                    <div class="d-flex align-items-center gap-3 mt-3">
                        <div class="rounded-circle d-flex align-items-center justify-content-center" style="width: 48px; height: 48px; background: rgba(37, 211, 102, 0.2);">
                            <i class="bi bi-phone icon-xl" style="color: var(--was-green);"></i>
                        </div>
                        <div>
                            <p class="font-semibold text-base mb-0">{}</p>
                            <p class="text-sm text-muted mb-0">Device linked and ready</p>
                        </div>
                    </div>
                </div>
                "#, phone));
            }
            _ => {}
        }
    }
    
    Html(r#"<div id="connected-account"></div>"#.to_string())
}

/// Server info partial
pub async fn server_info() -> impl IntoResponse {
    Html(format!(r#"
    <div>
        <div class="d-flex justify-content-between mb-2">
            <span class="text-muted">Version</span>
            <span class="font-mono text-sm">{}</span>
        </div>
        <div class="d-flex justify-content-between">
            <span class="text-muted">Build</span>
            <span class="font-mono text-sm">Release</span>
        </div>
    </div>
    "#, 
    env!("CARGO_PKG_VERSION")
    ))
}

/// Session controls partial
pub async fn session_controls(
    State(manager): State<Arc<AccountManager>>,
) -> impl IntoResponse {
    let authenticated = if let Some(account) = get_first_account(&manager).await {
        match account.get_auth_status().await {
            Ok(status) => status.authenticated,
            Err(_) => false,
        }
    } else {
        false
    };

    if authenticated {
        Html(r#"
        <div class="d-flex align-items-center gap-2" style="color: var(--was-green);">
            <i class="bi bi-check-circle-fill icon-md"></i>
            <span class="text-sm">Device Connected</span>
        </div>
        "#.to_string())
    } else {
        Html(r#"
        <div class="d-flex align-items-center gap-3">
            <div class="d-flex align-items-center gap-2 text-warning">
                <i class="bi bi-exclamation-circle-fill icon-md"></i>
                <span class="text-sm">Not Connected</span>
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

// =============================================================================
// Unlink Partial (for web UI)
// =============================================================================

/// Unlink WhatsApp account - called from settings page
/// Uses first available account (for web UI that doesn't have account selection yet)
pub async fn unlink_account(
    State(manager): State<Arc<AccountManager>>,
) -> impl IntoResponse {
    let Some(account) = get_first_account(&manager).await else {
        return Html(r#"
        <div class="alert alert-danger">
            <i class="bi bi-exclamation-triangle alert-icon"></i>
            <span class="alert-title">No Account</span>
            <span class="alert-description">No account found to unlink.</span>
        </div>
        "#.to_string());
    };

    match account.auth_service().logout().await {
        Ok(_) => {
            account.invalidate_auth_cache().await;
            Html(r#"
            <div class="d-flex align-items-center gap-3">
                <div class="d-flex align-items-center gap-2 text-warning">
                    <i class="bi bi-exclamation-circle-fill icon-md"></i>
                    <span class="text-sm">Not Connected</span>
                </div>
                <a href="/auth" class="btn btn-was btn-sm">Connect Device</a>
            </div>
            "#.to_string())
        }
        Err(e) => {
            Html(format!(r#"
            <div class="alert alert-danger">
                <i class="bi bi-exclamation-triangle alert-icon"></i>
                <span class="alert-title">Unlink Failed</span>
                <span class="alert-description">{}</span>
            </div>
            "#, e))
        }
    }
}

// =============================================================================
// Token List Partial
// =============================================================================

/// Token list partial - shows configured API tokens
pub async fn token_list() -> impl IntoResponse {
    // Tokens are configured via app.toml, not a CRUD system
    // Show informational message
    Html(r#"
    <div class="card">
        <div class="p-6">
            <div class="d-flex flex-column align-items-center justify-content-center text-center py-4" style="color: var(--color-foreground-muted);">
                <i class="bi bi-key icon-3xl mb-3 opacity-50"></i>
                <h4 class="h5 mb-2">Authentication Configured</h4>
                <p class="small mb-3">
                    Authentication is managed via <code class="code">config/app.toml</code>
                </p>
                <div class="alert alert-info alert-compact mb-0" style="max-width: 32rem;">
                    <i class="bi bi-info-circle alert-icon"></i>
                    <p class="alert-description">
                        Use <code>[auth]</code> JWT settings for login (MCP/web) or <code>[auth].secret_key</code> for scripts/CI/CD. 
                        Use <code>Authorization: Bearer &lt;token&gt;</code> header for API requests.
                    </p>
                </div>
            </div>
        </div>
    </div>
    "#.to_string())
}

// =============================================================================
// Webhook List Partial
// =============================================================================

/// Webhook list partial - shows configured webhooks
pub async fn webhook_list() -> impl IntoResponse {
    // Webhooks are configured via app.toml
    Html(r#"
    <div class="card">
        <div class="p-6">
            <div class="d-flex flex-column align-items-center justify-content-center text-center py-4" style="color: var(--color-foreground-muted);">
                <i class="bi bi-broadcast icon-3xl mb-3 opacity-50"></i>
                <h4 class="h5 mb-2">Webhooks Configuration</h4>
                <p class="small mb-3">
                    Webhook endpoints are configured via <code class="code">config/app.toml</code>
                </p>
                <div class="alert alert-info alert-compact mb-0" style="max-width: 32rem;">
                    <i class="bi bi-info-circle alert-icon"></i>
                    <div class="alert-description">
                        <p class="mb-2">Add webhook endpoints in your config file:</p>
                        <pre class="text-start small" style="background: var(--color-background-subtle); padding: var(--space-2); border-radius: var(--radius-sm);">[webhooks]
enabled = true

[[webhooks.endpoints]]
url = "https://your-server.com/webhook"
secret = "your-hmac-secret"</pre>
                    </div>
                </div>
            </div>
        </div>
    </div>
    "#.to_string())
}
