//! # WhatsApp Engine Rust Library
//!
//! A modular, high-performance WhatsApp Web automation engine built in Rust.
//!
//! ## Architecture
//!
//! The library is organized into feature-gated modules:
//!
//! - **Core** (always included): Browser automation, authentication, messaging, sessions
//! - **CLI** (`cli` feature): Command-line interface with service management
//! - **API** (`api` feature): REST API server with OpenAPI documentation  
//! - **MCP** (`mcp` feature): Model Context Protocol server over SSE
//!
//! ## Features
//!
//! Enable features in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! # Core library only
//! whatsapp-engine = { version = "0.2", default-features = false }
//!
//! # With REST API
//! whatsapp-engine = { version = "0.2", features = ["api"] }
//!
//! # With MCP server
//! whatsapp-engine = { version = "0.2", features = ["mcp"] }
//!
//! # Full server (API + MCP)
//! whatsapp-engine = { version = "0.2", features = ["server"] }
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use wae_rust::{WhatsAppEngine, Result};
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
pub mod engine;
pub mod error;
pub mod models;
pub mod services;
pub mod utils;

// Legacy modules (for backwards compatibility during transition)
pub mod locators;
pub mod session;

// ============================================================================
// Feature-gated Modules
// ============================================================================

/// HTTP API handlers (requires `api` or `mcp` feature)
#[cfg(any(feature = "api", feature = "mcp"))]
pub mod api;

/// HTTP handlers - legacy alias for api (requires `api` or `mcp` feature)
#[cfg(any(feature = "api", feature = "mcp"))]
pub mod handlers;

/// HTTP middleware (requires `api` or `mcp` feature)
#[cfg(any(feature = "api", feature = "mcp"))]
pub mod middleware;

// Re-export public API
pub use browser::{BrowserService, Locators};
pub use config::AppConfig;
pub use engine::WhatsAppEngine;
pub use error::{Result, WhatsAppError};
pub use models::domain::*;
