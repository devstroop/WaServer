//! WAS (WhatsApp Server) Services
//!
//! Core business logic services for WhatsApp automation.

pub mod auth;
pub mod database;
pub mod messaging_ports;
pub mod whatsapp;

// Re-exports for convenience
pub use auth::{AuthCheckResult, AuthService, AuthServiceTrait};
pub use database::Database;
pub use messaging_ports::ManagerBrowserAdapter;
pub use whatsapp::{
    ChatService, ChatServiceTrait, InstanceManager, InstanceService, StatusSnapshot,
};
