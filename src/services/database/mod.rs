//! Database Service Module
//!
//! Embedded SurrealDB (file-based) for persistent account storage.

pub mod schema;
pub mod service;

pub use service::Database;
