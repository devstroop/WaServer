//! Page Handlers for HTMX Frontend
//!
//! Server-rendered HTML pages.
//! - Debug: Uses minijinja for hot-reloading templates from disk
//! - Release: Uses pre-compiled askama templates for performance

use axum::response::{Html, IntoResponse};

// =============================================================================
// Release Mode: Compiled Askama Templates
// =============================================================================

#[cfg(not(debug_assertions))]
mod compiled {
    use askama::Template;

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
}

// =============================================================================
// Debug Mode: Runtime minijinja Templates (Hot Reload)
// =============================================================================

#[cfg(debug_assertions)]
fn render_page(template_path: &str, current_page: &str) -> String {
    use crate::utils::templates::render_template;
    use serde_json::json;

    let ctx = json!({
        "current_page": current_page,
        "theme": "light",
    });

    match render_template(template_path, ctx) {
        Ok(html) => html,
        Err(e) => format!(
            r#"<html><body>
            <h1>Template Error</h1>
            <pre style="color:red;background:#111;padding:20px;border-radius:8px">{}</pre>
            <p>Fix the template and refresh the page.</p>
            </body></html>"#,
            e
        ),
    }
}

// =============================================================================
// Page Handlers
// =============================================================================

/// Dashboard page
pub async fn dashboard_page() -> impl IntoResponse {
    #[cfg(debug_assertions)]
    {
        Html(render_page("pages/dashboard.html", "dashboard"))
    }
    #[cfg(not(debug_assertions))]
    {
        use askama::Template;
        let template = compiled::DashboardTemplate {
            current_page: "dashboard",
            theme: "light".to_string(),
        };
        Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
    }
}

/// Authentication page
pub async fn auth_page() -> impl IntoResponse {
    #[cfg(debug_assertions)]
    {
        Html(render_page("pages/auth.html", "auth"))
    }
    #[cfg(not(debug_assertions))]
    {
        use askama::Template;
        let template = compiled::AuthTemplate {
            current_page: "auth",
            theme: "light".to_string(),
        };
        Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
    }
}

/// Chat page
pub async fn chat_page() -> impl IntoResponse {
    #[cfg(debug_assertions)]
    {
        Html(render_page("pages/chats.html", "chats"))
    }
    #[cfg(not(debug_assertions))]
    {
        use askama::Template;
        let template = compiled::ChatTemplate {
            current_page: "chats",
            theme: "light".to_string(),
        };
        Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
    }
}

/// Settings page
pub async fn settings_page() -> impl IntoResponse {
    #[cfg(debug_assertions)]
    {
        Html(render_page("pages/settings.html", "settings"))
    }
    #[cfg(not(debug_assertions))]
    {
        use askama::Template;
        let template = compiled::SettingsTemplate {
            current_page: "settings",
            theme: "light".to_string(),
        };
        Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
    }
}

/// Webhooks page
pub async fn webhooks_page() -> impl IntoResponse {
    #[cfg(debug_assertions)]
    {
        Html(render_page("pages/webhooks.html", "webhooks"))
    }
    #[cfg(not(debug_assertions))]
    {
        use askama::Template;
        let template = compiled::WebhooksTemplate {
            current_page: "webhooks",
            theme: "light".to_string(),
        };
        Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
    }
}

/// Access Tokens page
pub async fn tokens_page() -> impl IntoResponse {
    #[cfg(debug_assertions)]
    {
        Html(render_page("pages/tokens.html", "tokens"))
    }
    #[cfg(not(debug_assertions))]
    {
        use askama::Template;
        let template = compiled::TokensTemplate {
            current_page: "tokens",
            theme: "light".to_string(),
        };
        Html(template.render().unwrap_or_else(|e| format!("Template error: {}", e)))
    }
}
