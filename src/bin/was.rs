//! WAS - WhatsApp Server
//!
//! REST API and MCP server for WhatsApp Web automation.
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

use was::{config::AppConfig, services::InstanceManager, utils::logging};

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
            tracing::warn!("⚠️  Config validation: {} (continuing in development mode)", e);
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

    // Initialize InstanceManager for multi-instance support
    let instance_manager = Arc::new(InstanceManager::new(config.clone()));

    // Discover existing instances from filesystem
    match instance_manager.discover_instances().await {
        Ok(discovered) => {
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
    run_server(config, instance_manager).await
}

async fn run_server(
    config: Arc<AppConfig>,
    instance_manager: Arc<InstanceManager>,
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
        api::{whatsapp, chat, health, instances},
        middleware::{
            auth_middleware, correlation_id_middleware, request_metrics_middleware,
            security_headers_middleware, AuthState,
        },
        models::{instance::*, auth::*, chat::*},
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
            instances::start_instance,
            instances::stop_instance,
            instances::discover_instances,
            instances::screenshot,
            instances::get_instance_config,
            instances::update_instance_config,
            // WhatsApp operations (uses path param)
            whatsapp::get_instance_status,
            whatsapp::get_qr_code,
            whatsapp::link_phone,
            whatsapp::unlink,
            whatsapp::get_profile,
            whatsapp::update_profile,
            whatsapp::get_privacy,
            whatsapp::update_privacy,
            // Chat
            chat::list_chats,
            chat::get_chat_messages,
            chat::watch_messages,
            chat::send_message,
            chat::get_message,
        ),
        components(
            schemas(
                // Health
                health::HealthResponse, health::ServiceHealth, health::StatusResponse,
                // Auth
                AuthStatusResponse, QrCodeResponse, PhoneLoginRequest, PhoneAuthResponse, SuccessResponse, ErrorResponse,
                // Chat
                SendMessageRequest, SendMessageResponse, ChatListResponse, ChatInfo, Message, MessageInfo, MessageListResponse, MessageQueryParams,
                // Instance management
                CreateInstanceRequest, CreateInstanceResponse, InstanceListResponse, InstanceInfo, InstanceStatus,
                DeleteInstanceResponse, DeleteInstanceQuery, InstanceActionResponse, ListInstancesQuery, BrowserOverrides,
                // Instance configuration
                InstanceConfig, InstanceBrowserConfig, InstanceWebhookConfig, WebhookEndpoint, InstanceRateLimits,
                UpdateInstanceConfigRequest, UpdateBrowserConfig, UpdateWebhookConfig, UpdateRateLimits,
                // Instance operations
                WhatsAppStatusResponse, PhoneLinkRequest, ProfileInfo, PrivacySettings,
                UpdateProfileRequest, UpdatePrivacyRequest,
                PrivacyVisibility, OnlineVisibility, GroupAddPermission,
            )
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "Health", description = "Server health and metrics endpoints"),
            (name = "Instances", description = "Instance lifecycle management (CRUD, start, stop, config)"),
            (name = "WhatsApp", description = "WhatsApp operations: linking, profile, privacy"),
            (name = "Messaging", description = "Chat and message operations")
        ),
        info(
            title = "WhatsApp Server - API",
            version = "0.3.0",
            description = "REST API for WhatsApp Web automation with multi-instance support.",
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

    // Create auth state for middleware (secret key only)
    let auth_state = AuthState::new(config.auth.secret_key.clone());

    // Start building the app
    let mut app = Router::new();

    // MCP endpoints (feature-gated and config-enabled)
    #[cfg(feature = "mcp")]
    if config.mcp.enabled {
        use was::api::mcp;
        info!("🤖 MCP enabled at {}", config.mcp.endpoint);
        app = app.nest(
            &config.mcp.endpoint,
            mcp::mcp_routes(instance_manager.clone()),
        );
    }

    #[cfg(feature = "mcp")]
    if !config.mcp.enabled {
        info!("🤖 MCP disabled (set mcp.enabled = true to enable)");
    }

    #[cfg(not(feature = "mcp"))]
    info!("🤖 MCP not compiled (build with --features mcp to enable)");

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
        .route("/discover", post(instances::discover_instances))
        .route("/:instance_id", get(instances::get_instance))
        .route("/:instance_id", delete(instances::delete_instance))
        // Instance lifecycle
        .route("/:instance_id/start", post(instances::start_instance))
        .route("/:instance_id/stop", post(instances::stop_instance))
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
        // Profile & privacy
        .route(
            "/:instance_id/profile",
            get(whatsapp::get_profile).put(whatsapp::update_profile),
        )
        .route(
            "/:instance_id/privacy",
            get(whatsapp::get_privacy).put(whatsapp::update_privacy),
        )
        // Chats & messages
        .route("/:instance_id/chats", get(chat::list_chats))
        .route("/:instance_id/chats/events", get(chat::watch_messages))
        .route("/:instance_id/chats/:chat_id", get(chat::get_chat_messages))
        .route("/:instance_id/messages", post(chat::send_message))
        .route("/:instance_id/messages/:message_id", get(chat::get_message))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .with_state(instance_manager.clone());

    // Mount health routes at /api (no auth required)
    app = app.nest("/api", health_routes);

    // Mount all v1 routes
    app = app.nest("/api/v1/instances", instances_routes);

    info!("📖 API at /api/v1/instances");

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
