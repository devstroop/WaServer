//! CSRF protection for state-changing web routes (#28)
//!
//! Stateless scheme: the CSRF token is `SHA256("was-csrf:<session-token>")` —
//! an attacker cannot read the httpOnly session cookie, so they cannot compute
//! it. Pages render the token into `<meta name="csrf-token">`; the app JS shim
//! copies it onto every htmx request as `X-CSRF-Token`.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use super::session::read_session_token;

/// Derive the per-session CSRF token from the raw session secret
pub fn derive_csrf(session_token: &str) -> String {
    crate::middleware::auth::hash_token(&format!("was-csrf:{session_token}"))
}

const CSRF_HEADER: &str = "x-csrf-token";

/// Reject unsafe-method requests whose `X-CSRF-Token` does not match the
/// token derived from the caller's session cookie. GET/HEAD/OPTIONS pass.
#[allow(clippy::result_large_err)]
pub async fn csrf_middleware(request: Request, next: Next) -> Result<Response, Response> {
    let unsafe_method = !matches!(
        request.method().as_str(),
        "GET" | "HEAD" | "OPTIONS" | "TRACE"
    );
    if !unsafe_method {
        return Ok(next.run(request).await);
    }

    let Some(token) = read_session_token(request.headers()) else {
        return Err((
            StatusCode::FORBIDDEN,
            axum::response::Html(super::pages::error_fragment(
                "Session expired. Reload the page.",
            )),
        )
            .into_response());
    };
    let expected = derive_csrf(&token);
    let provided = request
        .headers()
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    if !constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        tracing::warn!("web request rejected — CSRF token mismatch");
        return Err((
            StatusCode::FORBIDDEN,
            axum::response::Html(super::pages::error_fragment(
                "Invalid or missing CSRF token.",
            )),
        )
            .into_response());
    }
    Ok(next.run(request).await)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_csrf_deterministic_and_distinct() {
        let a = derive_csrf("tok1");
        assert_eq!(a, derive_csrf("tok1"));
        assert_ne!(a, derive_csrf("tok2"));
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
    }
}
