//! TokenService — extracted from `handlers/api/users.rs:416..494` + `domain/identity/token.rs` (#9)
//! Generates `was_<uuid>`, hashes via SHA256, builds expiry.

use crate::domain::identity::{TokenExpiry, TokenName};
use crate::domain::shared::error::DomainResult;

/// Input for creating a token
#[derive(Debug, Clone)]
pub struct CreateTokenInput {
    pub name: String,
    pub expires_in_days: Option<u32>,
}

impl CreateTokenInput {
    pub fn validate(&self) -> DomainResult<()> {
        TokenName::new(self.name.clone())?;
        Ok(())
    }
    pub fn expiry(&self, now_unix: i64) -> TokenExpiry {
        TokenExpiry::from_days(self.expires_in_days, now_unix)
    }
}

pub struct TokenService;

impl TokenService {
    pub fn generate_token() -> String {
        crate::domain::identity::generate_raw_token()
    }
    pub fn hash(token: &str) -> String {
        crate::application::auth::hash_token(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_input_validate() {
        assert!(CreateTokenInput {
            name: "default".into(),
            expires_in_days: Some(30)
        }
        .validate()
        .is_ok());
        assert!(CreateTokenInput {
            name: "".into(),
            expires_in_days: None
        }
        .validate()
        .is_err());
    }
    #[test]
    fn test_generate_and_hash() {
        let t = TokenService::generate_token();
        assert!(t.starts_with("was_"));
        assert_ne!(TokenService::hash(&t), t);
    }
}
