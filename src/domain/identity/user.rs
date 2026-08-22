//! User entity — pure domain, extracted from `models/user.rs:17` + `handlers/api/users.rs:119` (#9)
//! Validation lives here, not in handler. No DB/crypto.

use super::UserRole;
use crate::domain::shared::error::{DomainError, DomainResult};

/// Pure domain User entity (no password_hash exposure)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub role: UserRole,
    pub is_active: bool,
}

impl User {
    pub fn new(
        id: impl Into<String>,
        username: impl Into<String>,
        role: UserRole,
    ) -> DomainResult<Self> {
        let username = username.into();
        validate_username(&username)?;
        Ok(Self {
            id: id.into(),
            username,
            email: None,
            role,
            is_active: true,
        })
    }
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }
}

/// Validate username — mirrors `handlers/api/users.rs:119`
pub fn validate_username(username: &str) -> DomainResult<()> {
    if username.trim().is_empty() {
        return Err(DomainError::Validation("username cannot be empty".into()));
    }
    if username.len() < 3 {
        return Err(DomainError::Validation(
            "username must be at least 3 characters".into(),
        ));
    }
    Ok(())
}

/// Validate password — mirrors `handlers/api/users.rs:132`
pub fn validate_password(password: &str) -> DomainResult<()> {
    if password.len() < 8 {
        return Err(DomainError::Validation(
            "password must be at least 8 characters".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validate_username() {
        assert!(validate_username("alice").is_ok());
        assert!(validate_username("").is_err());
        assert!(validate_username("ab").is_err());
    }
    #[test]
    fn test_validate_password() {
        assert!(validate_password("longenough").is_ok());
        assert!(validate_password("short").is_err());
    }
    #[test]
    fn test_user_new() {
        let u = User::new("id1", "alice", UserRole::User).unwrap();
        assert_eq!(u.username, "alice");
        assert!(User::new("id2", "", UserRole::User).is_err());
    }
}
