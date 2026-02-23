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
//! # With environment variables
//! WHATSAPP_PORT=8080 WHATSAPP_SECRET=mysecret was
//! ```

use std::sync::Arc;
use tracing::info;

use was::{
    config::AppConfig,
    services::AccountManager,
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

    info!("Starting WAS (WhatsApp Server) v{}", VERSION);
    info!(
        "Server listening on {}:{}",
        config.server.host, config.server.port
    );

    let config = Arc::new(config);

    // Initialize AccountManager for multi-account support
    let account_manager = Arc::new(AccountManager::new(config.clone()));

    // Discover existing accounts from filesystem
    match account_manager.discover_accounts().await {
        Ok(discovered) => {
            if !discovered.is_empty() {
                info!("📂 Discovered {} existing account(s)", discovered.len());
            }
        }
        Err(e) => {
            tracing::warn!("Failed to discover existing accounts: {}", e);
        }
    }

    info!("🔑 Multi-account support enabled (v{})", VERSION);

    // Run server
    run_server(config, account_manager).await
}

async fn run_server(
    config: Arc<AppConfig>,
    account_manager: Arc<AccountManager>,
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
        api::{account, instances, chat, health},
        middleware::{
            auth_middleware, correlation_id_middleware,
            request_metrics_middleware, security_headers_middleware, AuthState,
        },
        models::{auth::*, chat::*, account::*},
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
            account::get_account_status,
            account::get_qr_code,
            account::link_phone,
            account::unlink,
            account::get_profile,
            account::update_profile,
            account::get_privacy,
            account::update_privacy,
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
                health::HealthResponse, health::ServiceHealth,
                // Auth
                AuthStatusResponse, QrCodeResponse, PhoneLoginRequest, PhoneAuthResponse, SuccessResponse, ErrorResponse,
                // Chat
                SendMessageRequest, SendMessageResponse, ChatListResponse, ChatInfo, Message, MessageInfo, MessageListResponse, MessageQueryParams,
                // Account management
                CreateAccountRequest, CreateAccountResponse, AccountListResponse, AccountInfo, AccountStatus,
                DeleteAccountResponse, DeleteAccountQuery, AccountActionResponse, ListAccountsQuery,
                // Instance configuration
                InstanceConfig, InstanceBrowserConfig, InstanceWebhookConfig, WebhookEndpoint, InstanceRateLimits,
                UpdateInstanceConfigRequest, UpdateBrowserConfig, UpdateWebhookConfig, UpdateRateLimits,
                // Account operations
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
            description = "REST API for WhatsApp Web automation with multi-account support.",
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
            mcp::mcp_routes(account_manager.clone()),
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
        .with_state(account_manager.clone());

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
        .route("/:instance_id/status", get(account::get_account_status))
        .route("/:instance_id/link/qr", get(account::get_qr_code))
        .route("/:instance_id/link/phone", post(account::link_phone))
        .route("/:instance_id/unlink", delete(account::unlink))
        // Profile & privacy
        .route(
            "/:instance_id/profile",
            get(account::get_profile).put(account::update_profile),
        )
        .route(
            "/:instance_id/privacy",
            get(account::get_privacy).put(account::update_privacy),
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
        .with_state(account_manager.clone());

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
