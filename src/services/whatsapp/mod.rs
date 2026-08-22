//! WhatsApp Service Module
//!
//! Core WhatsApp automation services: instance management and chat handling.
//! Use InstanceManager to create and manage InstanceService accounts.

pub mod chat;
pub mod instance;
pub mod instance_auth;
pub mod instance_lifecycle;
pub mod instance_manager;

// Re-exports for convenience
pub use chat::{ChatService, ChatServiceTrait};
pub use instance::InstanceService;
pub use instance_manager::InstanceManager;
