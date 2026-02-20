//! REST API Handlers for WAS (WhatsApp Server)
//!
//! This module contains Axum handlers for the REST API endpoints.
//! These handlers serve external integrations and programmatic access.

/// Account management API (create, list, delete accounts - no X-Account-Id required)
pub mod accounts;

/// WhatsApp account operations API (status, QR, profile, privacy - uses path param {account_id})
pub mod account;

/// Authentication API handlers (login, logout, QR code, auth status)
pub mod auth;

/// Chat/messaging API handlers (send message, get chats, message status)
pub mod chat;

/// Health check and metrics handlers (health, ready, live, metrics endpoints)
pub mod health;

/// MCP (Model Context Protocol) server handlers over SSE transport
#[cfg(feature = "mcp")]
pub mod mcp;
