//! Messaging Policy — pure domain validation extracted from `handlers/api/chat.rs:288` and `models/instance::validate_phone_number`

use crate::domain::messaging::{MediaType, MessageStatus};

/// Validate phone per E.164 (delegates to domain/instance)
pub trait ValidatePhone {
    fn validate(&self, phone: &str) -> Result<String, String>;
}

pub struct E164Validator;
impl ValidatePhone for E164Validator {
    fn validate(&self, phone: &str) -> Result<String, String> {
        crate::domain::instance::validate_phone_number(phone)
    }
}

/// Send policy — rate limits, cooldown, media checks (pure)
pub struct SendPolicy {
    pub max_per_minute: u32,
    pub cooldown_ms: u64,
}

impl SendPolicy {
    pub fn default_for_instance() -> Self {
        Self {
            max_per_minute: 60,
            cooldown_ms: 1000,
        }
    }

    pub fn can_send(&self, _media: MediaType, _status: MessageStatus) -> bool {
        true // placeholder — will check media type, queue depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_e164_validator() {
        let v = E164Validator;
        assert_eq!(v.validate("+1234567890").unwrap(), "1234567890");
        assert!(v.validate("123").is_err());
    }
    #[test]
    fn test_send_policy() {
        let p = SendPolicy::default_for_instance();
        assert!(p.can_send(MediaType::None, MessageStatus::Pending));
    }
}
