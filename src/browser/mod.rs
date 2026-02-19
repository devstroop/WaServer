//! Engine Module
//!
//! Core WAS (WhatsApp Server) implementation including browser automation,
//! element locators, and the main WhatsApp engine interface.

mod core;
mod driver;
mod locators;
mod session;

pub use core::WhatsAppEngine;
pub use driver::{BrowserService, BrowserServiceConfig};
pub use locators::{LocatorConfig, Locators, Timeouts};
pub use session::SessionManager;
