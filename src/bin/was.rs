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

use was::{config::AppConfig, services::{AccountManager, Database}, utils::logging};

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

    // Initialize embedded SurrealDB
    let db_data_dir = config
        .accounts
        .as_ref()
        .and_then(|ac| ac.base_directory.clone())
        .unwrap_or_else(|| {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| "/tmp".to_string());
            std::path::PathBuf::from(home).join(".was")
        });
    let db = Database::open(&db_data_dir).await.unwrap_or_else(|e| {
        eprintln!("Failed to open database: {}", e);
        std::process::exit(1);
    });

    // Initialize AccountManager for multi-account support
    let account_manager = Arc::new(AccountManager::new(config.clone(), db));

    // Discover existing accounts from filesystem
    match account_manager.discover_accounts().await {
        Ok((discovered, _)) => {
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
        api::{whatsapp, chat, health, accounts},
        middleware::{
            auth_middleware, correlation_id_middleware, request_metrics_middleware,
            security_headers_middleware, AuthState,
        },
        models::{account::*, auth::*, chat::*},
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
            // Accounts management
            accounts::list_accounts,
            accounts::create_account,
            accounts::get_account,
            accounts::delete_account,
            accounts::start_account,
            accounts::stop_account,
            accounts::screenshot,
            accounts::get_account_config,
            accounts::update_account_config,
            // WhatsApp operations (uses path param)
            whatsapp::get_account_status,
            whatsapp::get_qr_code,
            whatsapp::link_phone,
            whatsapp::unlink,
            whatsapp::get_profile,
            whatsapp::update_profile,

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
                // Account management
                CreateAccountRequest, CreateAccountResponse, AccountListResponse, AccountInfo, AccountStatus,
                DeleteAccountResponse, DeleteAccountQuery, AccountActionResponse, ListAccountsQuery, BrowserOverrides,
                // Account configuration
                AccountConfig, AccountBrowserConfig, AccountWebhookConfig, WebhookEndpoint, AccountRateLimits,
                UpdateAccountConfigRequest, UpdateBrowserConfig, UpdateWebhookConfig, UpdateRateLimits,
                // Account operations
                WhatsAppStatusResponse, PhoneLinkRequest, ProfileInfo,
                UpdateProfileRequest,
            )
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "Health", description = "Server health and metrics endpoints"),
            (name = "Accounts", description = "Account lifecycle management (CRUD, start, stop, config)"),
            (name = "Account", description = "Account operations: linking, profile, privacy"),
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

    // All account routes (management + WhatsApp ops) under one namespace
    let accounts_routes = Router::new()
        // Account CRUD
        .route("/", get(accounts::list_accounts))
        .route("/", post(accounts::create_account))
        .route("/:account_id", get(accounts::get_account))
        .route("/:account_id", delete(accounts::delete_account))
        // Account lifecycle
        .route("/:account_id/start", post(accounts::start_account))
        .route("/:account_id/stop", delete(accounts::stop_account))
        .route("/:account_id/screenshot", get(accounts::screenshot))
        .route("/:account_id/config", get(accounts::get_account_config))
        .route(
            "/:account_id/config",
            put(accounts::update_account_config),
        )
        // WhatsApp auth & linking
        .route("/:account_id/status", get(whatsapp::get_account_status))
        .route("/:account_id/link/qr", get(whatsapp::get_qr_code))
        .route("/:account_id/link/phone", post(whatsapp::link_phone))
        .route("/:account_id/unlink", delete(whatsapp::unlink))
        // Profile & privacy
        .route(
            "/:account_id/profile",
            get(whatsapp::get_profile).put(whatsapp::update_profile),
        )
        // Chats & messages
        .route("/:account_id/chats", get(chat::list_chats))
        .route("/:account_id/chats/events", get(chat::watch_messages))
        .route("/:account_id/chats/:chat_id", get(chat::get_chat_messages))
        .route("/:account_id/messages", post(chat::send_message))
        .route("/:account_id/messages/:message_id", get(chat::get_message))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .with_state(account_manager.clone());

    // Mount health routes at /api (no auth required)
    app = app.nest("/api", health_routes);

    // Mount all v1 routes
    app = app.nest("/api/v1/accounts", accounts_routes);

    info!("📖 API at /api/v1/accounts");

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
