use thiserror::Error;

/// Result type for WAS (WhatsApp Server) operations
pub type Result<T> = std::result::Result<T, WhatsAppError>;

/// Comprehensive error types for the WAS library
#[derive(Error, Debug)]
pub enum WhatsAppError {
    #[error("Browser initialization failed: {0}")]
    BrowserInit(String),

    #[error("Browser navigation failed: {0}")]
    BrowserNavigation(String),

    #[error("Browser connection lost: {0}")]
    BrowserConnection(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("QR code generation failed: {0}")]
    QrCodeGeneration(String),

    #[error("Phone authentication failed: {0}")]
    PhoneAuthentication(String),

    #[error("Message sending failed: {0}")]
    MessageSending(String),

    #[error("Contact retrieval failed: {0}")]
    ContactRetrieval(String),

    #[error("Chat navigation failed: {0}")]
    ChatNavigation(String),

    #[error("File upload failed: {0}")]
    FileUpload(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout error: operation timed out after {timeout_seconds}s: {operation}")]
    Timeout {
        operation: String,
        timeout_seconds: u64,
    },

    #[error("Invalid input: {field} - {reason}")]
    InvalidInput { field: String, reason: String },

    #[error("Service not ready: {service} is not initialized or has failed")]
    ServiceNotReady { service: String },

    #[error("Rate limit exceeded: {operation} - retry after {retry_after_seconds}s")]
    RateLimit {
        operation: String,
        retry_after_seconds: u32,
    },

    #[error("Session expired: authentication session is no longer valid")]
    SessionExpired,

    #[error("Permission denied: {operation} requires additional permissions")]
    PermissionDenied { operation: String },

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("File operation failed: {details}")]
    FileError { details: String },

    #[error("Serialization error: {details}")]
    SerializationError { details: String },
}

impl WhatsAppError {
    /// Create a timeout error
    pub fn timeout(operation: &str, timeout_seconds: u64) -> Self {
        Self::Timeout {
            operation: operation.to_string(),
            timeout_seconds,
        }
    }

    /// Create an invalid input error
    pub fn invalid_input(field: &str, reason: &str) -> Self {
        Self::InvalidInput {
            field: field.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a service not ready error
    pub fn service_not_ready(service: &str) -> Self {
        Self::ServiceNotReady {
            service: service.to_string(),
        }
    }

    /// Create a rate limit error
    pub fn rate_limit(operation: &str, retry_after_seconds: u32) -> Self {
        Self::RateLimit {
            operation: operation.to_string(),
            retry_after_seconds,
        }
    }

    /// Create a permission denied error
    pub fn permission_denied(operation: &str) -> Self {
        Self::PermissionDenied {
            operation: operation.to_string(),
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            WhatsAppError::Network(_)
                | WhatsAppError::Timeout { .. }
                | WhatsAppError::BrowserConnection(_)
                | WhatsAppError::RateLimit { .. }
        )
    }

    /// Get retry delay in seconds if this error is retryable
    pub fn retry_delay_seconds(&self) -> Option<u32> {
        match self {
            WhatsAppError::RateLimit {
                retry_after_seconds,
                ..
            } => Some(*retry_after_seconds),
            WhatsAppError::Network(_) => Some(5),
            WhatsAppError::Timeout { .. } => Some(10),
            WhatsAppError::BrowserConnection(_) => Some(15),
            _ => None,
        }
    }
}

/// Convert from anyhow::Error for backward compatibility
impl From<anyhow::Error> for WhatsAppError {
    fn from(err: anyhow::Error) -> Self {
        WhatsAppError::Internal(err.to_string())
    }
}

/// Convert from chromiumoxide errors
impl From<chromiumoxide::error::CdpError> for WhatsAppError {
    fn from(err: chromiumoxide::error::CdpError) -> Self {
        WhatsAppError::BrowserConnection(err.to_string())
    }
}

/// Convert from config errors
impl From<config::ConfigError> for WhatsAppError {
    fn from(err: config::ConfigError) -> Self {
        WhatsAppError::Configuration(err.to_string())
    }
}

/// Convert from serde_json errors
impl From<serde_json::Error> for WhatsAppError {
    fn from(err: serde_json::Error) -> Self {
        WhatsAppError::Internal(format!("JSON error: {}", err))
    }
}

/// Convert from rusqlite errors
impl From<rusqlite::Error> for WhatsAppError {
    fn from(err: rusqlite::Error) -> Self {
        WhatsAppError::Internal(format!("Database error: {}", err))
    }
}

/// Convert from std::io errors
impl From<std::io::Error> for WhatsAppError {
    fn from(err: std::io::Error) -> Self {
        WhatsAppError::FileError {
            details: err.to_string(),
        }
    }
}

/// Convert from tokio::sync::AcquireError
impl From<tokio::sync::AcquireError> for WhatsAppError {
    fn from(err: tokio::sync::AcquireError) -> Self {
        WhatsAppError::Internal(format!("Semaphore acquire error: {}", err))
    }
}

// ============================================================================
// Authentication Errors
// ============================================================================

/// Errors that can occur during authentication token operations
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),
    #[error("Password hashing failed: {0}")]
    HashingFailed(String),
}
