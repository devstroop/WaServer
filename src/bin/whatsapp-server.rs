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
//! WHATSAPP_PORT=8080 WHATSAPP_API_TOKEN=secret was
//! ```

use std::sync::Arc;
use tracing::info;

use was::{config::AppConfig, services::whatsapp::WhatsAppService, utils::logging};

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

    // Initialize WhatsApp service
    let whatsapp_service = Arc::new(WhatsAppService::new(config.clone()));

    whatsapp_service.initialize().await.unwrap_or_else(|e| {
        eprintln!("Failed to initialize WhatsApp service: {}", e);
        std::process::exit(1);
    });

    info!("WhatsApp service initialized");

    // Run server
    run_server(config, whatsapp_service).await
}

async fn run_server(
    config: Arc<AppConfig>,
    whatsapp_service: Arc<WhatsAppService>,
) -> Result<(), Box<dyn std::error::Error>> {
    use axum::{
        extract::DefaultBodyLimit,
        http::Method,
        middleware,
        routing::{get, post},
        Router,
    };
    use tower::ServiceBuilder;
    use tower_http::{
        cors::{Any, CorsLayer},
        services::{ServeDir, ServeFile},
        trace::TraceLayer,
    };
    use utoipa::{
        openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
        Modify, OpenApi,
    };
    use utoipa_swagger_ui::SwaggerUi;
    use was::{
        handlers::{auth, chat, health},
        middleware::{
            auth_middleware, correlation_id_middleware, request_metrics_middleware,
            security_headers_middleware,
        },
        models::{auth::*, chat::*},
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
            auth::get_auth_status,
            auth::get_qr_code,
            auth::login_with_phone,
            auth::logout,
            auth::get_local_auth_status,
            auth::local_login,
            auth::refresh_token,
            auth::local_logout,
            chat::send_message,
            chat::list_chats,
            chat::get_chat_messages,
            chat::watch_messages,
            chat::get_message,
            health::health_check,
        ),
        components(
            schemas(
                AuthStatusResponse, QrCodeResponse, PhoneLoginRequest, PhoneAuthResponse, SuccessResponse, ErrorResponse,
                LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, LocalAuthStatusResponse,
                SendMessageRequest, SendMessageResponse, ChatListResponse, ChatInfo, Message, MessageInfo, MessageListResponse, MessageQueryParams,
                health::HealthResponse, health::ServiceHealth
            )
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "Authentication", description = "WhatsApp authentication endpoints"),
            (name = "Local Authentication", description = "Local user authentication with JWT tokens"),
            (name = "Chat", description = "WhatsApp chat and messaging endpoints"),
            (name = "Messages", description = "Message management endpoints"),
            (name = "Health", description = "Health check endpoints")
        ),
        info(
            title = "WAS - WhatsApp Server API",
            version = "0.2.0",
            description = "REST API for WhatsApp Web automation"
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

    // Health endpoint
    let mut app = Router::new().route("/health", get(health::health_check));

    // MCP endpoints (feature-gated and config-enabled)
    #[cfg(feature = "mcp")]
    if config.mcp.enabled {
        use was::handlers::mcp;
        info!("🤖 MCP enabled at {}", config.mcp.endpoint);
        app = app.nest(
            &config.mcp.endpoint,
            mcp::mcp_routes(whatsapp_service.clone()),
        );
    }

    #[cfg(feature = "mcp")]
    if !config.mcp.enabled {
        info!("🤖 MCP disabled (set mcp.enabled = true to enable)");
    }

    #[cfg(not(feature = "mcp"))]
    info!("🤖 MCP not compiled (build with --features mcp to enable)");

    info!("📖 REST API at /api/v1");

    // REST API endpoints (always included)
    app = app.nest(
        "/api/v1",
        Router::new()
            // WhatsApp auth routes (protected)
            .route("/auth/status", get(auth::get_auth_status))
            .route("/auth/qr", get(auth::get_qr_code))
            .route("/auth/phone", post(auth::login_with_phone))
            .route("/auth/logout", post(auth::logout))
            // Chat routes (protected)
            .route("/chats", get(chat::list_chats))
            .route("/chats/events", get(chat::watch_messages))
            .route("/chats/:chat_id", get(chat::get_chat_messages))
            .route("/messages", post(chat::send_message))
            .route("/messages/:message_id", get(chat::get_message))
            .layer(middleware::from_fn_with_state(
                whatsapp_service.clone(),
                auth_middleware,
            ))
            // Local auth routes (public - no auth middleware)
            .route("/auth/local-status", get(auth::get_local_auth_status))
            .route("/auth/login", post(auth::local_login))
            .route("/auth/refresh", post(auth::refresh_token))
            .route("/auth/local-logout", post(auth::local_logout)),
    );

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

    // Serve static frontend if enabled and app/dist exists
    if config.web.enabled {
        let frontend_path = std::path::Path::new(&config.web.path);
        if frontend_path.exists() {
            info!("🌐 Web UI at / (serving from {})", config.web.path);
            app = app.fallback_service(
                ServeDir::new(frontend_path)
                    .not_found_service(ServeFile::new(frontend_path.join("index.html"))),
            );
        } else {
            info!(
                "💡 Web UI enabled but '{}' not found. Run 'cd app && npm install && npm run build' to build.",
                config.web.path
            );
        }
    } else {
        info!("🌐 Web UI disabled (set web.enabled = true to enable)");
    }

    // Middleware
    let app = app
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(correlation_id_middleware))
                .layer(middleware::from_fn(request_metrics_middleware))
                .layer(middleware::from_fn(security_headers_middleware))
                .layer(cors)
                .layer(DefaultBodyLimit::max(config.limits.max_upload_size)),
        )
        .with_state(whatsapp_service);

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
