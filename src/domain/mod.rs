//! Pure Domain Layer
//!
//! No `axum`, `tokio`, `chromiumoxide` or `rusqlite` deps allowed here.
//! Only `serde`, `chrono`, `uuid`, `thiserror`, `anyhow`.

pub mod identity;
pub mod instance;
pub mod messaging;
pub mod shared;

// Re-exports for convenience
pub use identity::{InstancePermission, UserRole};
pub use instance::{InstanceId, InstanceMetadata, InstanceStatus};
pub use messaging::{MediaType, Message, MessageStatus, SELF_JID};
pub use shared::error::{DomainError, DomainResult};
