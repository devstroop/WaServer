//! WAS - WhatsApp Server
//!
//! Minimal REST API server for WhatsApp Web automation (sending only).
//!
//! ## Usage
//!
//! ```bash
//! # Run with defaults
//! was
//!
//! # With environment variable for secret key
//! WAS__AUTH__SECRET_KEY=mysecret was
//! ```

use std::sync::Arc;
use tracing::info;

use was::{
    config::AppConfig,
    services::{Database, InstanceManager},
    utils::logging,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Failed to load configuration: {}", e);
        std::process::exit(1);
    });

    // Setup logging
    logging::init_logging(&config.environment).unwrap_or_else(|e| {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    });

    print_banner();

    // Initialize health check uptime counter
    was::handlers::api::health::init();

    info!("Starting WAS (WhatsApp Server) v{}", VERSION);

    // Validate configuration
    if let Err(e) = config.validate() {
        if config.environment.is_development() {
            tracing::warn!(
                "⚠️  Config validation: {} (continuing in development mode)",
                e
            );
        } else {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    }

    info!(
        "Server listening on {}:{}",
        config.server.host, config.server.port
    );

    let config = Arc::new(config);

    // Initialize embedded SQLite database
    let db_data_dir = config
        .instances
        .as_ref()
        .and_then(|ac| ac.base_directory.clone())
        .unwrap_or_else(|| {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| "/tmp".to_string());
            std::path::PathBuf::from(home).join(".was")
        });
    let db = Database::open(&db_data_dir).unwrap_or_else(|e| {
        eprintln!("Failed to open database: {}", e);
        std::process::exit(1);
    });

    // Clone db for auth middleware (RBAC user lookup)
    let auth_db = db.clone();

    // Initialize InstanceManager for multi-instance support
    let instance_manager = Arc::new(InstanceManager::new(config.clone(), db));

    // Discover existing instances from filesystem
    match instance_manager.discover_instances().await {
        Ok((discovered, _)) => {
            if !discovered.is_empty() {
                info!("📂 Discovered {} existing instance(s)", discovered.len());
            }
        }
        Err(e) => {
            tracing::warn!("Failed to discover existing instances: {}", e);
        }
    }

    info!("🔑 Multi-instance support enabled (v{})", VERSION);

    // Run server
    run_server(config, instance_manager, auth_db).await
}

