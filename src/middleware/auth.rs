//! Authentication Middleware
//!
//! Provides API authentication via static secret key.
//! Uses `AuthState` which can be applied independently of route-specific state.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{models::auth::AuthenticatedUser, utils::logging::CorrelationId};

/// Authentication state for middleware
#[derive(Clone)]
pub struct AuthState {
    /// Static secret key for authentication
    pub secret_key: String,
}

impl AuthState {
    /// Create new auth state
    pub fn new(secret_key: String) -> Self {
        Self { secret_key }
    }
}

/// Authentication middleware
///
/// Validates Bearer tokens (static API token) for protected endpoints.
/// Skips auth for public endpoints like health and swagger.
///
/// On successful authentication, adds `AuthenticatedUser` to request extensions.
pub async fn auth_middleware(
    State(auth_state): State<AuthState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path().to_string();

    // Get correlation ID for logging
    let correlation_id = request
        .extensions()
        .get::<CorrelationId>()
        .cloned()
        .unwrap_or_else(CorrelationId::new);

    // Extract the Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok());

    if let Some(auth_value) = auth_header {
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            // Validate static secret key
            if token == auth_state.secret_key {
                let authenticated_user = AuthenticatedUser::Secret;
                tracing::debug!(
                    correlation_id = %correlation_id.0,
                    path = %path,
                    auth_method = "secret",
                    "Secret key authentication successful"
                );
                request.extensions_mut().insert(authenticated_user);
                return Ok(next.run(request).await);
            }
        }
    }

    tracing::warn!(
        correlation_id = %correlation_id.0,
        path = %path,
        "Authentication failed - invalid or missing token"
    );

    Err((
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "errors": [{
                "status_code": 401,
                "message": "Authentication failed - invalid or missing token",
                "path": path,
                "correlation_id": correlation_id.0
            }]
        })),
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_state_creation() {
        let state = AuthState::new("test-secret-key-12345".to_string());
        assert_eq!(state.secret_key, "test-secret-key-12345");
    }
}
