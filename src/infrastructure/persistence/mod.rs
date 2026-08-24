//! Database Service Module
//!
//! Embedded SQLite (file-based) for persistent instance storage.

pub mod instance_repo;
pub mod schema;
pub mod service;
pub mod token_repo;
pub mod user_repo;

pub use instance_repo::SqliteInstanceStore;
pub use service::Database;
pub use token_repo::SqliteTokenStore;
pub use user_repo::SqliteUserStore;
