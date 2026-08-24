//! Cookie sessions for the web admin UI (#28)
//!
//! Reuses the existing session-token model: login mints a random token stored
//! hashed as a "Web Session" access token; the raw value rides in an httpOnly
//! cookie. Bearer API auth is untouched.

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Form,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    interfaces::http::web::pages::login_error_fragment, models::auth::AuthenticatedUser,
    services::Database, utils::logging::CorrelationId,
};

pub const SESSION_COOKIE: &str = "was_session";
pub const LOGIN_CSRF_COOKIE: &str = "was_login_csrf";

/// Raw session token for the current request (from cookie), if valid.
#[derive(Clone)]
pub struct WebSession {
    pub user: AuthenticatedUser,
    /// CSRF token derived from the session secret — safe to render into pages.
    pub csrf_token: String,
}

pub fn session_cookie_value(token: &str) -> String {
    format!("{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax")
}

pub fn clear_session_cookie() -> String {
    format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

pub fn login_csrf_cookie_value(v: &str) -> String {
    format!("{LOGIN_CSRF_COOKIE}={v}; Path=/app/login; SameSite=Lax")
}

/// Extract the raw session token from the request's cookies
pub fn read_session_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    for pair in raw.split(';') {
        let pair = pair.trim();
        if let Some(value) = pair.strip_prefix(&format!("{SESSION_COOKIE}=")) {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Resolve a session token to its owning user via the DB (same store as API tokens)
pub fn resolve_session(db: &Database, token: &str) -> Option<AuthenticatedUser> {
    let hash = crate::middleware::auth::hash_token(token);
    db.get_user_by_access_token(&hash)
        .ok()
        .flatten()
        .map(|(user_record, _)| AuthenticatedUser::User {
            id: user_record.id,
            username: user_record.username,
            role: user_record.role,
        })
}

/// True when the request came from htmx (v4 sends `HX-Request: true`)
fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get("hx-request")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("true"))
}

/// Redirect helper honoring htmx: boosted/partial requests need `HX-Redirect`
/// instead of a 3xx (htmx fetch follows redirects transparently).
pub fn redirect(headers: &HeaderMap, location: &str) -> Response {
    if is_htmx(headers) {
        return (StatusCode::OK, [("hx-redirect", location.to_string())]).into_response();
    }
    (StatusCode::SEE_OTHER, [(header::LOCATION, location)]).into_response()
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub csrf: Option<String>,
}

#[allow(clippy::result_large_err)]
pub async fn login_post(
    State(db): State<Database>,
    headers: HeaderMap,
    Form(form): Form<LoginForm>,
) -> Response {
    // Double-submit CSRF for the pre-session login form
    let expected = headers
        .get(header::COOKIE)
        .and_then(|c| c.to_str().ok())
        .and_then(|raw| {
            raw.split(';').find_map(|pair| {
                let pair = pair.trim();
                pair.strip_prefix(&format!("{LOGIN_CSRF_COOKIE}="))
            })
        })
        .unwrap_or_default();
    if form.csrf.as_deref().unwrap_or_default() != expected || expected.is_empty() {
        return (
            StatusCode::FORBIDDEN,
            axum::response::Html(login_error_fragment(
                "Session expired — reload the page and try again.",
            )),
        )
            .into_response();
    }

    // Find user by username or email (mirrors api::auth::login)
    let user = db
        .get_user_by_username(&form.username)
        .ok()
        .flatten()
        .or_else(|| db.get_user_by_email(&form.username).ok().flatten());

    let Some(user) = user else {
        return (
            StatusCode::UNAUTHORIZED,
            axum::response::Html(login_error_fragment("Invalid username/email or password.")),
        )
            .into_response();
    };
    if !crate::middleware::auth::verify_password(&form.password, &user.password_hash) {
        return (
            StatusCode::UNAUTHORIZED,
            axum::response::Html(login_error_fragment("Invalid username/email or password.")),
        )
            .into_response();
    }
    if !user.is_active {
        return (
            StatusCode::FORBIDDEN,
            axum::response::Html(login_error_fragment(
                "This account is inactive. Contact an administrator.",
            )),
        )
            .into_response();
    }

    // Mint session token stored hashed as a "Web Session" access token
    let token = format!("session_{}", Uuid::new_v4().simple());
    let token_hash = crate::middleware::auth::hash_token(&token);
    if let Err(e) = db.create_access_token(
        &Uuid::new_v4().to_string(),
        &user.id,
        "Web Session",
        &token_hash,
        None,
    ) {
        tracing::error!(error = %e, "web login failed to persist session");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::response::Html(login_error_fragment("Could not create session. Try again.")),
        )
            .into_response();
    }
    tracing::info!(user_id = %user.id, username = %user.username, correlation_id = %CorrelationId::new().0, "web login");

    let target = sanitize_next(form.next.as_deref()).unwrap_or("/app");
    let mut resp = redirect(&headers, target);
    resp.headers_mut().append(
        header::SET_COOKIE,
        header::HeaderValue::from_str(&session_cookie_value(&token)).unwrap(),
    );
    resp
}

/// Only allow same-site absolute paths for post-login redirects
fn sanitize_next(next: Option<&str>) -> Option<&str> {
    match next {
        Some(n) if n.starts_with('/') && !n.starts_with("//") => Some(n),
        _ => None,
    }
}

pub async fn logout(State(db): State<Database>, headers: HeaderMap) -> Response {
    if let Some(token) = read_session_token(&headers) {
        let hash = crate::middleware::auth::hash_token(&token);
        // Best-effort delete of the session record ("Web Session" tokens only)
        if let Ok(Some((_, token_record))) = db.get_user_by_access_token(&hash) {
            let _ = db.delete_access_token(&token_record.id, &token_record.user_id);
        }
    }
    let mut resp = redirect(&headers, "/app/login");
    resp.headers_mut().append(
        header::SET_COOKIE,
        header::HeaderValue::from_str(&clear_session_cookie()).unwrap(),
    );
    resp
}
