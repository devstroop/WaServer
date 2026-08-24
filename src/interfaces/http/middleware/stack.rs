//! HTTP middleware stack — `tower::ServiceBuilder` + `TraceLayer`, `correlation_id`, metrics
//! Mirrors `bin/was.rs:361..369` but as a reusable function.

use axum::{extract::DefaultBodyLimit, http::Method, middleware, Router};
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    middleware::{
        correlation_id_middleware, request_metrics_middleware, security_headers_middleware,
    },
    models::config::CorsConfig,
};

/// Build the CORS layer from config.
/// Empty origins or a `"*"` entry → permissive (Any). Otherwise the exact
/// origin list is enforced. Unparseable entries are skipped with a warning.
pub fn build_cors_layer(cors: &CorsConfig) -> CorsLayer {
    let permissive =
        cors.allow_origins.is_empty() || cors.allow_origins.iter().any(|o| o.trim() == "*");
    let list: Vec<_> = if permissive {
        Vec::new()
    } else {
        cors.allow_origins
            .iter()
            .filter_map(|o| match o.trim().parse() {
                Ok(v) => Some(v),
                Err(_) => {
                    tracing::warn!(origin = %o, "cors.allow_origins entry unparseable — skipped");
                    None
                }
            })
            .collect()
    };

    let origin = if list.is_empty() {
        AllowOrigin::any()
    } else {
        AllowOrigin::list(list)
    };

    let methods: Vec<Method> = cors
        .allow_methods
        .iter()
        .filter_map(|m| m.to_uppercase().parse().ok())
        .collect();
    let methods = if methods.is_empty() {
        vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ]
    } else {
        methods
    };

    CorsLayer::new()
        .allow_origin(origin)
        .allow_methods(methods)
        .allow_headers(Any)
}

/// Build the global middleware stack as a `Router` layer.
/// Extracted so `bin/was.rs:406` becomes ~40 lines and router is unit-testable without browser.
pub fn http_middleware_stack(router: Router, max_upload_size: usize) -> Router {
    // Default config = permissive ("*") — production deployments should set
    // [cors] allow_origins explicitly.
    let cors_config = CorsConfig::default();
    build_stack(router, build_cors_layer(&cors_config), max_upload_size)
}

/// Stack variant with an explicit CORS layer (used by `build_full_router`)
pub fn build_stack(router: Router, cors: CorsLayer, max_upload_size: usize) -> Router {
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

    #[test]
    fn test_cors_permissive_on_wildcard() {
        let cfg = CorsConfig {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["GET".into()],
            allow_headers: vec![],
        };
        let _ = format!("{:?}", build_cors_layer(&cfg));
    }

    #[test]
    fn test_cors_specific_origins_parse() {
        let cfg = CorsConfig {
            allow_origins: vec!["https://admin.example.com".to_string()],
            allow_methods: vec!["GET".into(), "POST".into(), "bogus".into()],
            allow_headers: vec!["authorization".into()],
        };
        let _ = format!("{:?}", build_cors_layer(&cfg));
    }
}
