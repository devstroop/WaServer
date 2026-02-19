//! Web UI Handlers for WAS (WhatsApp Server)
//!
//! This module contains Axum handlers for the HTMX-based web interface.
//! For REST API handlers, see the `api` module.

/// REST API handlers
pub mod api;

/// Template handlers for full-page HTML responses
pub mod templates;

/// HTMX partial handlers for dynamic content fragments
pub mod partials;
