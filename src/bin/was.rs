//! WAS - WhatsApp Server
//!
//! REST API and MCP server for WhatsApp Web automation.
//! Now uses AccountManager for multi-account support (no legacy WhatsAppService).
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
    services::{AccountManager, AuthTokenService},
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
        routing::{delete, get, post},
        Router,
    };
    use tower::ServiceBuilder;
    use tower_http::{
        cors::{Any, CorsLayer},
        services::ServeDir,
        trace::TraceLayer,
    };
    use utoipa::{
        openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
        Modify, OpenApi,
    };
    use was::{
        api::{account, accounts, auth, chat, health},
        handlers::{partials, templates},
        middleware::{
            account_middleware, auth_middleware, correlation_id_middleware,
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
            // Accounts management
            accounts::list_accounts,
            accounts::create_account,
            accounts::get_account,
            accounts::delete_account,
            accounts::start_account,
            accounts::stop_account,
            accounts::discover_accounts,
            // Account operations (requires X-Account-Id)
            account::get_account_status,
            account::get_qr_code,
            account::link_phone,
            account::unlink,
            account::get_profile,
            account::update_profile,
            account::get_privacy,
            account::update_privacy,
            // Local auth (JWT)
            auth::get_local_auth_status,
            auth::local_login,
            auth::refresh_token,
            auth::local_logout,
            // Chat (requires X-Account-Id)
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
                LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, LocalAuthStatusResponse,
                // Chat
                SendMessageRequest, SendMessageResponse, ChatListResponse, ChatInfo, Message, MessageInfo, MessageListResponse, MessageQueryParams,
                // Account management
                CreateAccountRequest, CreateAccountResponse, AccountListResponse, AccountInfo, AccountStatus,
                DeleteAccountResponse, DeleteAccountQuery, AccountActionResponse, ListAccountsQuery,
                // Account operations
                WhatsAppStatusResponse, PhoneLinkRequest, ProfileInfo, PrivacySettings,
                UpdateProfileRequest, UpdatePrivacyRequest,
                PrivacyVisibility, OnlineVisibility, GroupAddPermission
            )
        ),
        modifiers(&SecurityAddon),
        tags(
            // Admin API tags
            (name = "Health", description = "Server health and metrics endpoints"),
            (name = "Authentication", description = "Server authentication with JWT tokens"),
            (name = "Accounts", description = "Administrative account management (create, list, delete, start, stop)"),
            // WhatsApp API tags (require X-Account-Id)
            (name = "Account", description = "WhatsApp authentication, account operations (profile, privacy)"),
            (name = "Messaging", description = "Send, receive and manage messages")
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

    // Initialize auth token service for JWT authentication (always enabled)
    let auth_token_service = match AuthTokenService::new(
        config.auth.jwt_secret.clone(),
        config.auth.token_expiry_hours,
        config.auth.refresh_token_expiry_days,
    ) {
        Ok(service) => {
            let service = Arc::new(service);
            
            // Check if initial setup is needed
            if let Some(setup_token) = service.get_setup_token() {
                info!("🔐 JWT authentication service initialized");
                info!("");
                info!("╔══════════════════════════════════════════════════════════════════╗");
                info!("║                    INITIAL SETUP REQUIRED                        ║");
                info!("╠══════════════════════════════════════════════════════════════════╣");
                info!("║  No admin user found. Use this one-time setup token to create    ║");
                info!("║  your first admin account:                                       ║");
                info!("║                                                                  ║");
                info!("║  Setup Token: {}              ║", setup_token);
                info!("║                                                                  ║");
                info!("║  Visit: http://{}:{}/setup                             ║", config.server.host, config.server.port);
                info!("║  Or POST to: /api/v1/auth/setup                           ║");
                info!("╚══════════════════════════════════════════════════════════════════╝");
                info!("");
            } else {
                info!("🔐 JWT authentication service initialized");
            }
            
            Some(service)
        }
        Err(e) => {
            tracing::error!("Failed to initialize auth token service: {}", e);
            None
        }
    };

    // Create auth state for middleware
    let auth_state = AuthState::new(
        config.auth.secret_key.clone(),
        auth_token_service.clone(),
    );

    // Create local auth state for JWT routes
    let local_auth_state = auth::LocalAuthState {
        config: config.clone(),
        auth_token_service,
    };

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

    // ==========================================================================
    // Admin API Routes (/api/v1/admin) - Server administration, no X-Account-Id
    // ==========================================================================
    info!("📖 Admin API at /api/v1/admin");

    // Health check routes (no auth required)
    let health_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .route("/live", get(health::liveness_check))
        .route("/metrics", get(health::get_metrics))
        .with_state(account_manager.clone());

    // Account management routes (API auth required)
    let accounts_routes = Router::new()
        .route("/", get(accounts::list_accounts))
        .route("/", post(accounts::create_account))
        .route("/discover", post(accounts::discover_accounts))
        .route("/:id", get(accounts::get_account))
        .route("/:id", delete(accounts::delete_account))
        .route("/:id/start", post(accounts::start_account))
        .route("/:id/stop", post(accounts::stop_account))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .with_state(account_manager.clone());

    // Local auth routes (JWT-based server authentication)
    let auth_routes = Router::new()
        .route("/current-user", get(auth::get_local_auth_status))
        .route("/login", post(auth::local_login))
        .route("/refresh", post(auth::refresh_token))
        .route("/logout", delete(auth::local_logout))
        // Setup routes (no auth required)
        .route("/setup", get(auth::get_setup_status))
        .route("/setup", post(auth::complete_setup))
        .with_state(local_auth_state);

    // Mount health routes at /api (no auth required)
    app = app.nest("/api", health_routes);

    // ==========================================================================
    // API v1 Routes (/api/v1)
    // ==========================================================================
    info!("📖 API at /api/v1");

    // Account operations routes (profile, privacy, link/unlink)
    let account_routes = Router::new()
        // WhatsApp linking status and operations
        .route("/status", get(account::get_account_status))
        .route("/unlink", delete(account::unlink))
        .route("/link/qr", get(account::get_qr_code))
        .route("/link/phone", post(account::link_phone))
        // Profile management (GET + PUT with all fields optional)
        .route("/profile", get(account::get_profile).put(account::update_profile))
        // Privacy settings (GET + PUT with all fields optional)
        .route("/profile/privacy", get(account::get_privacy).put(account::update_privacy))
        .layer(middleware::from_fn_with_state(
            account_manager.clone(),
            account_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ));

    // Chat routes
    let chat_routes = Router::new()
        .route("/", get(chat::list_chats))
        .route("/events", get(chat::watch_messages))
        .route("/:chat_id", get(chat::get_chat_messages))
        .layer(middleware::from_fn_with_state(
            account_manager.clone(),
            account_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ));

    // Message routes
    let message_routes = Router::new()
        .route("/", post(chat::send_message))
        .route("/:message_id", get(chat::get_message))
        .layer(middleware::from_fn_with_state(
            account_manager.clone(),
            account_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ));

    // Mount all v1 routes
    app = app.nest(
        "/api/v1",
        Router::new()
            // Admin routes (server auth, account management)
            .nest("/auth", auth_routes)
            .nest("/accounts", accounts_routes)
            // WhatsApp routes (require X-Account-Id header)
            .nest("/account", account_routes)
            .nest("/chats", chat_routes)
            .nest("/messages", message_routes),
    );

    // HTMX Page routes (no state required)
    info!("🌐 HTMX Web UI at /");
    app = app
        .route("/", get(templates::dashboard_page))
        .route("/auth", get(templates::auth_page))
        .route("/chats", get(templates::chat_page))
        .route("/settings", get(templates::settings_page))
        .route("/webhooks", get(templates::webhooks_page))
        .route("/tokens", get(templates::tokens_page));

    // HTMX Partial routes (for dynamic updates) - need AccountManager state
    let partials_routes = Router::new()
        .route("/partials/health-cards", get(partials::health_cards))
        .route("/partials/auth-panel", get(partials::auth_panel))
        .route("/partials/qr-code", get(partials::qr_code))
        .route("/partials/auth-indicator", get(partials::auth_indicator))
        .route("/partials/phone-pair", post(partials::phone_pair))
        .route("/partials/chat-list", get(partials::chat_list))
        .route("/partials/chat-view/:chat_id", get(partials::chat_view))
        .route(
            "/partials/link-device-card",
            get(partials::link_device_card),
        )
        .route(
            "/partials/connected-account",
            get(partials::connected_account),
        )
        .route(
            "/partials/session-controls",
            get(partials::session_controls),
        )
        .with_state(account_manager.clone());

    // Stateless partials
    let stateless_partials = Router::new()
        .route("/partials/server-info", get(partials::server_info))
        .route("/partials/token-list", get(partials::token_list))
        .route("/partials/webhook-list", get(partials::webhook_list));

    app = app.merge(partials_routes).merge(stateless_partials);

    // Static files (JS, CSS, images)
    app = app.nest_service("/static", ServeDir::new("static"));

    // Swagger UI documentation (configurable)
    if config.swagger.enabled {
        use utoipa_swagger_ui::SwaggerUi;
        
        info!("📚 Swagger UI at {}", config.swagger.path);
        let swagger_path = config.swagger.path.clone();
        app = app.merge(
            SwaggerUi::new(swagger_path).url("/api-docs/openapi.json", ApiDoc::openapi()),
        );
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
