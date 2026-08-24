//! HTTP router — extracted from `src/bin/was.rs:107..396` (part of #10)
//!
//! `bin/was.rs` was 406 LOC building `Router` inline (health, instances, users, auth,
//! CORS, Swagger, shutdown). This module provides a thin, testable `build_router` that
//! will eventually replace the inline builder. Currently a scaffold — no wiring to
//! `InstanceManager`/`Database` yet, keeps existing `bin/was.rs` as facade.

use std::sync::Arc;

use axum::{extract::DefaultBodyLimit, http::Method, middleware, Router};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    config::AppConfig,
    middleware::{
        auth_middleware, correlation_id_middleware, request_metrics_middleware,
        security_headers_middleware, AuthState,
    },
    services::{Database, InstanceManager},
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

/// Full router with all versioned routes, auth, Swagger, and global middleware.
/// Mirrors `bin/was.rs:252..396` but is unit-testable without `TcpListener`.
/// `bin/was.rs` will delegate to this, becoming ~40-line bootstrap.
pub fn build_full_router(
    config: Arc<AppConfig>,
    instance_manager: Arc<InstanceManager>,
    auth_db: Database,
) -> Router {
    use crate::api::{auth, health, instances, users, whatsapp};
    use crate::interfaces::http::dto::messaging::{SendMessageRequestDto, SendMessageResponseDto};
    use crate::interfaces::http::handlers::messaging;
    use crate::models::{auth::*, chat::ErrorResponse, instance::*, user::*};
    use axum::routing::{delete, get, post, put};
    use utoipa::{
        openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
        Modify, OpenApi,
    };

    #[derive(OpenApi)]
    #[openapi(
        paths(
            health::health_check,
            instances::list_instances, instances::create_instance, instances::get_instance, instances::delete_instance,
            instances::warmup_instance, instances::screenshot, instances::reset_instance, instances::get_instance_config, instances::update_instance_config,
            whatsapp::get_instance_status, whatsapp::get_qr_code, whatsapp::link_phone, whatsapp::unlink,
            messaging::send_message,
            users::list_users, users::create_user, users::get_user, users::update_user, users::delete_user,
            users::create_access_token, users::list_access_tokens, users::delete_access_token, users::get_user_instances, users::assign_instance, users::remove_instance, users::get_me,
            auth::register, auth::login, auth::logout, auth::logout_all, auth::validate,
        ),
        components(schemas(
            health::HealthResponse, health::ServiceHealth, health::StatusResponse,
            AuthStatusResponse, QrCodeResponse, PhoneLoginRequest, PhoneAuthResponse, SuccessResponse, ErrorResponse,
            SendMessageRequestDto, SendMessageResponseDto,
            CreateInstanceRequest, CreateInstanceResponse, InstanceListResponse, InstanceInfo, InstanceStatus,
            DeleteInstanceResponse, DeleteInstanceQuery, InstanceActionResponse, ListInstancesQuery, BrowserOverrides,
            InstanceConfig, InstanceBrowserConfig, InstanceRateLimits, UpdateInstanceConfigRequest, UpdateBrowserConfig, UpdateRateLimits,
            WhatsAppStatusResponse,
            UserRole, InstancePermission, UserInfo, InstanceOwnerRecord, CreateUserRequest, CreateUserResponse, UpdateUserRequest, AssignInstanceRequest, ListUsersResponse, UserInstancesResponse,
            AccessTokenInfo, CreateAccessTokenRequest, CreateAccessTokenResponse, ListAccessTokensResponse,
            RegisterUserRequest, LoginRequest, LoginResponse,
        )),
        modifiers(&SecurityAddon),
        tags(
            (name = "Health", description = "Server health and metrics endpoints"),
            (name = "Instances", description = "Instance lifecycle management (CRUD, start, stop, config)"),
            (name = "WhatsApp", description = "WhatsApp operations: linking"),
            (name = "Messaging", description = "Send messages"),
            (name = "Auth", description = "User authentication (register, login, logout)"),
            (name = "Users", description = "User management, access tokens, instance assignments")
        ),
        info(title = "WhatsApp Server - API", version = "0.4.0", description = "Minimal REST API for WhatsApp Web automation — sending messages only.")
    )]
    struct ApiDoc;
    struct SecurityAddon;
    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "bearer_auth",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
                );
            }
        }
    }

    let auth_state = AuthState::new(
        config.auth.secret_key.clone(),
        auth_db.clone(),
        config.auth.session_ttl_hours,
    );
    let health_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .route("/live", get(health::liveness_check))
        .route("/metrics", get(health::get_metrics))
        .with_state(instance_manager.clone());

    let instances_routes = Router::new()
        .route("/", get(instances::list_instances))
        .route("/", post(instances::create_instance))
        .route("/:instance_id", get(instances::get_instance))
        .route("/:instance_id", delete(instances::delete_instance))
        .route("/:instance_id/warmup", post(instances::warmup_instance))
        .route("/:instance_id/reset", delete(instances::reset_instance))
        .route("/:instance_id/screenshot", get(instances::screenshot))
        .route("/:instance_id/config", get(instances::get_instance_config))
        .route(
            "/:instance_id/config",
            put(instances::update_instance_config),
        )
        .route("/:instance_id/status", get(whatsapp::get_instance_status))
        .route("/:instance_id/link/qr", get(whatsapp::get_qr_code))
        .route("/:instance_id/link/phone", post(whatsapp::link_phone))
        .route("/:instance_id/unlink", delete(whatsapp::unlink))
        .route("/:instance_id/send", post(messaging::send_message))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .with_state(instance_manager.clone());

    let users_routes = Router::new()
        .route("/me", get(users::get_me))
        .route("/", get(users::list_users))
        .route("/", post(users::create_user))
        .route("/:user_id", get(users::get_user))
        .route("/:user_id", axum::routing::patch(users::update_user))
        .route("/:user_id", delete(users::delete_user))
        .route("/:user_id/tokens", get(users::list_access_tokens))
        .route("/:user_id/tokens", post(users::create_access_token))
        .route(
            "/:user_id/tokens/:token_id",
            delete(users::delete_access_token),
        )
        .route("/assign-instance", post(users::assign_instance))
        .route("/:user_id/instances", get(users::get_user_instances))
        .route(
            "/:user_id/instances/:instance_id",
            delete(users::remove_instance),
        )
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .with_state(auth_db.clone());

    let auth_routes = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/logout-all", post(auth::logout_all))
        .route("/validate", get(auth::validate))
        .with_state(auth_state.clone());

    // Web admin UI (#28) — cookie-session pages under /app + embedded assets
    let web_auth_state = crate::middleware::auth::AuthState::new(
        config.auth.secret_key.clone(),
        auth_db.clone(),
        config.auth.session_ttl_hours,
    );

    let mut app = Router::new();
    app = app.nest("/api", health_routes);
    app = app.nest("/api/v1/instances", instances_routes);
    app = app.nest("/api/v1/auth", auth_routes);
    app = app.nest("/api/v1/users", users_routes);
    app = app.nest(
        "/app",
        crate::interfaces::http::web::router(web_auth_state, instance_manager.clone()),
    );
    app = app.nest(
        "/assets/web",
        crate::interfaces::http::web::assets::router(),
    );

    if config.swagger.enabled {
        use utoipa_swagger_ui::SwaggerUi;
        let swagger_path = config.swagger.path.clone();
        app = app
            .merge(SwaggerUi::new(swagger_path).url("/api-docs/openapi.json", ApiDoc::openapi()));
    }

    // Global middleware — CORS honors [cors] config (wildcard/empty = permissive)
    let cors = crate::interfaces::http::middleware::stack::build_cors_layer(&config.cors);
    crate::interfaces::http::middleware::stack::build_stack(
        app,
        cors,
        config.limits.max_upload_size,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_health_router_has_routes() {
        let r = health_router();
        let _ = format!("{:?}", r);
    }
    #[test]
    fn test_build_router_wraps() {
        let api = api_router();
        let app = build_router(api);
        let _ = format!("{:?}", app);
    }
    #[test]
    fn test_build_full_router_mock() {
        // Build with mock config/manager/db — will use in-memory SQLite and dummy InstanceManager
        // Just verify it constructs without panic; handlers are not invoked.
        let config = Arc::new(AppConfig::default());
        let db = {
            let dir = std::env::temp_dir().join(format!("test-router-{}", uuid::Uuid::new_v4()));
            let _ = std::fs::create_dir_all(&dir);
            crate::services::Database::open(&dir).unwrap()
        };
        let manager = Arc::new(crate::services::InstanceManager::new(
            config.clone(),
            db.clone(),
        ));
        let app = build_full_router(config, manager, db);
        let _ = format!("{:?}", app);
    }
}
