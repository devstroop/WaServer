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
//! WHATSAPP_PORT=8080 WHATSAPP_API_TOKEN=secret was
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
        routing::{delete, get, post, put},
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
    use utoipa_swagger_ui::SwaggerUi;
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
            account::logout,
            account::get_profile,
            account::update_profile_name,
            account::update_profile_about,
            account::update_profile_picture,
            account::get_privacy,
            account::update_privacy_last_seen,
            account::update_privacy_online,
            account::update_privacy_profile_photo,
            account::update_privacy_about,
            account::update_privacy_read_receipts,
            account::update_privacy_groups,
            // WhatsApp auth (requires X-Account-Id)
            auth::get_auth_status,
            auth::get_qr_code,
            auth::login_with_phone,
            auth::logout,
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
                UpdateProfileNameRequest, UpdateProfileAboutRequest,
                UpdatePrivacyLastSeenRequest, UpdatePrivacyOnlineRequest, UpdatePrivacyProfilePhotoRequest,
                UpdatePrivacyAboutRequest, UpdatePrivacyReadReceiptsRequest, UpdatePrivacyGroupsRequest,
                PrivacyVisibility, OnlineVisibility, GroupAddPermission
            )
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "Health", description = "Health check endpoints"),
            (name = "Accounts", description = "Account management (create, list, delete)"),
            (name = "Account", description = "WhatsApp account operations (requires X-Account-Id)"),
            (name = "Authentication", description = "User authentication with JWT tokens"),
            (name = "Chat", description = "WhatsApp chat and messaging endpoints (requires X-Account-Id)"),
            (name = "Messages", description = "Message management endpoints (requires X-Account-Id)")
        ),
        info(
            title = "WhatsApp Server - API",
            version = "0.3.0",
            description = "REST API for WhatsApp Web with multi-account support. All WhatsApp operations require X-Account-Id header to specify which account to use. Built with Rust and Axum.",
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

    // Initialize auth token service for local auth (JWT)
    let auth_token_service = if config.local_auth.enabled {
        match AuthTokenService::new(
            config.local_auth.jwt_secret.clone(),
            config.local_auth.token_expiry_hours,
            config.local_auth.refresh_token_expiry_days,
            Some(config.local_auth.default_username.clone()),
            Some(config.local_auth.default_password.clone()),
        ) {
            Ok(service) => {
                info!("🔐 Local authentication enabled with JWT tokens");
                Some(Arc::new(service))
            }
            Err(e) => {
                tracing::error!("Failed to initialize auth token service: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Create auth state for middleware
    let auth_state = AuthState::new(
        config.auth.enabled,
        config.local_auth.enabled,
        config.auth.api_token.clone(),
        auth_token_service.clone(),
    );

    // Create local auth state for JWT routes
    let local_auth_state = auth::LocalAuthState {
        config: config.clone(),
        auth_token_service,
    };

    // Health endpoint (uses AccountManager for metrics)
    let mut app = Router::new()
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .route("/live", get(health::liveness_check))
        .route("/metrics", get(health::get_metrics))
        .with_state(account_manager.clone());

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

    info!("📖 REST API at /api/v1");

    // Account management routes (no X-Account-Id required, but API auth required)
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

    // Account operations routes (requires X-Account-Id header)
    let account_routes = Router::new()
        .route("/status", get(account::get_account_status))
        .route("/qr", get(account::get_qr_code))
        .route("/phone", post(account::link_phone))
        .route("/logout", post(account::logout))
        // Profile management
        .route("/profile", get(account::get_profile))
        .route("/profile/name", put(account::update_profile_name))
        .route("/profile/about", put(account::update_profile_about))
        .route("/profile/picture", put(account::update_profile_picture))
        // Privacy settings
        .route("/privacy", get(account::get_privacy))
        .route(
            "/privacy/last-seen",
            put(account::update_privacy_last_seen),
        )
        .route("/privacy/online", put(account::update_privacy_online))
        .route(
            "/privacy/profile-photo",
            put(account::update_privacy_profile_photo),
        )
        .route("/privacy/about", put(account::update_privacy_about))
        .route(
            "/privacy/read-receipts",
            put(account::update_privacy_read_receipts),
        )
        .route("/privacy/groups", put(account::update_privacy_groups))
        .layer(middleware::from_fn_with_state(
            account_manager.clone(),
            account_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ));

    // WhatsApp auth routes (requires X-Account-Id header)
    let whatsapp_auth_routes = Router::new()
        .route("/status", get(auth::get_auth_status))
        .route("/qr", get(auth::get_qr_code))
        .route("/phone", post(auth::login_with_phone))
        .route("/logout", post(auth::logout))
        .layer(middleware::from_fn_with_state(
            account_manager.clone(),
            account_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ));

    // Local auth routes (public - no auth middleware, no account needed)
    let local_auth_routes = Router::new()
        .route("/local-status", get(auth::get_local_auth_status))
        .route("/login", post(auth::local_login))
        .route("/refresh", post(auth::refresh_token))
        .route("/local-logout", post(auth::local_logout))
        .with_state(local_auth_state);

    // Chat routes (requires X-Account-Id header)
    let chat_routes = Router::new()
        .route("/chats", get(chat::list_chats))
        .route("/chats/events", get(chat::watch_messages))
        .route("/chats/:chat_id", get(chat::get_chat_messages))
        .route("/messages", post(chat::send_message))
        .route("/messages/:message_id", get(chat::get_message))
        .layer(middleware::from_fn_with_state(
            account_manager.clone(),
            account_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ));

    // REST API endpoints
    app = app.nest(
        "/api/v1",
        Router::new()
            // Account management (no X-Account-Id)
            .nest("/accounts", accounts_routes)
            // Account operations (requires X-Account-Id)
            .nest("/account", account_routes)
            // WhatsApp authentication (requires X-Account-Id)
            .nest("/auth", whatsapp_auth_routes)
            // Local auth routes (public, no X-Account-Id)
            .merge(local_auth_routes)
            // Chat operations (requires X-Account-Id)
            .merge(chat_routes),
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

    // Swagger UI (configurable)
    if config.swagger.enabled {
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
