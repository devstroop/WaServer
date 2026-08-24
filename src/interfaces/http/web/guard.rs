//! Web session guard — resolves the `was_session` cookie to an
//! `AuthenticatedUser`, redirecting (or `HX-Redirect`-ing) to login on failure.

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use super::{
    csrf::derive_csrf,
    pages::AppState,
    session::{read_session_token, redirect, resolve_session, WebSession},
};

#[allow(clippy::result_large_err)]
pub async fn web_auth_middleware(
    State(app): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path().to_string();
    let Some(token) = read_session_token(request.headers()) else {
        return Err(unauthorized(&request, &path));
    };
    let Some(user) = resolve_session(&app.db, &token) else {
        return Err(unauthorized(&request, &path));
    };

    let csrf_token = derive_csrf(&token);
    request
        .extensions_mut()
        .insert(WebSession { user, csrf_token });
    Ok(next.run(request).await)
}

/// Redirect to login preserving the target so login can bounce back (`?next=`)
fn unauthorized(request: &Request, path: &str) -> Response {
    redirect(
        request.headers(),
        &format!("/app/login?next={}", urlencode(path)),
    )
}

fn urlencode(s: &str) -> String {
    s.bytes()
        .map(
            |b| match b.is_ascii_alphanumeric() || b"-.~/=_&?".contains(&b) {
                true => (b as char).to_string(),
                false => format!("%{b:02X}"),
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlencode_keeps_paths_safe() {
        assert_eq!(urlencode("/app/instances/abc-1"), "/app/instances/abc-1");
        assert_eq!(urlencode("/a b"), "/a%20b");
    }
}
