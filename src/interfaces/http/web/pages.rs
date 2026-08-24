//! Web pages — login and the guarded app shell (#28)
//! Askama templates render to HTML; fragments (error partials) are swap-friendly.

use askama::Template;
use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{middleware::auth::AuthState, models::auth::AuthenticatedUser, services::Database};

use super::{
    csrf::csrf_middleware,
    guard::web_auth_middleware,
    session::{login_csrf_cookie_value, login_post, logout, WebSession, LOGIN_CSRF_COOKIE},
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Flattened session view for templates
#[derive(Clone)]
pub struct SessionView {
    pub csrf_token: String,
    pub username: String,
    pub role: String,
    pub admin: bool,
}

/// Extractor: pulls `WebSession` from extensions and flattens for templates
pub struct WebSessionExt(pub SessionView);

impl std::ops::Deref for WebSessionExt {
    type Target = SessionView;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait::async_trait]
impl axum::extract::FromRequestParts<crate::services::Database> for WebSessionExt {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &crate::services::Database,
    ) -> Result<Self, Self::Rejection> {
        from_parts(parts)
    }
}

#[async_trait::async_trait]
impl axum::extract::FromRequestParts<crate::services::InstanceManager> for WebSessionExt {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &crate::services::InstanceManager,
    ) -> Result<Self, Self::Rejection> {
        from_parts(parts)
    }
}

#[async_trait::async_trait]
impl axum::extract::FromRequestParts<AppState> for WebSessionExt {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        from_parts(parts)
    }
}

#[allow(clippy::result_large_err)]
fn from_parts(parts: &mut axum::http::request::Parts) -> Result<WebSessionExt, Response> {
    match parts.extensions.get::<WebSession>() {
        Some(s) => {
            let (username, role, admin) = match &s.user {
                AuthenticatedUser::Secret => ("superadmin".to_string(), "admin".to_string(), true),
                AuthenticatedUser::User { username, role, .. } => {
                    let r = format!("{role:?}").to_lowercase();
                    let admin = r == "admin";
                    (username.clone(), r, admin)
                }
            };
            Ok(WebSessionExt(SessionView {
                csrf_token: s.csrf_token.clone(),
                username,
                role,
                admin,
            }))
        }
        None => Err(StatusCode::UNAUTHORIZED.into_response()),
    }
}

#[derive(Template)]
#[template(path = "web/login.html")]
pub struct LoginTemplate {
    pub title: String,
    pub version: &'static str,
    pub csrf: String,
    pub next: Option<String>,
}

/// Swap-friendly error fragment used by CSRF/auth failures
#[derive(Template)]
#[template(path = "web/_error.html")]
pub struct ErrorFragment {
    pub message: String,
}

pub fn error_fragment(message: &str) -> String {
    ErrorFragment {
        message: message.to_string(),
    }
    .render()
    .unwrap_or_default()
}

pub fn login_error_fragment(message: &str) -> String {
    error_fragment(message)
}

fn random_token() -> String {
    Uuid::new_v4().simple().to_string()
}

fn set_cookie(resp: &mut Response, value: &str) {
    if let Ok(v) = header::HeaderValue::from_str(value) {
        resp.headers_mut().append(header::SET_COOKIE, v);
    }
}

