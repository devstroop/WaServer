//! Page Handlers for HTMX Frontend
//!
//! Server-rendered HTML pages using Askama templates.

use askama::Template;
use axum::response::{Html, IntoResponse};

// =============================================================================
// Template Definitions
// =============================================================================

#[derive(Template)]
#[template(path = "pages/dashboard.html")]
pub struct DashboardTemplate {
    pub current_page: &'static str,
    pub theme: String,
}

#[derive(Template)]
#[template(path = "pages/auth.html")]
pub struct AuthTemplate {
    pub current_page: &'static str,
    pub theme: String,
}

#[derive(Template)]
#[template(path = "pages/chats.html")]
pub struct ChatTemplate {
    pub current_page: &'static str,
    pub theme: String,
}

#[derive(Template)]
#[template(path = "pages/settings.html")]
pub struct SettingsTemplate {
    pub current_page: &'static str,
    pub theme: String,
}

#[derive(Template)]
#[template(path = "pages/webhooks.html")]
pub struct WebhooksTemplate {
    pub current_page: &'static str,
    pub theme: String,
}

#[derive(Template)]
#[template(path = "pages/tokens.html")]
pub struct TokensTemplate {
    pub current_page: &'static str,
    pub theme: String,
}

// =============================================================================
// Page Handlers
// =============================================================================

/// Dashboard page
pub async fn dashboard_page() -> impl IntoResponse {
    let template = DashboardTemplate {
        current_page: "dashboard",
        theme: "light".to_string(),
    };
    Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
}

/// Authentication page
pub async fn auth_page() -> impl IntoResponse {
    let template = AuthTemplate {
        current_page: "auth",
        theme: "light".to_string(),
    };
    Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
}

/// Chat page
pub async fn chat_page() -> impl IntoResponse {
    let template = ChatTemplate {
        current_page: "chats",
        theme: "light".to_string(),
    };
    Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
}

/// Settings page
pub async fn settings_page() -> impl IntoResponse {
    let template = SettingsTemplate {
        current_page: "settings",
        theme: "light".to_string(),
    };
    Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
}

/// Webhooks page
pub async fn webhooks_page() -> impl IntoResponse {
    let template = WebhooksTemplate {
        current_page: "webhooks",
        theme: "light".to_string(),
    };
    Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
}

/// Access Tokens page
pub async fn tokens_page() -> impl IntoResponse {
    let template = TokensTemplate {
        current_page: "tokens",
        theme: "light".to_string(),
    };
    Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
}
