//! Database Service Module
//!
//! Embedded SQLite (file-based) for persistent instance storage.

pub mod schema;
pub mod service;

pub use service::Database;
