//! WAS - WhatsApp Server
//!
//! Minimal REST API server for WhatsApp Web automation (sending only).
//! Thin bootstrap per #10 — router/middleware extracted to `interfaces::http`.

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
    let config = AppConfig::load().unwrap_or_else(|e| {
        eprintln!("Failed to load configuration: {}", e);
        std::process::exit(1);
    });
    logging::init_logging(&config.environment).unwrap_or_else(|e| {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    });
    print_banner();
    was::handlers::api::health::init();
    info!("Starting WAS (WhatsApp Server) v{}", VERSION);
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
    let auth_db = db.clone();
    let instance_manager = Arc::new(InstanceManager::new(config.clone(), db));
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
    run_server(config, instance_manager, auth_db).await
}

async fn run_server(
    config: Arc<AppConfig>,
    instance_manager: Arc<InstanceManager>,
    auth_db: Database,
) -> Result<(), Box<dyn std::error::Error>> {
    // Router extracted to `interfaces::http::router::build_full_router` per #10 — bin is now ~50 lines
    let app =
        was::interfaces::http::router::build_full_router(config.clone(), instance_manager, auth_db);
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
            .await?;
    info!(
        "🚀 WAS running at http://{}:{}",
        config.server.host, config.server.port
    );
    let shutdown = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::warn!("Failed to install Ctrl+C handler: {}", e);
            std::future::pending::<()>().await;
        }
        info!("🛑 Shutting down...");
    };
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
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
