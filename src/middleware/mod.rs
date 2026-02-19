//! HTTP Middleware for WAS (WhatsApp Server)
//!
//! Production-ready middleware for request correlation, metrics, authentication, and security.

pub mod account;
pub mod auth;

use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use std::time::SystemTime;
use tracing::Instrument;

use crate::utils::logging::{log_request_metrics, CorrelationId, RequestMetrics};

// Re-export account middleware
pub use account::{account_middleware, CurrentAccount};

// Re-export auth middleware
pub use auth::{auth_middleware, AuthState};

/// Correlation ID middleware - adds correlation ID to all requests
pub async fn correlation_id_middleware(mut request: Request, next: Next) -> Response {
    let correlation_id = extract_or_generate_correlation_id(request.headers());

    // Add correlation ID to request extensions
    request.extensions_mut().insert(correlation_id.clone());

    // Create a span with correlation ID
    let span = tracing::info_span!(
        "request",
        correlation_id = %correlation_id.0,
        method = %request.method(),
        path = %request.uri().path(),
    );

    // Execute request within the span
    let response = async move { next.run(request).await }
        .instrument(span)
        .await;

    // Add correlation ID to response headers
    let mut response = response;
    if let Ok(header_value) = correlation_id.0.parse() {
        response
            .headers_mut()
            .insert("x-correlation-id", header_value);
    }

    response
}

/// Request metrics middleware - tracks request timing and logs metrics
pub async fn request_metrics_middleware(request: Request, next: Next) -> Response {
    let start_time = SystemTime::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Extract correlation ID from request extensions
    let correlation_id = request
        .extensions()
        .get::<CorrelationId>()
        .cloned()
        .unwrap_or_else(CorrelationId::new);

    // Get client IP
    let ip_address = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        });

    let response = next.run(request).await;

    let duration = SystemTime::now()
        .duration_since(start_time)
        .unwrap()
        .as_millis() as u64;

    let metrics = RequestMetrics {
        method,
        path,
        status_code: response.status().as_u16(),
        duration_ms: duration,
        user_agent,
        ip_address,
    };

    log_request_metrics(&metrics, &correlation_id);

    response
}

/// Security headers middleware
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Add security headers
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert("x-frame-options", "DENY".parse().unwrap());
    headers.insert("x-xss-protection", "1; mode=block".parse().unwrap());
    headers.insert(
        "referrer-policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    // CSP: allow self, data: for QR code images, and unsafe-inline for React styles
    headers.insert(
        "content-security-policy",
        "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'"
            .parse()
            .unwrap(),
    );

    response
}

fn extract_or_generate_correlation_id(headers: &HeaderMap) -> CorrelationId {
    headers
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| CorrelationId::from_string(s.to_string()))
        .unwrap_or_else(CorrelationId::new)
}
