//! WAS (WhatsApp Server) Services
//!
//! Core business logic services for WhatsApp automation.

pub mod auth;
pub mod database;
pub mod webhook;
pub mod whatsapp;

// Re-exports for convenience
pub use auth::{AuthCheckResult, AuthError, AuthService, AuthServiceTrait, AuthTokenService};
pub use database::{
    is_self, Contact, DatabaseService, MediaType, Message, MessageStatus, NewMessage, SELF_JID,
};
pub use webhook::{WebhookEvent, WebhookMessageData, WebhookService};
pub use whatsapp::{AccountManager, ChatService, ChatServiceTrait, WhatsAppAccount};