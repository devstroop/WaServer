//! WhatsApp Service Module
//!
//! Core WhatsApp automation services: instance management and chat handling.
//! Use InstanceManager to create and manage WhatsAppInstance instances.

pub mod instance;
pub mod instance_manager;
pub mod chat;

// Re-exports for convenience
pub use instance::WhatsAppInstance;
pub use instance_manager::InstanceManager;
pub use chat::{ChatService, ChatServiceTrait};
