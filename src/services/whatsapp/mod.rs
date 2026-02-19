//! WhatsApp Service Module
//!
//! Core WhatsApp automation services: account management and chat handling.
//! WhatsAppService has been removed - use AccountManager with WhatsAppAccount instead.

pub mod account;
pub mod account_manager;
pub mod chat;

// Re-exports for convenience
pub use account::WhatsAppAccount;
pub use account_manager::AccountManager;
pub use chat::{ChatService, ChatServiceTrait};
