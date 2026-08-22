//! Secret handling — extracted from `src/middleware/auth.rs:73` and `models/config.rs:188` `validate`
//! Pure, no DB — checks default secret, length, constant-time compare.

use sha2::{Digest, Sha256};

/// Hash token via SHA256 (same as `services/auth/auth.rs` `hash_token`)
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Validate secret at boot — fails in prod if default or weak
pub struct SecretValidator;

impl SecretValidator {
    pub fn validate(secret: &str, is_development: bool) -> Result<(), String> {
        if secret == "change-this-secret-key-in-production" {
            if is_development {
                return Ok(());
            }
            return Err("Please change the default secret_key in configuration".into());
        }
        if secret.len() < 16 {
            return Err("secret_key must be at least 16 characters".into());
        }
        Ok(())
    }

    /// Constant-time compare to avoid timing leak
    pub fn constant_time_eq(a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut diff = 0u8;
        for (x, y) in a.bytes().zip(b.bytes()) {
            diff |= x ^ y;
        }
        diff == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hash_token() {
        assert_eq!(hash_token("a"), hash_token("a"));
        assert_ne!(hash_token("a"), "a");
    }
    #[test]
    fn test_validate_secret() {
        assert!(SecretValidator::validate("short", false).is_err());
        assert!(SecretValidator::validate("change-this-secret-key-in-production", true).is_ok());
        assert!(SecretValidator::validate("change-this-secret-key-in-production", false).is_err());
        assert!(SecretValidator::validate("a-very-strong-secret-123", false).is_ok());
    }
    #[test]
    fn test_constant_time_eq() {
        assert!(SecretValidator::constant_time_eq("abc", "abc"));
        assert!(!SecretValidator::constant_time_eq("abc", "abd"));
        assert!(!SecretValidator::constant_time_eq("abc", "ab"));
    }
}
