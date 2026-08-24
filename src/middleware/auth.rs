//! Authentication Middleware
//!
//! Provides API authentication via:
//! - Static secret key (superadmin access)
//! - Access tokens (user API access)
//! - Session tokens (web UI access - JWT based)
//!
//! Uses `AuthState` which can be applied independently of route-specific state.

use axum::{
    extract::{OriginalUri, Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    application::auth::{hash_token as app_hash_token, SecretValidator},
    models::auth::AuthenticatedUser,
    services::Database,
    utils::logging::CorrelationId,
};

/// Authentication state for middleware
#[derive(Clone)]
pub struct AuthState {
    /// Optional static secret key for superadmin authentication.
    /// When `None`/empty, the static-key path is disabled entirely (opt-in auth).
    pub secret_key: Option<String>,
    /// Database for user/token lookup
    pub db: Database,
}

impl AuthState {
    /// Create new auth state
    pub fn new(secret_key: Option<String>, db: Database) -> Self {
        Self {
            secret_key: secret_key.filter(|k| !k.trim().is_empty()),
            db,
        }
    }
}

/// Hash a token for secure comparison/storage.
/// Delegates to `application::auth::hash_token` (SHA256) — keeps middleware free of crypto impl.
pub fn hash_token(token: &str) -> String {
    app_hash_token(token)
}

/// Hash a password using SHA256 (for simplicity; use bcrypt/argon2 in production)
pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}

/// Authentication middleware
///
/// Validates Bearer tokens for protected endpoints.
/// Checks in order:
/// 1. Static secret key (superadmin access)
/// 2. Access tokens from database (API access)
///
/// On successful authentication, adds `AuthenticatedUser` to request extensions.
#[allow(clippy::result_large_err)]
pub async fn auth_middleware(
    State(auth_state): State<AuthState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Use OriginalUri to get the full path (before nest stripping)
    let path = request
        .extensions()
        .get::<OriginalUri>()
        .map(|uri| uri.0.path().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());

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
            // 1. Static secret key (superadmin) — only when configured (opt-in).
            //    Constant-time compare to avoid timing leak.
            if let Some(secret) = &auth_state.secret_key {
                if SecretValidator::constant_time_eq(token, secret) {
                    let authenticated_user = AuthenticatedUser::Secret;
                    tracing::debug!(
                        correlation_id = %correlation_id.0,
                        path = %path,
                        auth_method = "secret",
                        "Superadmin secret key authentication successful"
                    );
                    request.extensions_mut().insert(authenticated_user);
                    return Ok(next.run(request).await);
                }
            }

            // 2. Check access tokens from database
            let token_hash = hash_token(token);
            if let Ok(Some((user_record, _token_record))) =
                auth_state.db.get_user_by_access_token(&token_hash)
            {
                let authenticated_user = AuthenticatedUser::User {
                    id: user_record.id.clone(),
                    username: user_record.username.clone(),
                    role: user_record.role,
                };
                tracing::debug!(
                    correlation_id = %correlation_id.0,
                    path = %path,
                    auth_method = "access_token",
                    user_id = %user_record.id,
                    username = %user_record.username,
                    "Access token authentication successful"
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
    fn test_hash_token() {
        let hash1 = hash_token("test-token");
        let hash2 = hash_token("test-token");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, "test-token");
    }

    #[test]
    fn test_verify_password() {
        let hash = hash_password("my-password");
        assert!(verify_password("my-password", &hash));
        assert!(!verify_password("wrong-password", &hash));
    }
}
