//! Access token domain — pure value objects, extracted from `models/user.rs:62` + `application/auth/token.rs:18` (#9)
//! No chrono/DB in domain; expiry as `Option<i64>` unix timestamp for purity.

use crate::domain::shared::error::{DomainError, DomainResult};

/// Pure token name value object
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenName(pub String);

impl TokenName {
    pub fn new(name: impl Into<String>) -> DomainResult<Self> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DomainError::Validation("token name cannot be empty".into()));
        }
        if name.len() > 64 {
            return Err(DomainError::Validation("token name too long".into()));
        }
        Ok(Self(name))
    }
}

/// Expiry as optional unix timestamp (pure, no chrono)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenExpiry(pub Option<i64>);

impl TokenExpiry {
    pub fn from_days(days: Option<u32>, now_unix: i64) -> Self {
        match days {
            Some(d) => Self(Some(now_unix + (d as i64) * 86400)),
            None => Self(None),
        }
    }
    pub fn is_expired(&self, now_unix: i64) -> bool {
        match self.0 {
            Some(exp) => now_unix > exp,
            None => false,
        }
    }
}

/// Generate raw token string — format `was_<uuid>` like `handlers/api/users.rs:31`
pub fn generate_raw_token() -> String {
    format!("was_{}", uuid::Uuid::new_v4().to_string().replace("-", ""))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_token_name() {
        assert!(TokenName::new("default").is_ok());
        assert!(TokenName::new("").is_err());
    }
    #[test]
    fn test_expiry() {
        let now = 1000;
        let exp = TokenExpiry::from_days(Some(1), now);
        assert!(!exp.is_expired(now + 100));
        assert!(exp.is_expired(now + 86401));
        assert!(!TokenExpiry(None).is_expired(now + 9999999));
    }
    #[test]
    fn test_generate() {
        let t = generate_raw_token();
        assert!(t.starts_with("was_"));
    }
}
