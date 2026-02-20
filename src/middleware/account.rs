//! Account Middleware
//!
//! Extracts X-Account-Id header and adds the account to request extensions.

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::services::{AccountManager, WhatsAppAccount};

/// Request extension for the current account
#[derive(Clone)]
pub struct CurrentAccount(pub Arc<WhatsAppAccount>);

/// Extract account from X-Account-Id header
/// Only applies to routes that require account context:
/// - /api/v1/account/* (WhatsApp account operations)
/// - /api/v1/chats/*
/// - /api/v1/messages/*
pub async fn account_middleware(
    State(manager): State<Arc<AccountManager>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let path = request.uri().path();

    // Routes that don't need account context
    // - /api/v1/auth/* (local JWT auth)
    // - /api/v1/accounts/* (account management - uses path param)
    // - /health, /api-docs, etc.
    if !requires_account_header(path) {
        return Ok(next.run(request).await);
    }

    // Extract X-Account-Id header
    let account_id = match headers.get("X-Account-Id").and_then(|v| v.to_str().ok()) {
        Some(id) if !id.is_empty() => id,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "missing_account_id",
                    "message": "This endpoint requires X-Account-Id header. Create an account first via POST /api/v1/accounts"
                })),
            )
                .into_response());
        }
    };

    // Get the account
    let account = match manager.get_account(account_id).await {
        Some(acc) => acc,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "account_not_found",
                    "message": format!("Account '{}' not found", account_id),
                    "account_id": account_id
                })),
            )
                .into_response());
        }
    };

    // Add account to request extensions
    request.extensions_mut().insert(CurrentAccount(account));

    Ok(next.run(request).await)
}

/// Check if a path requires the X-Account-Id header
fn requires_account_header(path: &str) -> bool {
    // Account operations (WhatsApp auth, profile, privacy)
    if path.starts_with("/api/v1/account/") || path == "/api/v1/account" {
        return true;
    }

    // Chat and message operations
    if path.starts_with("/api/v1/chats") || path.starts_with("/api/v1/messages") {
        return true;
    }

    false
}

/// Helper to extract CurrentAccount from request extensions
pub fn extract_account(request: &Request) -> Option<Arc<WhatsAppAccount>> {
    request
        .extensions()
        .get::<CurrentAccount>()
        .map(|ca| ca.0.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_account_header() {
        // Should require header
        assert!(requires_account_header("/api/v1/account/status"));
        assert!(requires_account_header("/api/v1/account/qr"));
        assert!(requires_account_header("/api/v1/account/profile"));
        assert!(requires_account_header("/api/v1/chats"));
        assert!(requires_account_header("/api/v1/chats/123"));
        assert!(requires_account_header("/api/v1/messages"));
        assert!(requires_account_header("/api/v1/messages/123"));

        // Should NOT require header
        assert!(!requires_account_header("/api/v1/auth/login"));
        assert!(!requires_account_header("/api/v1/auth/status"));
        assert!(!requires_account_header("/api/v1/accounts"));
        assert!(!requires_account_header("/api/v1/accounts/business-1"));
        assert!(!requires_account_header("/health"));
        assert!(!requires_account_header("/api-docs"));
    }
}
