//! WhatsApp Service Module
//!
//! Core WhatsApp automation services: account management and chat handling.
//! Use AccountManager to create and manage WhatsAppAccount instances.

pub mod account;
pub mod account_manager;
pub mod chat;

// Re-exports for convenience
pub use account::WhatsAppAccount;
pub use account_manager::AccountManager;
pub use chat::{ChatService, ChatServiceTrait};
