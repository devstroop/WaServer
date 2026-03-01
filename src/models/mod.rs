pub mod auth;
pub mod chat;
pub mod config;
pub mod domain;
pub mod environment;
pub mod error;
pub mod instance;
pub mod message;
pub mod session;
pub mod user;

// Re-export domain models for easier access
pub use domain::*;

// Re-export auth types
pub use auth::AuthenticatedUser;

// Re-export user types
pub use user::{
    AccessTokenInfo, AccessTokenRecord, AssignInstanceRequest, CreateAccessTokenRequest,
    CreateAccessTokenResponse, CreateUserRequest, CreateUserResponse, InstanceOwnerRecord,
    InstancePermission, ListAccessTokensResponse, ListUsersResponse, LoginRequest,
    LoginResponse, RegisterUserRequest, UpdateUserRequest, UserInfo, UserInstancesResponse,
    UserRecord, UserRole,
};

// Re-export instance types
pub use instance::{
    phone_to_dir_name,
    validate_phone_number,
    CreateInstanceRequest,
    CreateInstanceResponse,
    DeleteInstanceQuery,
    DeleteInstanceResponse,
    InstanceActionResponse,
    InstanceId,
    InstanceInfo,
    InstanceListResponse,
    InstanceMetadata,
    InstanceSetupConfig,
    InstanceStatus,
    // API query/response types
    ListInstancesQuery,
    ProfileInfo,
    WhatsAppStatusResponse,
};
