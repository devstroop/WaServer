//! HTTP Handlers for WAS (WhatsApp Server)
//!
//! This module contains Axum handlers for the REST API and MCP server.

/// Authentication handlers (REST API)
pub mod auth;

/// Chat/messaging handlers (REST API)
pub mod chat;

/// Health check handlers (shared between API and MCP)
pub mod health;

/// MCP (Model Context Protocol) handlers over SSE
#[cfg(feature = "mcp")]
pub mod mcp;
