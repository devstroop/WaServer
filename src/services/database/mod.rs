//! Database Service Module
//!
//! SQLite-based persistence for messages, contacts, conversations, settings, and users.

mod contacts;
mod conversations;
mod messages;
mod queue;
mod service;
mod session;
mod settings;

// Re-export DatabaseService and constants
pub use service::{DatabaseService, CONTACT_BATCH_SIZE};

// Re-export message models from models module
pub use crate::models::message::{
    is_self, ChatSettings, Contact, Conversation, MediaType, Message, MessageDebugTimings,
    MessageStatus, NewMessage, QueueStatus, SELF_JID,
};