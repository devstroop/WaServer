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
    AccountConfig, AccountId, AccountInfo, AccountMetadata, AccountStatus,
    CreateAccountRequest, CreateAccountResponse, AccountListResponse,
    WhatsAppStatusResponse, ProfileInfo, PrivacySettings, 
    validate_phone_number, phone_to_dir_name,
    // API query/response types
    ListAccountsQuery, DeleteAccountResponse, DeleteAccountQuery, 
    AccountActionResponse, PhoneLinkRequest,
};
