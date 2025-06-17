// WhatsApp Engine - API Server Binary
//
// This binary provides a REST API server using the WhatsApp Engine library.
// It wraps the library functionality in HTTP endpoints.

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
use tracing::info;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use wae_rust::{
    config::AppConfig,
    handlers::{auth, chat, health},
    middleware,
    models::{auth::*, chat::*},
    services::whatsapp::WhatsAppService,
    utils::logging,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::get_auth_status,
        auth::get_qr_code,
        auth::login_with_phone,
        auth::logout,
        chat::send_message,
        health::health_check,
        health::readiness_check,
        health::liveness_check,
        health::get_metrics,
    ),
    components(
        schemas(AuthStatusResponse, QrCodeResponse, PhoneAuthResponse, SuccessResponse, ErrorResponse, SendMessageResponse, health::HealthResponse, health::MetricsResponse, health::ServiceHealth)
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Authentication", description = "WhatsApp authentication endpoints"),
        (name = "Chat", description = "WhatsApp chat and messaging endpoints"),
        (name = "Health", description = "Health check and readiness endpoints"),
        (name = "Metrics", description = "Service metrics and observability endpoints")
    ),
    info(
        title = "WhatsApp Engine - API Server",
        version = "0.2.0",
        description = "REST API server powered by WhatsApp Engine library - A high-performance WhatsApp Web automation engine built in Rust",
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
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
        )
    }
}

async fn auth_middleware(
    State(whatsapp_service): State<Arc<WhatsAppService>>,
    headers: axum::http::HeaderMap,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    middleware::auth_middleware(State(whatsapp_service), headers, request, next).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Setup logging based on environment configuration
    if let Err(e) = logging::init_logging(&config.environment) {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    info!("Starting WhatsApp Engine API Server");
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
        // Health and metrics endpoints (no auth required)
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        .route("/live", get(health::liveness_check))
        .route("/metrics", get(health::get_metrics))
        // API endpoints (auth required)
        .nest("/api", Router::new()
            .route("/auth/status", get(auth::get_auth_status))
            .route("/auth/qrcode", get(auth::get_qr_code))
            .route("/auth/phone/:phone_number", post(auth::login_with_phone))
            .route("/auth/logout", post(auth::logout))
            .route("/chat/send", post(chat::send_message))
            .layer(middleware::from_fn_with_state(whatsapp_service.clone(), auth_middleware))
        )
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(middleware::correlation_id_middleware))
                .layer(middleware::from_fn(middleware::request_metrics_middleware))
                .layer(middleware::from_fn(middleware::security_headers_middleware))
                .layer(cors)
                .layer(DefaultBodyLimit::max(config.limits.max_upload_size))
        )
        .with_state(whatsapp_service);

    // Create listener
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
        .await?;

    info!("🚀 WhatsApp Engine API Server is running!");
    info!("📖 API Documentation: http://{}:{}/swagger-ui/", config.server.host, config.server.port);
    info!("🔗 OpenAPI Spec: http://{}:{}/api-docs/openapi.json", config.server.host, config.server.port);
    info!("💡 This server is powered by WhatsApp Engine library");

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}
