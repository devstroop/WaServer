//! Instance Middleware
//!
//! Extracts X-Instance-Id header and adds the instance to request extensions.
//! Currently unused — all routes use path parameters for instance context.

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::services::{InstanceManager, WhatsAppInstance};

/// Request extension for the current instance
#[derive(Clone)]
pub struct CurrentInstance(pub Arc<WhatsAppInstance>);

/// Extract instance from X-Instance-Id header
/// Currently unused — all routes use path params with State<Arc<InstanceManager>>.
pub async fn instance_middleware(
    State(manager): State<Arc<InstanceManager>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path();

    // Routes that don't need instance context via header
    if !requires_account_header(path) {
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

    // Add instance to request extensions
    request.extensions_mut().insert(CurrentInstance(instance));

    Ok(next.run(request).await)
}

/// Check if a path requires the X-Instance-Id header
fn requires_account_header(path: &str) -> bool {
    // Chat and message operations still use header
    if path.starts_with("/api/v1/chats") || path.starts_with("/api/v1/messages") {
        return true;
    }

    false
}

/// Helper to extract CurrentInstance from request extensions
pub fn extract_account(request: &Request) -> Option<Arc<WhatsAppInstance>> {
    request
        .extensions()
        .get::<CurrentInstance>()
        .map(|ca| ca.0.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_account_header() {
        // Should require header (chat/message routes)
        assert!(requires_account_header("/api/v1/chats"));
        assert!(requires_account_header("/api/v1/chats/123"));
        assert!(requires_account_header("/api/v1/messages"));
        assert!(requires_account_header("/api/v1/messages/123"));

        // Should NOT require header (uses path params or no instance needed)
        assert!(!requires_account_header("/api/v1/instances/123/status"));
        assert!(!requires_account_header("/api/v1/instances/123/profile"));
        assert!(!requires_account_header("/api/v1/instances"));
        assert!(!requires_account_header("/api/v1/instances/business-1"));
        assert!(!requires_account_header("/health"));
        assert!(!requires_account_header("/api-docs"));
    }
}
