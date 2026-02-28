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
        response::Html,
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
        api::{chat, health, instances, whatsapp},
        middleware::{
            auth_middleware, correlation_id_middleware, request_metrics_middleware,
            security_headers_middleware, AuthState,
        },
        models::{auth::*, chat::*, instance::*},
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
            // get_profile and update_profile hidden from Swagger (not yet implemented)

            // Chat (list_chats and get_chat_messages hidden from Swagger)
            chat::send_message,
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
                // WhatsApp operations
                WhatsAppStatusResponse, ProfileInfo,
                UpdateProfileRequest,
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
        // Profile & privacy
        .route(
            "/:instance_id/profile",
            get(whatsapp::get_profile).put(whatsapp::update_profile),
        )
        // Messages
        .route("/:instance_id/chats", get(chat::list_chats))
        .route("/:instance_id/send", post(chat::send_message))
        .route(
            "/:instance_id/messages/:phone",
            get(chat::get_chat_messages),
        )
        .route(
            "/:instance_id/messages/:phone/typing",
            post(whatsapp::send_typing),
        )
        .route(
            "/:instance_id/messages/:phone/read",
            post(whatsapp::mark_read),
        )
        .route(
            "/:instance_id/messages/:phone/:message_id/react",
            post(whatsapp::send_reaction),
        )
        .route(
            "/:instance_id/messages/:phone/:message_id/reply",
            post(whatsapp::send_reply),
        )
        // Contacts & Groups
        .route(
            "/:instance_id/contacts/:contact_id",
            get(whatsapp::get_contact_info),
        )
        .route(
            "/:instance_id/contacts/:contact_id/presence",
            get(whatsapp::get_presence),
        )
        .route(
            "/:instance_id/groups/:group_id",
            get(whatsapp::get_group_info),
        )
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

    // Root landing page
    let swagger_enabled = config.swagger.enabled;
    let swagger_path = config.swagger.path.clone();
    app = app.route("/", get(move || async move {
        let swagger_link = if swagger_enabled {
            format!(r#"<a href="{swagger_path}">API Documentation (Swagger UI)</a>"#)
        } else {
            "API Documentation (disabled)".to_string()
        };
        Html(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>WAS — WhatsApp Server</title>
<style>
  *{{margin:0;padding:0;box-sizing:border-box}}
  body{{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;background:#0b141a;color:#e9edef;min-height:100vh;display:flex;align-items:center;justify-content:center}}
  .card{{background:#1f2c34;border-radius:12px;padding:2.5rem;max-width:480px;width:90%;box-shadow:0 4px 24px rgba(0,0,0,.4)}}
  h1{{font-size:1.6rem;margin-bottom:.25rem;color:#00a884}}
  .version{{font-size:.85rem;color:#8696a0;margin-bottom:1.5rem}}
  .links{{display:flex;flex-direction:column;gap:.75rem}}
  a{{color:#53bdeb;text-decoration:none;padding:.6rem .8rem;border-radius:8px;background:#182229;transition:background .15s}}
  a:hover{{background:#233138}}
  .sep{{border-top:1px solid #2a3942;margin:.5rem 0}}
  .status{{font-size:.8rem;color:#8696a0;margin-top:1rem;text-align:center}}
</style>
</head>
<body>
<div class="card">
  <h1>WAS</h1>
  <div class="version">WhatsApp Server v{version}</div>
  <div class="links">
    {swagger_link}
    <a href="/api/health">Health Check</a>
    <a href="/api/metrics">Metrics</a>
    <div class="sep"></div>
    <a href="/api-docs/openapi.json">OpenAPI Spec (JSON)</a>
  </div>
  <div class="status">Ready</div>
</div>
</body>
</html>"#, version = env!("CARGO_PKG_VERSION")))
    }));

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
