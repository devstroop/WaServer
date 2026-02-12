//! WhatsApp Engine - Unified CLI Binary
//!
//! This binary provides a unified command-line interface with feature-gated capabilities:
//!
//! - **Core CLI** (`cli` feature): Run core engine with service management
//! - **REST API** (`api` feature): HTTP REST API with OpenAPI documentation
//! - **MCP Server** (`mcp` feature): Model Context Protocol over SSE
//!
//! ## Usage
//!
//! ```bash
//! # Show help
//! whatsapp-engine --help
//!
//! # Run with defaults (uses enabled features)
//! whatsapp-engine run
//!
//! # Run with specific options
//! whatsapp-engine --port 8080 --headless run
//!
//! # Windows service management
//! whatsapp-engine install --start
//! whatsapp-engine status
//! whatsapp-engine stop
//! whatsapp-engine uninstall
//! ```

use clap::{Parser, Subcommand};
use std::sync::Arc;
use tracing::info;

use wae_rust::{config::AppConfig, services::whatsapp::WhatsAppService, utils::logging};

// ============================================================================
// CLI Definitions
// ============================================================================

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SERVICE_NAME: &str = "WhatsAppEngine";
const SERVICE_DISPLAY_NAME: &str = "WhatsApp Engine Service";
const SERVICE_DESCRIPTION: &str = "WhatsApp Web automation engine";

/// WhatsApp Engine - High-performance WhatsApp Web automation
#[derive(Parser, Debug)]
#[command(name = "whatsapp-engine")]
#[command(author = "Devstroop Technologies <info@devstroop.com>")]
#[command(version = VERSION)]
#[command(about = "WhatsApp Engine - Modular WhatsApp Web Automation", long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config/app.toml")]
    config: String,

    /// Server host address
    #[arg(long, env = "WHATSAPP_HOST")]
    host: Option<String>,

    /// Server port
    #[arg(short, long, env = "WHATSAPP_PORT")]
    port: Option<u16>,

    /// Run in headless mode (no browser window)
    #[arg(long)]
    headless: bool,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// API authentication token (overrides config)
    #[arg(long, env = "WHATSAPP_API_TOKEN")]
    api_token: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the engine (default)
    Run {
        /// Disable API server (if compiled with api feature)
        #[cfg(feature = "api")]
        #[arg(long)]
        no_api: bool,

        /// Disable MCP server (overrides config, if compiled with mcp feature)
        #[cfg(feature = "mcp")]
        #[arg(long)]
        no_mcp: bool,

        /// Enable MCP server (overrides config, if compiled with mcp feature)
        #[cfg(feature = "mcp")]
        #[arg(long)]
        enable_mcp: bool,
    },

    /// Install as Windows service
    #[cfg(windows)]
    Install {
        /// Auto-start the service after installation
        #[arg(long)]
        start: bool,
    },

    /// Uninstall Windows service
    #[cfg(windows)]
    Uninstall,

    /// Start the Windows service
    #[cfg(windows)]
    Start,

    /// Stop the Windows service
    #[cfg(windows)]
    Stop,

    /// Show service status
    #[cfg(windows)]
    Status,

    /// Show current configuration
    Config,

    /// Validate configuration file
    Validate,

    /// Show version and build info
    Info,
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if running as Windows service
    #[cfg(windows)]
    {
        // If started by SCM (no console), run as service
        if std::env::args().any(|arg| arg == "--service") {
            return run_as_windows_service();
        }
    }

    // Run as CLI application
    run_cli()
}

fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Config) => {
            show_config(&cli)?;
            return Ok(());
        }
        Some(Commands::Validate) => {
            validate_config(&cli)?;
            return Ok(());
        }
        Some(Commands::Info) => {
            show_info();
            return Ok(());
        }
        #[cfg(windows)]
        Some(Commands::Install { start }) => {
            install_service(*start)?;
            return Ok(());
        }
        #[cfg(windows)]
        Some(Commands::Uninstall) => {
            uninstall_service()?;
            return Ok(());
        }
        #[cfg(windows)]
        Some(Commands::Start) => {
            start_service()?;
            return Ok(());
        }
        #[cfg(windows)]
        Some(Commands::Stop) => {
            stop_service()?;
            return Ok(());
        }
        #[cfg(windows)]
        Some(Commands::Status) => {
            show_service_status()?;
            return Ok(());
        }
        Some(Commands::Run { .. }) | None => {
            // Continue to run the engine
        }
    }

    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_engine(cli))
}

// ============================================================================
// Engine Runner
// ============================================================================

async fn run_engine(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let mut config = match AppConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Override config with CLI arguments
    if let Some(host) = cli.host {
        config.server.host = host;
    }
    if let Some(port) = cli.port {
        config.server.port = port;
    }
    if cli.headless {
        config.browser.headless = true;
    }
    if cli.debug {
        config.logging.level = "debug".to_string();
        config.environment.debug_mode = true;
    }
    if let Some(token) = cli.api_token {
        config.auth.api_token = token;
    }

    // Setup logging
    if let Err(e) = logging::init_logging(&config.environment) {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    print_banner();

    info!("Starting WhatsApp Engine v{}", VERSION);
    info!(
        "Server will listen on {}:{}",
        config.server.host, config.server.port
    );

    // Show enabled features
    print_features();

    let config = Arc::new(config);

    // Initialize WhatsApp service
    let whatsapp_service = Arc::new(WhatsAppService::new(config.clone()));

    if let Err(e) = whatsapp_service.initialize().await {
        eprintln!("Failed to initialize WhatsApp service: {}", e);
        std::process::exit(1);
    }

    info!("WhatsApp service initialized successfully");

    // Build and run the server based on enabled features
    #[cfg(any(feature = "api", feature = "mcp"))]
    {
        // Extract MCP flags from CLI
        #[cfg(feature = "mcp")]
        let (no_mcp, enable_mcp) = match &cli.command {
            Some(Commands::Run {
                no_mcp, enable_mcp, ..
            }) => (*no_mcp, *enable_mcp),
            _ => (false, false),
        };
        #[cfg(not(feature = "mcp"))]
        let (no_mcp, enable_mcp) = (false, false);

        run_server(config, whatsapp_service, no_mcp, enable_mcp).await?;
    }

    #[cfg(not(any(feature = "api", feature = "mcp")))]
    {
        // CLI-only mode - just keep the engine running
        run_cli_mode(whatsapp_service).await?;
    }

    Ok(())
}

// ============================================================================
// Server Mode (API/MCP)
// ============================================================================

#[cfg(any(feature = "api", feature = "mcp"))]
async fn run_server(
    config: Arc<AppConfig>,
    whatsapp_service: Arc<WhatsAppService>,
    no_mcp: bool,
    enable_mcp: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use axum::{extract::DefaultBodyLimit, http::Method, middleware, routing::get, Router};
    use tower::ServiceBuilder;
    use tower_http::{
        cors::{Any, CorsLayer},
        trace::TraceLayer,
    };
    use wae_rust::{
        handlers::health,
        middleware::{
            correlation_id_middleware, request_metrics_middleware, security_headers_middleware,
        },
    };

    // Determine if MCP should be enabled:
    // CLI --enable-mcp overrides everything to ON
    // CLI --no-mcp overrides everything to OFF
    // Otherwise use config.mcp.enabled
    #[cfg(feature = "mcp")]
    let mcp_enabled = if enable_mcp {
        true
    } else if no_mcp {
        false
    } else {
        config.mcp.enabled
    };
    #[cfg(not(feature = "mcp"))]
    let mcp_enabled = false;
    let _ = (no_mcp, enable_mcp); // Suppress unused warnings when mcp feature is off

    // Setup CORS
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

    // Build router - start with health endpoints (always available)
    let mut app = Router::new().route("/health", get(health::health_check));

    // MCP endpoints (only if feature enabled AND config/CLI allows)
    #[cfg(feature = "mcp")]
    if mcp_enabled {
        use wae_rust::handlers::mcp;
        let mcp_endpoint = &config.mcp.endpoint;
        info!("🤖 MCP over SSE enabled at {}/sse", mcp_endpoint);
        app = app.nest(mcp_endpoint, mcp::mcp_routes(whatsapp_service.clone()));
    } else {
        info!("🤖 MCP server disabled by configuration");
    }

    #[cfg(not(feature = "mcp"))]
    {
        let _ = mcp_enabled; // Suppress unused warning
    }

    // API endpoints
    #[cfg(feature = "api")]
    {
        use axum::routing::post;
        use utoipa::{
            openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
            Modify, OpenApi,
        };
        use utoipa_swagger_ui::SwaggerUi;
        use wae_rust::{
            handlers::{auth, chat},
            middleware::auth_middleware,
            models::{auth::*, chat::*},
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
            ),
            components(
                schemas(AuthStatusResponse, QrCodeResponse, PhoneAuthResponse, SuccessResponse, ErrorResponse, SendMessageResponse, health::HealthResponse, health::ServiceHealth)
            ),
            modifiers(&SecurityAddon),
            tags(
                (name = "Authentication", description = "WhatsApp authentication endpoints"),
                (name = "Chat", description = "WhatsApp chat and messaging endpoints"),
                (name = "Health", description = "Health check and readiness endpoints")
            ),
            info(
                title = "WhatsApp Engine API",
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

        info!("📖 REST API enabled at /api");
        info!("📚 Swagger UI at /swagger-ui/");

        app = app
            .nest(
                "/api",
                Router::new()
                    .route("/status", get(auth::get_auth_status))
                    .route("/qr", get(auth::get_qr_code))
                    .route("/login/:phone_number", post(auth::login_with_phone))
                    .route("/logout", post(auth::logout))
                    .route("/send", post(chat::send_message))
                    .layer(middleware::from_fn_with_state(
                        whatsapp_service.clone(),
                        auth_middleware,
                    )),
            )
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
    }

    // Apply global middleware and state
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

    // Create listener and serve
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
            .await?;

    info!("🚀 WhatsApp Engine is running!");
    info!("🔗 http://{}:{}", config.server.host, config.server.port);

    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// CLI-Only Mode
// ============================================================================

#[cfg(not(any(feature = "api", feature = "mcp")))]
async fn run_cli_mode(
    _whatsapp_service: Arc<WhatsAppService>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running in CLI-only mode (no API/MCP server)");
    info!("Press Ctrl+C to exit");

    // Keep the engine running
    tokio::signal::ctrl_c().await?;

    info!("Shutting down...");
    Ok(())
}

// ============================================================================
// Utility Functions
// ============================================================================

fn print_banner() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║   __        ___           _       _                       ║
║   \ \      / / |__   __ _| |_ ___| |__   ___  _ __        ║
║    \ \ /\ / /| '_ \ / _` | __/ __|__'_ \ / _ \| '_ \      ║
║     \ V  V / | | | | (_| | |_\__ \ |_) | (_) | |_) |      ║
║      \_/\_/  |_| |_|\__,_|\__|___/_.__/ \___/| .__/       ║
║                                              |_|          ║
║   ╔═══════════════════════════════════════════════════╗   ║
║   ║  E N G I N E   v{:<29}║   ║
║   ╚═══════════════════════════════════════════════════╝   ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
"#,
        VERSION
    );
}

