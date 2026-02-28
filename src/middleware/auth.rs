//! Authentication Middleware
//!
//! Provides API authentication via static secret key (superadmin) and user API keys.
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
    models::auth::AuthenticatedUser,
    services::Database,
    utils::logging::CorrelationId,
};

/// Authentication state for middleware
#[derive(Clone)]
pub struct AuthState {
    /// Static secret key for superadmin authentication
    pub secret_key: String,
    /// Database for user API key lookup
    pub db: Database,
}

impl AuthState {
    /// Create new auth state
    pub fn new(secret_key: String, db: Database) -> Self {
        Self { secret_key, db }
    }
}

/// Hash an API key for secure comparison/storage.
pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Authentication middleware
///
/// Validates Bearer tokens for protected endpoints.
/// Checks in order:
/// 1. Static secret key (superadmin access)
/// 2. User API key from database
///
/// On successful authentication, adds `AuthenticatedUser` to request extensions.
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
            // 1. Check static secret key (superadmin)
            if token == auth_state.secret_key {
                let authenticated_user = AuthenticatedUser::Secret;
                tracing::debug!(
                    correlation_id = %correlation_id.0,
                    path = %path,
                    auth_method = "superadmin",
                    "Superadmin authentication successful"
                );
                request.extensions_mut().insert(authenticated_user);
                return Ok(next.run(request).await);
            }

            // 2. Check user API key
            let api_key_hash = hash_api_key(token);
            if let Ok(Some(user)) = auth_state.db.get_user_by_api_key(&api_key_hash) {
                if user.is_active {
                    let authenticated_user = AuthenticatedUser::User {
                        id: user.id.clone(),
                        username: user.username.clone(),
                        role: user.role,
                    };
                    tracing::debug!(
                        correlation_id = %correlation_id.0,
                        path = %path,
                        auth_method = "api_key",
                        user_id = %user.id,
                        username = %user.username,
                        role = %user.role,
                        "User authentication successful"
                    );
                    request.extensions_mut().insert(authenticated_user);
                    return Ok(next.run(request).await);
                } else {
                    tracing::warn!(
                        correlation_id = %correlation_id.0,
                        path = %path,
                        user_id = %user.id,
                        "Authentication failed - user is inactive"
                    );
                }
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
    fn test_hash_api_key() {
        let key = "test-api-key-12345";
        let hash = hash_api_key(key);
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex chars
        
        // Same key should produce same hash
        assert_eq!(hash, hash_api_key(key));
        
        // Different key should produce different hash
        assert_ne!(hash, hash_api_key("different-key"));
    }
}
