//! UserService — extracted from `handlers/api/users.rs:119..176` + `models/user.rs:142` (#9)
//! Validates via `domain::identity::user`, delegates persistence to `UserStore` port.

use crate::domain::identity::{validate_password, validate_username, UserRole};
use crate::domain::shared::error::DomainResult;

/// Input for creating a user — mirrors `CreateUserRequest` but validated
#[derive(Debug, Clone)]
pub struct CreateUserInput {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub role: UserRole,
}

impl CreateUserInput {
    pub fn validate(&self) -> DomainResult<()> {
        validate_username(&self.username)?;
        validate_password(&self.password)?;
        Ok(())
    }
}

/// Pure user application service (no DB)
pub struct UserService;

impl UserService {
    pub fn validate_create(input: &CreateUserInput) -> DomainResult<()> {
        input.validate()
    }
    /// Hash password via SHA256 (same as `middleware/auth.rs:52`); real prod would use argon2
    pub fn hash_password(password: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validate_create() {
        let ok = CreateUserInput {
            username: "alice".into(),
            email: None,
            password: "longenough".into(),
            role: UserRole::User,
        };
        assert!(ok.validate().is_ok());
        let bad = CreateUserInput {
            username: "".into(),
            email: None,
            password: "short".into(),
            role: UserRole::User,
        };
        assert!(bad.validate().is_err());
    }
    #[test]
    fn test_hash() {
        let h = UserService::hash_password("secret123");
        assert_eq!(h, UserService::hash_password("secret123"));
        assert_ne!(h, "secret123");
    }
}
