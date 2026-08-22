use thiserror::Error;

/// Pure domain errors — no infra details (DB, browser, network)
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("invalid input: {field} - {reason}")]
    InvalidInput { field: String, reason: String },

    #[error("not found: {entity} '{id}'")]
    NotFound { entity: String, id: String },

    #[error("conflict: {entity} '{id}' already exists")]
    Conflict { entity: String, id: String },

    #[error("permission denied: {operation}")]
    PermissionDenied { operation: String },

    #[error("rate limited: {operation} - retry after {retry_after_seconds}s")]
    RateLimited {
        operation: String,
        retry_after_seconds: u32,
    },

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("internal: {0}")]
    Internal(String),
}

pub type DomainResult<T> = Result<T, DomainError>;

impl DomainError {
    pub fn invalid_input(field: &str, reason: &str) -> Self {
        Self::InvalidInput {
            field: field.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn not_found(entity: &str, id: &str) -> Self {
        Self::NotFound {
            entity: entity.to_string(),
            id: id.to_string(),
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, DomainError::RateLimited { .. })
    }
}
