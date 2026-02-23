pub mod account;
pub mod auth;
pub mod chat;
pub mod config;
pub mod domain;
pub mod environment;
pub mod error;
pub mod message;
pub mod session;

// Re-export domain models for easier access
pub use domain::*;

// Re-export auth types
pub use auth::AuthenticatedUser;

// Re-export account types
pub use account::{
    phone_to_dir_name,
    validate_phone_number,
    AccountActionResponse,
    AccountSetupConfig,
    AccountId,
    AccountInfo,
    AccountListResponse,
    AccountMetadata,
    AccountStatus,
    CreateAccountRequest,
    CreateAccountResponse,
    DeleteAccountQuery,
    DeleteAccountResponse,
    // API query/response types
    ListAccountsQuery,
    PhoneLinkRequest,
    ProfileInfo,
    WhatsAppStatusResponse,
};
