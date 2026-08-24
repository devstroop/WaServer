//! Web pages — login and the guarded app shell (#28)
//! Askama templates render to HTML; fragments (error partials) are swap-friendly.

use askama::Template;
use axum::{
    http::{header, HeaderMap, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    Extension, Router,
};
use uuid::Uuid;

use crate::{middleware::auth::AuthState, models::auth::AuthenticatedUser};

use super::{
    csrf::csrf_middleware,
    guard::web_auth_middleware,
    session::{login_csrf_cookie_value, login_post, logout, WebSession, LOGIN_CSRF_COOKIE},
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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
pub async fn shell(Extension(session): Extension<WebSession>) -> Response {
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
    let (username, role, admin) = match &session.user {
        AuthenticatedUser::Secret => ("superadmin".to_string(), "admin".to_string(), true),
        AuthenticatedUser::User { username, role, .. } => {
            let r = format!("{role:?}").to_lowercase();
            let admin = r == "admin";
            (username.clone(), r, admin)
        }
    };
    let tpl = ShellTemplate {
        title: "WAS Admin".into(),
        version: VERSION,
        csrf: session.csrf_token,
        username,
        role,
        admin,
    };
    (
        StatusCode::OK,
        axum::response::Html(tpl.render().unwrap_or_default()),
    )
        .into_response()
}

/// `/app` router: public login + guarded shell/logout
pub fn router(auth_state: AuthState) -> Router<()> {
    let db = auth_state.db;

    let public = Router::new()
        .route("/login", axum::routing::get(login_get).post(login_post))
        .with_state(db.clone());

    // Layer order: auth (outer) runs before csrf (inner); both wrap handlers.
    let protected = Router::new()
        .route("/", axum::routing::get(shell))
        .route("/logout", axum::routing::post(logout))
        .layer(axum::middleware::from_fn(csrf_middleware))
        .layer(from_fn_with_state(db.clone(), web_auth_middleware))
        .with_state(db);

    public.merge(protected)
}
