//! # WAS - WhatsApp Server
//!
//! A minimal WhatsApp Web automation server built in Rust.
//! Focused on sending messages only — no incoming, no MCP, no contacts/JID.
//!
//! ## Architecture
//!
//! The library provides:
//!
//! - **Core**: Browser automation, authentication, sending messages
//! - **REST API**: REST API with OpenAPI/Swagger documentation
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

pub mod application;
pub mod browser;
pub mod config;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod models;
pub mod services;
pub mod shared;
pub mod utils;

// ============================================================================
// Server Modules (always included)
// ============================================================================

/// HTTP handlers (REST API)
pub mod handlers;

/// HTTP middleware
pub mod middleware;

/// Interfaces layer — identity handlers split (#9)
pub mod interfaces;

// Re-export api at crate root for convenience
pub use handlers::api;

// Re-export public API
pub use browser::{
    BrowserService, LocatorConfig, Locators, SessionManager, Timeouts, WhatsAppEngine,
};
pub use config::AppConfig;
pub use error::{Result, WhatsAppError};
pub use models::domain::*;
