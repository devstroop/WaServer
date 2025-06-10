use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;

/// API Key authentication middleware
pub async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get API token from app state (we'll need to modify this to access config)
    let expected_token = std::env::var("WHATSAPP_AUTH__API_TOKEN")
        .unwrap_or_else(|_| "your-secure-api-token-change-this".to_string());

    // Extract the Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok());

    if let Some(auth_value) = auth_header {
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            if token == expected_token {
                // Token is valid, proceed with the request
                return Ok(next.run(request).await);
            }
        }
    }

    // Return 401 Unauthorized if token is missing or invalid
    Err(StatusCode::UNAUTHORIZED)
}

/// Extract API token from Authorization header
pub fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth_value| auth_value.strip_prefix("Bearer "))
        .map(|token| token.to_string())
}

/// Validate API token against configured token
pub fn validate_token(provided_token: &str, expected_token: &str) -> bool {
    provided_token == expected_token
}