/// `GET /app/login` — renders the login form; seeds the double-submit CSRF cookie
pub async fn login_get(headers: HeaderMap) -> Response {
    // Reuse an existing token so refreshes don't rotate needlessly
    let csrf = headers
        .get(header::COOKIE)
        .and_then(|c| c.to_str().ok())
        .and_then(|raw| {
            raw.split(';')
                .find_map(|pair| pair.trim().strip_prefix(&format!("{LOGIN_CSRF_COOKIE}=")))
        })
        .map(str::to_string)
        .unwrap_or_else(random_token);

    let tpl = LoginTemplate {
        title: "Sign in — WAS".into(),
        version: VERSION,
        csrf: csrf.clone(),
        next: None,
    };
    let mut resp = (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response();
    set_cookie(&mut resp, &login_csrf_cookie_value(&csrf));
    resp
}

/// `GET /app` — guarded shell page (dashboard lands in #30)
pub async fn shell(session: WebSessionExt) -> Response {
    #[derive(Template)]
    #[template(path = "web/shell.html")]
    struct ShellTemplate {
        title: String,
        version: &'static str,
        csrf: String,
        username: String,
        role: String,
        admin: bool,
    }
    let tpl = ShellTemplate {
        title: "WAS Admin".into(),
        version: VERSION,
        csrf: session.csrf_token.clone(),
        username: session.username.clone(),
        role: session.role.clone(),
        admin: session.admin,
    };
    (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response()
}

static BROWSER_AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

fn browser_available() -> bool {
    *BROWSER_AVAILABLE.get_or_init(|| crate::services::maintenance::detect_browser().is_ok())
}

#[derive(Template)]
#[template(path = "web/_overview.html")]
pub struct OverviewTemplate {
    pub version: &'static str,
    pub uptime: String,
    pub messages_sent: u64,
    pub browser_available: bool,
    pub total: usize,
    pub active: usize,
    pub warming_up: usize,
    pub sleeping: usize,
    pub errored: usize,
    pub instances: Vec<OverviewInstanceRow>,
}

pub struct OverviewInstanceRow {
    pub id: String,
    pub label: String,
    pub phone: String,
    pub status: &'static str,
    pub badge_class: &'static str,
}

fn format_uptime(secs: u64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    if d > 0 {
        format!("{d}d {h}h")
    } else if h > 0 {
        format!("{h}h {m}m")
    } else {
        format!("{m}m")
    }
}

/// Shared state for guarded web routes — handlers pull what they need
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub manager: Arc<crate::services::InstanceManager>,
    pub session_ttl_hours: u64,
}

/// `GET /app/fragments/overview` — polled dashboard fragment (#30)
/// Cheap on purpose: in-memory locks + atomic counters, no browser round-trips.
pub async fn overview_fragment(State(app): State<AppState>) -> Response {
    use crate::models::instance::InstanceStatus;

    let snapshots = app.manager.list_status_snapshots().await;
    let obs = app.manager.observability.snapshot_all().await;
    let messages_sent: u64 = obs.values().map(|s| s.messages_sent).sum();

    let mut active = 0usize;
    let mut warming_up = 0usize;
    let mut sleeping = 0usize;
    let mut errored = 0usize;
    let mut rows = Vec::with_capacity(snapshots.len());
    for s in &snapshots {
        let (status, badge_class) = match s.status {
            InstanceStatus::Active => ("active", "dt-badge--success"),
            InstanceStatus::WarmingUp => ("warming up", "dt-badge--warning"),
            InstanceStatus::Sleeping => ("sleeping", "dt-badge--neutral"),
            InstanceStatus::Error(_) => ("error", "dt-badge--danger"),
        };
        match s.status {
            InstanceStatus::Active => active += 1,
            InstanceStatus::WarmingUp => warming_up += 1,
            InstanceStatus::Sleeping => sleeping += 1,
            InstanceStatus::Error(_) => errored += 1,
        }
        rows.push(OverviewInstanceRow {
            id: s.id.to_string(),
            label: s.name.clone().unwrap_or_else(|| s.id.to_string()),
            phone: s.phone.clone().unwrap_or_else(|| "—".into()),
            status,
            badge_class,
        });
    }

    let tpl = OverviewTemplate {
        version: VERSION,
        uptime: format_uptime(crate::handlers::api::health::uptime_seconds()),
        messages_sent,
        browser_available: browser_available(),
        total: snapshots.len(),
        active,
        warming_up,
        sleeping,
        errored,
        instances: rows,
    };
    (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response()
}

/// `/app` router: public login + guarded shell/logout/fragments
pub fn router(auth_state: AuthState, manager: Arc<crate::services::InstanceManager>) -> Router<()> {
    let state = AppState {
        session_ttl_hours: auth_state.session_ttl_hours,
        db: auth_state.db.clone(),
        manager,
    };

    let public = Router::new()
        .route("/login", axum::routing::get(login_get).post(login_post))
        .with_state(auth_state);

    // Layer order: auth (outer) runs before csrf (inner); both wrap handlers.
    let protected = Router::new()
        .route("/", axum::routing::get(shell))
        .route("/logout", axum::routing::post(logout))
        .route(
            "/logout-all",
            axum::routing::post(super::session::logout_all),
        )
        .route("/fragments/overview", axum::routing::get(overview_fragment))
        // instances (#31)
        .route(
            "/instances",
            axum::routing::get(super::instances::list_page),
        )
        .route(
            "/instances",
            axum::routing::post(super::instances::create_post),
        )
        .route(
            "/fragments/instances",
            axum::routing::get(super::instances::table_fragment),
        )
        .route(
            "/instances/:id",
            axum::routing::get(super::instances::detail_page),
        )
        .route(
            "/fragments/instances/:id/status",
            axum::routing::get(super::instances::status_fragment),
        )
        .route(
            "/fragments/instances/:id/link",
            axum::routing::get(super::instances::link_fragment),
        )
        .route(
            "/fragments/instances/:id/qr.png",
            axum::routing::get(super::instances::qr_png),
        )
        .route(
            "/instances/:id/shot",
            axum::routing::get(super::instances::shot_fragment),
        )
        .route(
            "/fragments/instances/:id/screenshot.png",
            axum::routing::get(super::instances::screenshot_png),
        )
        .route(
            "/instances/:id/warmup",
            axum::routing::post(super::instances::warmup_post),
        )
        .route(
            "/instances/:id/reset",
            axum::routing::post(super::instances::reset_post),
        )
        .route(
            "/instances/:id/delete",
            axum::routing::post(super::instances::delete_post),
        )
        .route(
            "/instances/:id/config",
            axum::routing::post(super::instances::config_post),
        )
        .route(
            "/instances/:id/send",
            axum::routing::post(super::instances::send_post),
        )
        // users & tokens admin (#33)
        .route("/users", axum::routing::get(super::users::list_page))
        .route("/users", axum::routing::post(super::users::create_post))
        .route(
            "/fragments/users",
            axum::routing::get(super::users::table_fragment),
        )
        .route("/users/:id", axum::routing::get(super::users::detail_page))
        .route(
            "/users/:id/delete",
            axum::routing::post(super::users::delete_post),
        )
        .route(
            "/users/:id/update",
            axum::routing::post(super::users::update_post),
        )
        .route(
            "/users/:id/tokens",
            axum::routing::post(super::users::create_token_post),
        )
        .route(
            "/users/:id/tokens/:token_id/revoke",
            axum::routing::post(super::users::revoke_token_post),
        )
        .route(
            "/users/:id/assign",
            axum::routing::post(super::users::assign_post),
        )
        .route(
            "/users/:id/unassign/:instance_id",
            axum::routing::post(super::users::unassign_post),
        )
        .route(
            "/fragments/users/:id/assignments",
            axum::routing::get(super::users::assignments_fragment),
        )
        .route("/me", axum::routing::get(super::users::me_page))
        .layer(axum::middleware::from_fn(csrf_middleware))
        .layer(from_fn_with_state(state.clone(), web_auth_middleware))
        .with_state(state);

    public.merge(protected)
}
