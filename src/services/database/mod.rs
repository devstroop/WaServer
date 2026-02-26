//! Database Service Module
//!
//! Embedded SQLite (file-based) for persistent account storage.

pub mod schema;
pub mod service;

pub use service::Database;
