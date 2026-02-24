//! REST API Handlers for WAS (WhatsApp Server)
//!
//! This module contains Axum handlers for the REST API endpoints.

/// Account management API (create, list, delete accounts)
pub mod accounts;

/// WhatsApp account operations API (status, QR, profile, privacy)
pub mod whatsapp;

/// Chat/messaging API handlers (send message, get chats, message status)
pub mod chat;

/// Health check and metrics handlers (health, ready, live, metrics endpoints)
pub mod health;

/// MCP (Model Context Protocol) server handlers over SSE transport
#[cfg(feature = "mcp")]
pub mod mcp;
