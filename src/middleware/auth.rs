//! Authentication Middleware
//!
//! Provides API authentication via JWT tokens or static API tokens.
//! Uses `AuthState` which can be applied independently of route-specific state.

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{services::AuthTokenService, utils::logging::CorrelationId};

/// Authentication state for middleware
///
/// This is separate from route-specific state so auth can be applied to any router.
#[derive(Clone)]
pub struct AuthState {
    /// Whether authentication is enabled
    pub auth_enabled: bool,
    /// Whether local JWT auth is enabled
    pub local_auth_enabled: bool,
    /// Static API token for authentication
    pub api_token: String,
    /// JWT token service (if local auth enabled)
    pub auth_token_service: Option<Arc<AuthTokenService>>,
}

impl AuthState {
    /// Create new auth state
    pub fn new(
        auth_enabled: bool,
        local_auth_enabled: bool,
        api_token: String,
        auth_token_service: Option<Arc<AuthTokenService>>,
    ) -> Self {
        Self {
            auth_enabled,
            local_auth_enabled,
            api_token,
            auth_token_service,
        }
    }
}

/// Authentication middleware
///
/// Validates Bearer tokens (JWT or static API token) for protected endpoints.
/// Skips auth for public endpoints like health, swagger, and auth routes.
pub async fn auth_middleware(
    State(auth_state): State<AuthState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth if disabled in config
    if !auth_state.auth_enabled {
        return Ok(next.run(request).await);
    }

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
            // First, try JWT validation if local auth is enabled
            if auth_state.local_auth_enabled {
                if let Some(ref auth_token_service) = auth_state.auth_token_service {
                    if auth_token_service.validate_access_token(token).is_ok() {
                        tracing::debug!(
                            correlation_id = %correlation_id.0,
                            path = %path,
                            "JWT authentication successful"
                        );
                        return Ok(next.run(request).await);
                    }
                }
            }

            // Fall back to static token validation
            if token == auth_state.api_token {
                tracing::debug!(
                    correlation_id = %correlation_id.0,
                    path = %path,
                    "Static token authentication successful"
                );
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
        || path.starts_with("/swagger-ui")
        || path.starts_with("/api-docs")
        || path.starts_with("/mcp")
        || path == "/api/v1/auth/login"
        || path == "/api/v1/auth/refresh"
        || path == "/api/v1/auth/local-status"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_paths() {
        assert!(is_public_path("/health"));
        assert!(is_public_path("/health/live"));
        assert!(is_public_path("/swagger-ui"));
        assert!(is_public_path("/api-docs/openapi.json"));
        assert!(is_public_path("/mcp"));
        assert!(is_public_path("/api/v1/auth/login"));
        assert!(is_public_path("/api/v1/auth/refresh"));
        assert!(is_public_path("/api/v1/auth/local-status"));

        // Protected paths
        assert!(!is_public_path("/api/v1/accounts"));
        assert!(!is_public_path("/api/v1/account/status"));
        assert!(!is_public_path("/api/v1/chats"));
        assert!(!is_public_path("/api/v1/messages"));
    }
}
