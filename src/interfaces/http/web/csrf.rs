//! CSRF protection for state-changing web routes (#28)
//!
//! Stateless scheme: the CSRF token is `SHA256("was-csrf:<session-token>")` —
//! an attacker cannot read the httpOnly session cookie, so they cannot compute
//! it. Pages render the token into `<meta name="csrf-token">`; the app JS shim
//! copies it onto every htmx request as `X-CSRF-Token`.

use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use super::session::read_session_token;

/// Derive the per-session CSRF token from the raw session secret
pub fn derive_csrf(session_token: &str) -> String {
    crate::middleware::auth::hash_token(&format!("was-csrf:{session_token}"))
}

const CSRF_HEADER: &str = "x-csrf-token";
const CSRF_FIELD: &str = "csrf=";

/// Reject unsafe-method requests without a valid CSRF token. Accepted sources:
/// `X-CSRF-Token` header (htmx shim) or a `csrf=` field in a small urlencoded
/// body (no-JS fallback). GET/HEAD/OPTIONS pass.
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
    let provided_header = request
        .headers()
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    if constant_time_eq(provided_header.as_bytes(), expected.as_bytes()) {
        return Ok(next.run(request).await);
    }

    // Header mismatch/absent — try the form field (buffer small urlencoded bodies)
    let is_form = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/x-www-form-urlencoded"));

    if is_form {
        let (parts, body) = request.into_parts();
        match axum::body::to_bytes(body, 64 * 1024).await {
            Ok(bytes) => {
                let provided_field = extract_form_field(&bytes, CSRF_FIELD);
                if constant_time_eq(provided_field.as_bytes(), expected.as_bytes()) {
                    let request = Request::from_parts(parts, Body::from(bytes));
                    return Ok(next.run(request).await);
                }
            }
            Err(_) => return Err(csrf_reject()),
        }
    }

    tracing::warn!("web request rejected — CSRF token mismatch");
    Err(csrf_reject())
}

fn csrf_reject() -> Response {
    (
        StatusCode::FORBIDDEN,
        axum::response::Html(super::pages::error_fragment(
            "Invalid or missing CSRF token.",
        )),
    )
        .into_response()
}

/// Scan urlencoded body bytes for a `name=value` pair (tokens are hex — no
/// percent-decoding needed)
fn extract_form_field(body: &[u8], prefix: &str) -> String {
    for pair in body.split(|&b| b == b'&') {
        if pair.starts_with(prefix.as_bytes()) {
            return String::from_utf8_lossy(&pair[prefix.len()..]).into_owned();
        }
    }
    String::new()
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
