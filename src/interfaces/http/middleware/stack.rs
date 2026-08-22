//! HTTP middleware stack — `tower::ServiceBuilder` + `TraceLayer`, `correlation_id`, metrics, CORS
//! Mirrors `bin/was.rs:361..369` but as a reusable function.

use axum::{extract::DefaultBodyLimit, http::Method, middleware, Router};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::middleware::{
    correlation_id_middleware, request_metrics_middleware, security_headers_middleware,
};

/// Build the global middleware stack as a `Router` layer.
/// Extracted so `bin/was.rs:406` becomes ~40 lines and router is unit-testable without browser.
pub fn http_middleware_stack(router: Router, max_upload_size: usize) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .allow_origin(Any);

    router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(middleware::from_fn(correlation_id_middleware))
            .layer(middleware::from_fn(request_metrics_middleware))
            .layer(middleware::from_fn(security_headers_middleware))
            .layer(cors)
            .layer(DefaultBodyLimit::max(max_upload_size)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_stack_wraps_router() {
        let app = Router::new().route("/ping", axum::routing::get(|| async { "pong" }));
        let wrapped = http_middleware_stack(app, 1024 * 1024);
        let _ = format!("{:?}", wrapped);
    }
}
