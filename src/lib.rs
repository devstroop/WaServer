//! # WAS - WhatsApp Server
//!
//! A high-performance WhatsApp Web automation server built in Rust.
//!
//! ## Architecture
//!
//! The library provides:
//!
//! - **Core**: Browser automation, authentication, messaging, sessions
//! - **REST API**: Full REST API with OpenAPI/Swagger documentation (always included)
//! - **MCP** (`mcp` feature): Model Context Protocol server over Streamable HTTP
//!
//! ## Features
//!
//! ```toml
//! [dependencies]
//! # Server with REST API (default)
//! was = "0.2"
//!
//! # With MCP server support
//! was = { version = "0.2", features = ["mcp"] }
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use was::{WhatsAppEngine, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let engine = WhatsAppEngine::with_defaults().await?;
//!     
//!     // Authenticate
//!     let qr = engine.authenticate_with_qr().await?;
//!     println!("Scan QR: {}", qr.data);
//!     
//!     // Wait for auth...
//!     while !engine.is_authenticated().await? {
//!         tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
//!     }
//!     
//!     // Send message
//!     engine.send_message("+1234567890", "Hello!").await?;
//!     
//!     engine.close().await
//! }
//! ```

// ============================================================================
// Core Modules (always included)
// ============================================================================

pub mod browser;
pub mod config;
pub mod error;
pub mod models;
pub mod services;
pub mod utils;

// ============================================================================
// Server Modules (always included)
// ============================================================================

/// HTTP handlers (pages, partials, and REST API)
pub mod handlers;

/// HTTP middleware
pub mod middleware;

// Re-export api at crate root for convenience
pub use handlers::api;

// Re-export public API
pub use browser::{BrowserService, LocatorConfig, Locators, SessionManager, Timeouts, WhatsAppEngine};
pub use config::AppConfig;
pub use error::{Result, WhatsAppError};
pub use models::domain::*;
