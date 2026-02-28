//! WAS (WhatsApp Server) Services
//!
//! Core business logic services for WhatsApp automation.

pub mod auth;
pub mod database;
pub mod webhook;
pub mod whatsapp;

// Re-exports for convenience
pub use auth::{AuthCheckResult, AuthService, AuthServiceTrait};
pub use database::Database;
pub use webhook::{WebhookEvent, WebhookMessageData, WebhookService};
pub use whatsapp::{ChatService, ChatServiceTrait, InstanceManager, InstanceService};
