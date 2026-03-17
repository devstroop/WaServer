//! Instance Middleware and Permission Checking
//!
//! Provides utilities for instance permission checking with RBAC support.
//! Includes middleware for X-Instance-Id header extraction (legacy, unused)
//! and permission checking utilities for path-based instance access.

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{
    models::{auth::AuthenticatedUser, user::InstancePermission},
    services::{Database, InstanceManager, InstanceService},
};

/// Request extension for the current instance
#[derive(Clone)]
pub struct CurrentInstance(pub Arc<InstanceService>);

/// Instance access state for permission checking
#[derive(Clone)]
pub struct InstanceAccessState {
    pub db: Database,
}

/// Check if a user has permission to access an instance.
/// Returns Ok(()) if access is granted, Err(Response) if denied.
///
/// Access rules:
/// - Superadmin (secret key): full access to all instances
/// - Admin role: full access to all instances
/// - User role: must have explicit permission assigned
pub fn check_instance_access(
    auth_user: &AuthenticatedUser,
    instance_id: &str,
    required_permission: InstancePermission,
    db: &Database,
) -> Result<(), Response> {
    // Superadmin has full access
    if auth_user.is_superadmin() {
        return Ok(());
    }

    // Admin role has full access
    if auth_user.is_admin() {
        return Ok(());
    }

    // For regular users, check instance_owner table
    if let Some(user_id) = auth_user.user_id() {
        match db.get_instance_permission(user_id, instance_id) {
            Ok(Some(permission)) => {
                if auth_user.can_access_instance(Some(permission), required_permission) {
                    return Ok(());
                }
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "insufficient_permission",
                        "message": format!(
                            "You need '{}' permission for instance '{}'",
                            required_permission, instance_id
                        )
                    })),
                )
                    .into_response());
            }
            Ok(None) => {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "no_access",
                        "message": format!("You don't have access to instance '{}'", instance_id)
                    })),
                )
                    .into_response());
            }
            Err(_) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "permission_check_failed",
                        "message": "Failed to check instance permissions"
                    })),
                )
                    .into_response());
            }
        }
    }

    Err((
        StatusCode::FORBIDDEN,
        Json(json!({
            "error": "access_denied",
            "message": "Access denied"
        })),
    )
        .into_response())
}

/// Extract account from X-Instance-Id header
/// Currently unused — all routes use path params with State<Arc<InstanceManager>>.
pub async fn instance_middleware(
    State(manager): State<Arc<InstanceManager>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path();

    // Routes that don't need instance context via header
    if !requires_instance_header(path) {
        return Ok(next.run(request).await);
    }

    // Extract X-Instance-Id header
    let instance_id = match headers.get("X-Instance-Id").and_then(|v| v.to_str().ok()) {
        Some(id) if !id.is_empty() => id,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "missing_instance_id",
                    "message": "This endpoint requires X-Instance-Id header. Create an instance first via POST /api/v1/instances"
                })),
            )
                .into_response());
        }
    };

    // Get the instance
    let instance = match manager.get_instance(instance_id).await {
        Some(acc) => acc,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "instance_not_found",
                    "message": format!("Instance '{}' not found", instance_id),
                    "instance_id": instance_id
                })),
            )
                .into_response());
        }
    };

    // Add account to request extensions
    request.extensions_mut().insert(CurrentInstance(instance));

    Ok(next.run(request).await)
}

/// Check if a path requires the X-Instance-Id header
fn requires_instance_header(path: &str) -> bool {
    // Chat and message operations still use header
    if path.starts_with("/api/v1/chats") || path.starts_with("/api/v1/messages") {
        return true;
    }

    false
}

/// Helper to extract CurrentInstance from request extensions
pub fn extract_account(request: &Request) -> Option<Arc<InstanceService>> {
    request
        .extensions()
        .get::<CurrentInstance>()
        .map(|ca| ca.0.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_instance_header() {
        // Should require header (chat/message routes)
        assert!(requires_instance_header("/api/v1/chats"));
        assert!(requires_instance_header("/api/v1/chats/123"));
        assert!(requires_instance_header("/api/v1/messages"));
        assert!(requires_instance_header("/api/v1/messages/123"));

        // Should NOT require header (uses path params or no account needed)
        assert!(!requires_instance_header("/api/v1/accounts/123/status"));
        assert!(!requires_instance_header("/api/v1/accounts/123/profile"));
        assert!(!requires_instance_header("/api/v1/accounts"));
        assert!(!requires_instance_header("/api/v1/accounts/business-1"));
        assert!(!requires_instance_header("/health"));
        assert!(!requires_instance_header("/api-docs"));
    }
}
