pub mod auth;
pub mod chat;
pub mod domain; // New: Domain models for library use

// Re-export domain models for easier access
pub use domain::*;
