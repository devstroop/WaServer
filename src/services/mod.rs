//! WhatsApp Engine Services
//!
//! Core business logic services for WhatsApp automation.

pub mod auth;
pub mod chat;
pub mod mcp_client;
pub mod whatsapp;

// Re-exports for convenience
pub use auth::{AuthService, AuthServiceTrait};
pub use chat::{ChatService, ChatServiceTrait};