fn print_features() {
    println!("Enabled Features:");

    #[cfg(feature = "cli")]
    println!("  ✓ CLI with service management");

    #[cfg(feature = "api")]
    println!("  ✓ REST API with OpenAPI/Swagger");

    #[cfg(feature = "mcp")]
    println!("  ✓ MCP Server over SSE");

    #[cfg(windows)]
    println!("  ✓ Windows Service support");

    println!();
}

fn show_config(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Current Configuration\n");

    let config = AppConfig::load()?;

    println!("Server:");
    println!(
        "  Host: {}",
        cli.host.as_ref().unwrap_or(&config.server.host)
    );
    println!("  Port: {}", cli.port.unwrap_or(config.server.port));
    println!();
    println!("Browser:");
    println!("  Headless: {}", cli.headless || config.browser.headless);
    println!("  Timeout: {}ms", config.browser.timeout_ms);
    println!();
    println!("Authentication:");
    println!("  Token configured: {}", !config.auth.api_token.is_empty());
    println!();
    println!("Logging:");
    println!(
        "  Level: {}",
        if cli.debug {
            "debug"
        } else {
            &config.logging.level
        }
    );
    println!();
    println!("Limits:");
    println!(
        "  Max concurrent requests: {}",
        config.limits.max_concurrent_requests
    );
    println!("  Request timeout: {}ms", config.limits.request_timeout_ms);
    println!("  Max upload size: {} bytes", config.limits.max_upload_size);
    println!();
    println!("MCP (Model Context Protocol):");
    println!("  Enabled: {}", config.mcp.enabled);
    println!("  Endpoint: {}", config.mcp.endpoint);
    println!("  SSE enabled: {}", config.mcp.sse_enabled);
    println!(
        "  Heartbeat interval: {}s",
        config.mcp.heartbeat_interval_secs
    );

    Ok(())
}

