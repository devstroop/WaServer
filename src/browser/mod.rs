//! Engine Module
//!
//! Core WAS (WhatsApp Server) implementation including browser automation,
//! element locators, and the main WhatsApp engine interface.

pub mod country_codes;
mod core;
mod driver;
mod locators;
pub mod selector;
mod session;

pub use core::WhatsAppEngine;
pub use driver::{BrowserService, BrowserServiceConfig};
pub use locators::{LocatorConfig, Locators, Timeouts};
pub use selector::{parse_selector, SelectorType};
pub use session::SessionManager;