async fn run_server(
    config: Arc<AppConfig>,
    instance_manager: Arc<InstanceManager>,
    auth_db: Database,
) -> Result<(), Box<dyn std::error::Error>> {
    use axum::{
        extract::DefaultBodyLimit,
        http::Method,
        middleware,
        routing::{delete, get, post, put},
        Router,
    };
    use tower::ServiceBuilder;
    use tower_http::{
        cors::{Any, CorsLayer},
        trace::TraceLayer,
    };
    use utoipa::{
        openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
        Modify, OpenApi,
    };
    use was::{
        api::{auth, chat, health, instances, users, whatsapp},
        middleware::{
            auth_middleware, correlation_id_middleware, request_metrics_middleware,
            security_headers_middleware, AuthState,
        },
        models::{auth::*, chat::SendMessageRequest, chat::SendMessageResponse, chat::ErrorResponse, instance::*, user::*},
    };

    // CORS
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

    // OpenAPI documentation
    #[derive(OpenApi)]
    #[openapi(
        paths(
            // Health
            health::health_check,
            // Instances management
            instances::list_instances,
            instances::create_instance,
            instances::get_instance,
            instances::delete_instance,
            instances::warmup_instance,
            instances::screenshot,
            instances::reset_instance,
            instances::get_instance_config,
            instances::update_instance_config,
            // WhatsApp operations (uses path param)
            whatsapp::get_instance_status,
            whatsapp::get_qr_code,
            whatsapp::link_phone,
            whatsapp::unlink,

            // Messaging (send only)
            chat::send_message,

            // Users management
            users::list_users,
            users::create_user,
            users::get_user,
            users::update_user,
            users::delete_user,
            users::create_access_token,
            users::list_access_tokens,
            users::delete_access_token,
            users::get_user_instances,
            users::assign_instance,
            users::remove_instance,
            users::get_me,
            // Authentication
            auth::register,
            auth::login,
            auth::logout,
            auth::validate,
        ),
        components(
            schemas(
                // Health
                health::HealthResponse, health::ServiceHealth, health::StatusResponse,
                // Auth
                AuthStatusResponse, QrCodeResponse, PhoneLoginRequest, PhoneAuthResponse, SuccessResponse, ErrorResponse,
                // Chat (send only)
                SendMessageRequest, SendMessageResponse,
                // Instance management
                CreateInstanceRequest, CreateInstanceResponse, InstanceListResponse, InstanceInfo, InstanceStatus,
                DeleteInstanceResponse, DeleteInstanceQuery, InstanceActionResponse, ListInstancesQuery, BrowserOverrides,
                // Instance configuration
                InstanceConfig, InstanceBrowserConfig, InstanceRateLimits,
                UpdateInstanceConfigRequest, UpdateBrowserConfig, UpdateRateLimits,
                // WhatsApp operations
                WhatsAppStatusResponse,
                // Users
                UserRole, InstancePermission, UserInfo, InstanceOwnerRecord,
                CreateUserRequest, CreateUserResponse, UpdateUserRequest,
                AssignInstanceRequest, ListUsersResponse, UserInstancesResponse,
                // Access Tokens
                AccessTokenInfo, CreateAccessTokenRequest, CreateAccessTokenResponse, ListAccessTokensResponse,
                // Web Authentication
                RegisterUserRequest, LoginRequest, LoginResponse,
            )
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "Health", description = "Server health and metrics endpoints"),
            (name = "Instances", description = "Instance lifecycle management (CRUD, start, stop, config)"),
            (name = "WhatsApp", description = "WhatsApp operations: linking"),
            (name = "Messaging", description = "Send messages"),
            (name = "Auth", description = "User authentication (register, login, logout)"),
            (name = "Users", description = "User management, access tokens, instance assignments")
        ),
        info(
            title = "WhatsApp Server - API",
            version = "0.3.0",
            description = "Minimal REST API for WhatsApp Web automation — sending messages only.",
        )
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap();
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            );
        }
    }

    // Create auth state for middleware (secret key + database for RBAC)
    let auth_state = AuthState::new(config.auth.secret_key.clone(), auth_db);

    // Start building the app
    let mut app = Router::new();

    // MCP endpoints (removed — waserver is send-only, no MCP)

    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .route("/live", get(health::liveness_check))
        .route("/metrics", get(health::get_metrics))
        .with_state(instance_manager.clone());

    // All instance routes (management + WhatsApp ops) under one namespace
    let instances_routes = Router::new()
        // Instance CRUD
        .route("/", get(instances::list_instances))
        .route("/", post(instances::create_instance))
        .route("/:instance_id", get(instances::get_instance))
        .route("/:instance_id", delete(instances::delete_instance))
        // Instance lifecycle
        .route("/:instance_id/warmup", post(instances::warmup_instance))
        .route("/:instance_id/reset", delete(instances::reset_instance))
        .route("/:instance_id/screenshot", get(instances::screenshot))
        .route("/:instance_id/config", get(instances::get_instance_config))
        .route(
            "/:instance_id/config",
            put(instances::update_instance_config),
        )
        // WhatsApp auth & linking
        .route("/:instance_id/status", get(whatsapp::get_instance_status))
        .route("/:instance_id/link/qr", get(whatsapp::get_qr_code))
        .route("/:instance_id/link/phone", post(whatsapp::link_phone))
        .route("/:instance_id/unlink", delete(whatsapp::unlink))
        // Messaging (send only)
        .route("/:instance_id/send", post(chat::send_message))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .with_state(instance_manager.clone());

    // Mount health routes at /api (no auth required)
    app = app.nest("/api", health_routes);

    // Mount all v1 routes
    app = app.nest("/api/v1/instances", instances_routes);

    // User management routes (admin only for most)
    let users_routes = Router::new()
        // Self-info endpoint (must come before /:user_id to avoid conflict)
        .route("/me", get(users::get_me))
        // User CRUD
        .route("/", get(users::list_users))
        .route("/", post(users::create_user))
        .route("/:user_id", get(users::get_user))
        .route("/:user_id", axum::routing::patch(users::update_user))
        .route("/:user_id", delete(users::delete_user))
        // Access token management
        .route("/:user_id/tokens", get(users::list_access_tokens))
        .route("/:user_id/tokens", post(users::create_access_token))
        .route("/:user_id/tokens/:token_id", delete(users::delete_access_token))
        // Instance assignments
        .route("/assign-instance", post(users::assign_instance))
        .route("/:user_id/instances", get(users::get_user_instances))
        .route("/:user_id/instances/:instance_id", delete(users::remove_instance))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .with_state(auth_state.db.clone());

    // Auth routes (public, no auth required)
    let auth_routes = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/validate", get(auth::validate))
        .with_state(auth_state.db.clone());

    app = app.nest("/api/v1/auth", auth_routes);
    app = app.nest("/api/v1/users", users_routes);

    info!("📖 API at /api/v1/instances");
    info!("👤 Users API at /api/v1/users");
    info!("🔐 Auth API at /api/v1/auth");

    // Swagger UI documentation (configurable)
    if config.swagger.enabled {
        use utoipa_swagger_ui::SwaggerUi;

        info!("📚 Swagger UI at {}", config.swagger.path);
        let swagger_path = config.swagger.path.clone();
        app = app
            .merge(SwaggerUi::new(swagger_path).url("/api-docs/openapi.json", ApiDoc::openapi()));
    } else {
        info!("📚 Swagger UI disabled (set swagger.enabled = true to enable)");
    }

    // Middleware
    let app = app.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(middleware::from_fn(correlation_id_middleware))
            .layer(middleware::from_fn(request_metrics_middleware))
            .layer(middleware::from_fn(security_headers_middleware))
            .layer(cors)
            .layer(DefaultBodyLimit::max(config.limits.max_upload_size)),
    );

    // Listen
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
            .await?;

    info!(
        "🚀 WAS running at http://{}:{}",
        config.server.host, config.server.port
    );

    // Graceful shutdown
    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        info!("🛑 Shutting down...");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;

    Ok(())
}

fn print_banner() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║   ██╗    ██╗ █████╗ ███████╗                              ║
║   ██║    ██║██╔══██╗██╔════╝                              ║
║   ██║ █╗ ██║███████║███████╗                              ║
║   ██║███╗██║██╔══██║╚════██║                              ║
║   ╚███╔███╔╝██║  ██║███████║                              ║
║    ╚══╝╚══╝ ╚═╝  ╚═╝╚══════╝                              ║
║                                                           ║
║   ╔═══════════════════════════════════════════════════╗   ║
║   ║  WhatsApp Server   v{:<28}║   ║
║   ╚═══════════════════════════════════════════════════╝   ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
"#,
        VERSION
    );
}
