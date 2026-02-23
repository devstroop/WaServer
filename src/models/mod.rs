pub mod instance;
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

// Re-export instance types
pub use instance::{
    phone_to_dir_name,
    validate_phone_number,
    InstanceActionResponse,
    InstanceSetupConfig,
    InstanceId,
    InstanceInfo,
    InstanceListResponse,
    InstanceMetadata,
    InstanceStatus,
    CreateInstanceRequest,
    CreateInstanceResponse,
    DeleteInstanceQuery,
    DeleteInstanceResponse,
    // API query/response types
    ListInstancesQuery,
    PhoneLinkRequest,
    PrivacySettings,
    ProfileInfo,
    WhatsAppStatusResponse,
};
