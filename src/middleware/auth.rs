//! Authentication Middleware
//!
//! Provides API authentication via JWT tokens or static secret tokens.
//! Uses `AuthState` which can be applied independently of route-specific state.
//!
//! ## Authentication Methods
//! 
//! 1. **Static Secret Key** (`[auth].secret_key` config): Simple machine-to-machine access
//!    - Use for: External scripts, CI/CD pipelines, simple integrations
//!    - Token is configured in `app.toml` under `auth.secret_key`
//!
//! 2. **Local User JWT** (`[auth]` JWT settings): User-based access  
//!    - Use for: Web UI dashboard, MCP clients, user-specific access control
//!    - Requires login via `/api/v1/auth/login`
//!    - JWT tokens contain username for auditing

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{
    models::auth::AuthenticatedUser,
    services::AuthTokenService,
    utils::logging::CorrelationId,
};

/// Authentication state for middleware
///
/// This is separate from route-specific state so auth can be applied to any router.
#[derive(Clone)]
pub struct AuthState {
    /// Static secret key for authentication (for scripts/CI/CD)
    pub secret_key: String,
    /// JWT token service (for local user authentication)
    pub auth_token_service: Option<Arc<AuthTokenService>>,
}

impl AuthState {
    /// Create new auth state
    pub fn new(
        secret_key: String,
        auth_token_service: Option<Arc<AuthTokenService>>,
    ) -> Self {
        Self {
            secret_key,
            auth_token_service,
        }
    }
}

/// Authentication middleware
///
/// Validates Bearer tokens (JWT or static API token) for protected endpoints.
/// Skips auth for public endpoints like health, swagger, and auth routes.
///
/// On successful authentication, adds `AuthenticatedUser` to request extensions
/// so handlers can determine how the request was authenticated.
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
            // First, try JWT validation (local user authentication)
            if let Some(ref auth_token_service) = auth_state.auth_token_service {
                if let Ok(username) = auth_token_service.validate_access_token(token) {
                    // Look up the full user to get user_id and is_admin
                    match auth_token_service.get_user(&username) {
                        Ok(Some(user)) => {
                            let authenticated_user = AuthenticatedUser::LocalUser { 
                                user_id: user.id,
                                username: username.clone(),
                                is_admin: user.is_admin,
                            };
                            tracing::debug!(
                                correlation_id = %correlation_id.0,
                                path = %path,
                                auth_method = "jwt",
                                user = %username,
                                user_id = %user.id,
                                is_admin = user.is_admin,
                                "JWT authentication successful"
                            );
                            // Store authenticated user in request extensions
                            request.extensions_mut().insert(authenticated_user);
                            return Ok(next.run(request).await);
                        }
                        Ok(None) => {
                            tracing::warn!(
                                correlation_id = %correlation_id.0,
                                path = %path,
                                user = %username,
                                "JWT valid but user not found in database"
                            );
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                        Err(e) => {
                            tracing::error!(
                                correlation_id = %correlation_id.0,
                                path = %path,
                                error = %e,
                                "Database error during auth"
                            );
                            return Err(StatusCode::INTERNAL_SERVER_ERROR);
                        }
                    }
                }
            }

            // Fall back to static secret key validation
            if token == auth_state.secret_key {
                let authenticated_user = AuthenticatedUser::Secret;
                tracing::debug!(
                    correlation_id = %correlation_id.0,
                    path = %path,
                    auth_method = "secret",
                    "Static secret token authentication successful"
                );
                // Store authenticated user in request extensions
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

// =============================================================================
// Helper Functions
// =============================================================================

/// Extract authenticated user from request extensions
///
/// Use this in handlers to determine how the request was authenticated:
/// ```rust,ignore
/// async fn my_handler(
///     Extension(auth_user): Extension<AuthenticatedUser>,
/// ) -> impl IntoResponse {
///     match auth_user {
///         AuthenticatedUser::LocalUser { username } => {
///             // User-based access - log username, apply user-specific limits
///         }
///         AuthenticatedUser::Secret => {
///             // Secret token access - scripts/CI/CD
///         }
///     }
/// }
/// ```
///
/// Or with Optional extraction (for routes that might not have auth):
/// ```rust,ignore
/// use axum::Extension;
/// async fn my_handler(
///     auth_user: Option<Extension<AuthenticatedUser>>,
/// ) -> impl IntoResponse {
///     if let Some(Extension(user)) = auth_user {
///         // authenticated
///     }
/// }
/// ```
pub fn get_authenticated_user(request: &Request) -> Option<AuthenticatedUser> {
    request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
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
        || path.starts_with("/api/v1/auth")  // All auth routes are public (login, setup, etc.)
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
        assert!(is_public_path("/api/v1/auth/login"));
        assert!(is_public_path("/api/v1/auth/refresh"));
        assert!(is_public_path("/api/v1/auth/current-user"));
        assert!(is_public_path("/api/v1/auth/setup"));
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
