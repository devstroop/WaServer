//! Identity Application Services — part of #9
//!
//! Splits `handlers/api/users.rs:845` god file: `user_service`, `token_service`, `rbac`.
//! Each service depends only on `domain::identity` + ports, no `rusqlite`/`axum`.

pub mod rbac;
pub mod token_service;
pub mod user_service;

pub use rbac::{PermissionCheck, RbacService};
pub use token_service::{CreateTokenInput, TokenService};
pub use user_service::{CreateUserInput, UserService};
