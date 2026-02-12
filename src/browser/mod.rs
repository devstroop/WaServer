//! Browser Module
//!
//! Handles Chrome/Chromium browser automation for WhatsApp Web.
//! This module provides browser lifecycle management, page navigation,
//! and element locators.

mod driver;
mod locators;

pub use driver::BrowserService;
pub use locators::{Locators, Timeouts};
