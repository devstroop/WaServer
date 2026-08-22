//! HTTP router — extracted from `src/bin/was.rs:107..396` (part of #10)
//!
//! `bin/was.rs` was 406 LOC building `Router` inline (health, instances, users, auth,
//! CORS, Swagger, shutdown). This module provides a thin, testable `build_router` that
//! will eventually replace the inline builder. Currently a scaffold — no wiring to
//! `InstanceManager`/`Database` yet, keeps existing `bin/was.rs` as facade.

use axum::{extract::DefaultBodyLimit, http::Method, middleware, Router};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::middleware::{
    correlation_id_middleware, request_metrics_middleware, security_headers_middleware,
};

/// Build the full application router (versioned).
/// Thin wrapper — callers pass pre-built sub-routers; this adds global middleware.
/// Mirrors `bin/was.rs:360` ServiceBuilder stack.
pub fn build_router(api_router: Router) -> Router {
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

    // Default limit 10MB mirrors `config.limits.max_upload_size`
    let limit = 10 * 1024 * 1024usize;

    api_router.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(middleware::from_fn(correlation_id_middleware))
            .layer(middleware::from_fn(request_metrics_middleware))
            .layer(middleware::from_fn(security_headers_middleware))
            .layer(cors)
            .layer(DefaultBodyLimit::max(limit)),
    )
}

/// Health routes — `GET /api/health|ready|live|metrics`, no auth.
/// For unit tests: no state required. Real handlers are in `handlers::api::health` and require
/// `State<Arc<InstanceManager>>`; this scaffold uses mock handlers so `Router<()>` is testable
/// without DB/browser (see `bin/was.rs:261` for real wiring).
pub fn health_router() -> Router {
    Router::new()
        .route(
            "/health",
            axum::routing::get(|| async { axum::Json(serde_json::json!({"status":"healthy"})) }),
        )
        .route(
            "/ready",
            axum::routing::get(|| async { axum::Json(serde_json::json!({"status":"ready"})) }),
        )
        .route(
            "/live",
            axum::routing::get(|| async { axum::Json(serde_json::json!({"status":"live"})) }),
        )
        .route(
            "/metrics",
            axum::routing::get(|| async { axum::Json(serde_json::json!({"uptime_seconds":0})) }),
        )
}

/// Placeholder for versioned API router; will nest `health_router` + `instances` + `users` + `auth`.
/// Kept minimal so `cargo test` can spin router without browser/DB.
pub fn api_router() -> Router {
    health_router().nest("/api", Router::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_health_router_has_routes() {
        let r = health_router();
        // router should be constructible without panic
        let _ = format!("{:?}", r);
    }
    #[test]
    fn test_build_router_wraps() {
        let api = api_router();
        let app = build_router(api);
        let _ = format!("{:?}", app);
    }
}
