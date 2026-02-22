//! Authentication Middleware
//!
//! Provides API authentication via static secret key.
//! Uses `AuthState` which can be applied independently of route-specific state.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{
    models::auth::AuthenticatedUser,
    utils::logging::CorrelationId,
};

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
) -> Result<Response, StatusCode> {
    // Skip auth for public endpoints
    let path = request.uri().path();
    if is_public_path(path) {
        return Ok(next.run(request).await);
    }

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

    Err(StatusCode::UNAUTHORIZED)
}

/// Check if path is public (no auth required)
fn is_public_path(path: &str) -> bool {
    path.starts_with("/health")
        || path.starts_with("/ready")
        || path.starts_with("/live")
        || path.starts_with("/metrics")
        || path.starts_with("/api-docs")
        || path.starts_with("/docs")
        || path.starts_with("/api/health")
        || path.starts_with("/api/ready")
        || path.starts_with("/api/live")
        || path.starts_with("/api/metrics")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_paths() {
        assert!(is_public_path("/health"));
        assert!(is_public_path("/health/live"));
        assert!(is_public_path("/api-docs"));
        assert!(is_public_path("/api-docs/openapi.json"));
        assert!(is_public_path("/docs"));
        assert!(is_public_path("/api/health"));
        assert!(is_public_path("/api/ready"));
        assert!(is_public_path("/api/live"));
        assert!(is_public_path("/api/metrics"));

        // Protected paths (require auth)
        assert!(!is_public_path("/mcp"));
        assert!(!is_public_path("/api/v1/accounts"));
        assert!(!is_public_path("/api/v1/account/status"));
        assert!(!is_public_path("/api/v1/chats"));
        assert!(!is_public_path("/api/v1/messages"));
    }
}
