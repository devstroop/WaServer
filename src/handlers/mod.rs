//! Web UI Handlers for WAS (WhatsApp Server)
//!
//! This module contains Axum handlers for the HTMX-based web interface.
//! For REST API handlers, see the `api` module.

/// Page handlers for full-page HTML responses
pub mod pages;

/// HTMX partial handlers for dynamic content fragments
pub mod partials;
