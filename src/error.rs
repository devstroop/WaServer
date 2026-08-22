//! Error types for WAS (WhatsApp Server)
//!
//! Re-exports error types from models for backward compatibility.

pub use crate::domain::shared::error::{DomainError, DomainResult};
pub use crate::models::error::{Result, WhatsAppError};
