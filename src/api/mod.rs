//! HTTP Handlers for WhatsApp Engine
//!
//! This module contains Axum handlers for the REST API and MCP server.
//! Feature-gated to only compile when `api` or `mcp` features are enabled.

/// Authentication handlers (REST API)
#[cfg(feature = "api")]
pub mod auth;

/// Chat/messaging handlers (REST API)
#[cfg(feature = "api")]
pub mod chat;

/// Health check handlers (shared between API and MCP)
pub mod health;

/// MCP (Model Context Protocol) handlers over SSE
#[cfg(feature = "mcp")]
pub mod mcp;
