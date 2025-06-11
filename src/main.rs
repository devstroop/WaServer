use axum::{
    extract::{DefaultBodyLimit, State},
    http::Method,
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use wae_rust::{
    config::AppConfig,
    handlers::{auth, chat},
    models::{auth::*, chat::*},
    services::whatsapp::WhatsAppService,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::get_auth_status,
        auth::get_qr_code,
        auth::login_with_phone,
        auth::logout,
        chat::send_message,
    ),
    components(
        schemas(AuthStatusResponse, QrCodeResponse, PhoneAuthResponse, SuccessResponse, ErrorResponse, SendMessageResponse)
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Authentication", description = "WhatsApp authentication endpoints"),
        (name = "Chat", description = "WhatsApp chat and messaging endpoints")
    ),
    info(
        title = "WhatsApp Engine - Rust Edition",
        version = "0.1.0",
        description = "A high-performance WhatsApp Web automation engine built in Rust with parallel processing capabilities",
        contact(
            name = "DevStroop",
            email = "devstroop@example.com"
        )
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            )
        }
    }
}

async fn auth_middleware(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
    headers: axum::http::HeaderMap,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    // Get API token from configuration through the WhatsApp service
    let expected_token = whatsapp_service.get_api_token();

    // Extract the Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok());

    if let Some(auth_value) = auth_header {
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            if token == expected_token {
                return Ok(next.run(request).await);
            }
        }
    }

    Err(axum::http::StatusCode::UNAUTHORIZED)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Failed to load configuration: {}", e);
        eprintln!("Using default configuration");
        AppConfig::default()
    });

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration validation failed: {}", e);
        std::process::exit(1);
    }

    // Initialize tracing
    let log_level = match config.logging.level.as_str() {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => LevelFilter::INFO,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("wae_rust={}", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting WhatsApp Engine - Rust Edition");
    info!("Server will listen on {}:{}", config.server.host, config.server.port);

    let config = Arc::new(config);

    // Initialize WhatsApp service
    let whatsapp_service = Arc::new(WhatsAppService::new(config.clone()));
    
    // Initialize the service
    if let Err(e) = whatsapp_service.initialize().await {
        eprintln!("Failed to initialize WhatsApp service: {}", e);
        std::process::exit(1);
    }

    info!("WhatsApp service initialized successfully");

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);

    // Create the application router
    let app = Router::new()
        .route("/api/auth/status", get(auth::get_auth_status))
        .route("/api/auth/qrcode", get(auth::get_qr_code))
        .route("/api/auth/phone/:phone_number", post(auth::login_with_phone))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/chat/send", post(chat::send_message))
        .layer(middleware::from_fn_with_state(whatsapp_service.clone(), auth_middleware))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors)
                .layer(DefaultBodyLimit::max(config.limits.max_upload_size))
        )
        .with_state(whatsapp_service);

    // Create listener
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
        .await?;

    info!("🚀 Server is running!");
    info!("📖 API Documentation: http://{}:{}/swagger-ui/", config.server.host, config.server.port);
    info!("🔗 OpenAPI Spec: http://{}:{}/api-docs/openapi.json", config.server.host, config.server.port);

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}
