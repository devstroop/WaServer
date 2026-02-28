#[allow(clippy::module_inception)]
pub mod auth;

// Re-export auth types for easier access
pub use auth::{AuthCheckResult, AuthService, AuthServiceTrait};
