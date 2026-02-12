//! WhatsApp Engine Services
//!
//! Core business logic services for WhatsApp automation.

pub mod auth;
pub mod chat;
pub mod database;
pub mod webhook;
pub mod whatsapp;

// Re-exports for convenience
pub use auth::{AuthService, AuthServiceTrait};
pub use chat::{ChatService, ChatServiceTrait};
pub use database::{
    is_self, Contact, DatabaseService, MediaType, Message, MessageStatus, NewMessage, SELF_JID,
};
pub use webhook::{WebhookEvent, WebhookMessageData, WebhookService};