fn validate_config(_cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Validating configuration...\n");

    match AppConfig::load() {
        Ok(config) => {
            println!("✅ Configuration file loaded successfully");

            let mut warnings = Vec::new();
            let mut errors = Vec::new();

            if config.auth.api_token == "your-secure-api-token-change-this" {
                warnings.push("API token is using default value - change it for production!");
            }

            if config.auth.api_token.len() < 16 {
                errors.push("API token should be at least 16 characters");
            }

            if config.server.port == 0 {
                errors.push("Server port cannot be 0");
            }

            if config.browser.timeout_ms < 5000 {
                warnings.push("Browser timeout is very low (< 5000ms)");
            }

            if !warnings.is_empty() {
                println!("\n⚠️  Warnings:");
                for w in warnings {
                    println!("   - {}", w);
                }
            }

            if !errors.is_empty() {
                println!("\n❌ Errors:");
                for e in errors {
                    println!("   - {}", e);
                }
                std::process::exit(1);
            }

            println!("\n✅ Configuration is valid!");
        }
        Err(e) => {
            println!("❌ Configuration error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn show_info() {
    println!("WhatsApp Engine");
    println!("===============");
    println!("Version: {}", VERSION);
    println!("Authors: Devstroop Technologies <info@devstroop.com>");
    println!();
    println!("Build Information:");
    println!("  Target: {}", std::env::consts::ARCH);
    println!("  OS: {}", std::env::consts::OS);
    println!();
    println!("Compiled Features:");

    #[cfg(feature = "cli")]
    println!("  ✓ cli - Command-line interface");

    #[cfg(feature = "api")]
    println!("  ✓ api - REST API server");

    #[cfg(feature = "mcp")]
    println!("  ✓ mcp - MCP server over SSE");

    #[cfg(windows)]
    println!("  ✓ windows-service - Windows service support");

    println!();

    #[cfg(feature = "mcp")]
    {
        println!("MCP Tools:");
        println!("  • whatsapp_get_auth_status");
        println!("  • whatsapp_get_qr_code");
        println!("  • whatsapp_login_with_phone");
        println!("  • whatsapp_logout");
        println!("  • whatsapp_send_message");
        println!("  • whatsapp_health_check");
        println!();
    }

    println!("Repository: https://github.com/devstroop/whatsapp-engine-rust");
}

// ============================================================================
// Windows Service Functions
// ============================================================================

#[cfg(windows)]
fn install_service(auto_start: bool) -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::OsString;
    use windows_service::{
        service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType},
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    println!("📦 Installing {} as Windows service...", SERVICE_NAME);

    let manager =
        ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;

    let exe_path = std::env::current_exe()?;

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: if auto_start {
            ServiceStartType::AutoStart
        } else {
            ServiceStartType::OnDemand
        },
        error_control: ServiceErrorControl::Normal,
        executable_path: exe_path,
        launch_arguments: vec![OsString::from("--service")],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    let service = manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description(SERVICE_DESCRIPTION)?;

    println!("✅ Service '{}' installed successfully!", SERVICE_NAME);

    if auto_start {
        println!("   Auto-start: enabled");
        start_service()?;
    } else {
        println!("   Auto-start: disabled");
        println!("   Run 'whatsapp-engine start' to start the service");
    }

    Ok(())
}

#[cfg(windows)]
fn uninstall_service() -> Result<(), Box<dyn std::error::Error>> {
    use windows_service::{
        service::ServiceAccess,
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    println!("🗑️  Uninstalling {} service...", SERVICE_NAME);

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;

    let service = manager.open_service(SERVICE_NAME, ServiceAccess::DELETE)?;
    service.delete()?;

    println!("✅ Service '{}' uninstalled successfully!", SERVICE_NAME);

    Ok(())
}

#[cfg(windows)]
fn start_service() -> Result<(), Box<dyn std::error::Error>> {
    use windows_service::{
        service::ServiceAccess,
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    println!("▶️  Starting {} service...", SERVICE_NAME);

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;

    let service = manager.open_service(SERVICE_NAME, ServiceAccess::START)?;
    service.start(&[] as &[&str])?;

    println!("✅ Service '{}' started!", SERVICE_NAME);

    Ok(())
}

#[cfg(windows)]
fn stop_service() -> Result<(), Box<dyn std::error::Error>> {
    use windows_service::{
        service::ServiceAccess,
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    println!("⏹️  Stopping {} service...", SERVICE_NAME);

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;

    let service = manager.open_service(SERVICE_NAME, ServiceAccess::STOP)?;
    service.stop()?;

    println!("✅ Service '{}' stopped!", SERVICE_NAME);

    Ok(())
}

#[cfg(windows)]
fn show_service_status() -> Result<(), Box<dyn std::error::Error>> {
    use windows_service::{
        service::ServiceAccess,
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;

    match manager.open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS) {
        Ok(service) => {
            let status = service.query_status()?;
            println!("📊 Service Status: {}", SERVICE_NAME);
            println!("   State: {:?}", status.current_state);
            println!("   PID: {:?}", status.process_id);
        }
        Err(_) => {
            println!("📊 Service '{}' is not installed", SERVICE_NAME);
        }
    }

    Ok(())
}

// ============================================================================
// Windows Service Entry Point
// ============================================================================

#[cfg(windows)]
fn run_as_windows_service() -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::OsString;
    use std::sync::mpsc;
    use std::time::Duration;
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
    };

    define_windows_service!(ffi_service_main, service_main);

    fn service_main(_arguments: Vec<OsString>) {
        let _ = run_service();
    }

    fn run_service() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Stop => {
                    let _ = shutdown_tx.send(());
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

        // Report running status
        status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        // Create tokio runtime and run the engine in background
        let rt = tokio::runtime::Runtime::new()?;

        // Run the engine in a separate thread
        std::thread::spawn(move || {
            rt.block_on(async {
                let cli = Cli {
                    config: "config/app.toml".to_string(),
                    host: None,
                    port: None,
                    headless: true,
                    debug: false,
                    api_token: None,
                    command: Some(Commands::Run {
                        #[cfg(feature = "api")]
                        no_api: false,
                        #[cfg(feature = "mcp")]
                        no_mcp: false,
                        #[cfg(feature = "mcp")]
                        enable_mcp: false, // Use config default for Windows service
                    }),
                };
                let _ = run_engine(cli).await;
            });
        });

        // Wait for shutdown signal
        let _ = shutdown_rx.recv();

        // Report stopping status
        status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::StopPending,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(30),
            process_id: None,
        })?;

        // Report stopped status
        status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        Ok(())
    }

    // Start the service dispatcher
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}
