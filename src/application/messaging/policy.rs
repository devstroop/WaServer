//! Messaging Policy — pure domain validation extracted from `handlers/api/chat.rs:288` and `models/instance::validate_phone_number`

use crate::domain::messaging::{MediaType, MessageStatus};
use crate::domain::shared::error::{DomainError, DomainResult};

/// Validate phone per E.164 (delegates to domain/instance)
pub trait ValidatePhone {
    fn validate(&self, phone: &str) -> Result<String, DomainError>;
}

pub struct E164Validator;
impl ValidatePhone for E164Validator {
    fn validate(&self, phone: &str) -> Result<String, DomainError> {
        crate::domain::instance::validate_phone_number(phone)
            .map_err(|e| DomainError::Validation(e.to_string()))
    }
}

/// Typed policy error
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum SendPolicyError {
    #[error("media type not allowed: {0:?}")]
    MediaNotAllowed(MediaType),
    #[error("queue full: status {0:?}")]
    QueueFull(MessageStatus),
}

/// Send policy — rate limits, cooldown, media checks (pure)
#[derive(Clone)]
pub struct SendPolicy {
    pub max_per_minute: u32,
    pub cooldown_ms: u64,
    pub allowed_media: Vec<MediaType>,
    pub max_queue_depth: usize,
}

impl Default for SendPolicy {
    fn default() -> Self {
        Self::default_for_instance()
    }
}

impl SendPolicy {
    pub fn default_for_instance() -> Self {
        Self {
            max_per_minute: 60,
            cooldown_ms: 1000,
            allowed_media: vec![
                MediaType::None,
                MediaType::Image,
                MediaType::Video,
                MediaType::Voice,
                MediaType::Document,
            ],
            max_queue_depth: 100,
        }
    }

    pub fn can_send(
        &self,
        media: MediaType,
        status: MessageStatus,
        queue_depth: usize,
        last_send_ms: Option<u64>,
        now_ms: u64,
    ) -> DomainResult<()> {
        if !self.allowed_media.contains(&media) {
            return Err(DomainError::Validation(format!(
                "media not allowed: {media:?}"
            )));
        }
        if matches!(status, MessageStatus::Processing) && queue_depth >= self.max_queue_depth {
            return Err(DomainError::Validation(format!(
                "queue full: {queue_depth} >= {}",
                self.max_queue_depth
            )));
        }
        if let Some(last) = last_send_ms {
            if now_ms < last + self.cooldown_ms {
                return Err(DomainError::RateLimited {
                    operation: "send".to_string(),
                    retry_after_seconds: ((last + self.cooldown_ms - now_ms) / 1000) as u32 + 1,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_e164_validator_ok() {
        let v = E164Validator;
        assert_eq!(v.validate("+1234567890").unwrap(), "1234567890");
    }
    #[test]
    fn test_e164_validator_err() {
        let v = E164Validator;
        assert!(v.validate("123").is_err());
    }
    #[test]
    fn test_send_policy_ok() {
        let p = SendPolicy::default_for_instance();
        assert!(p
            .can_send(MediaType::None, MessageStatus::Pending, 0, None, 0)
            .is_ok());
    }
    #[test]
    fn test_send_policy_media_not_allowed() {
        let mut p = SendPolicy::default_for_instance();
        p.allowed_media = vec![MediaType::None];
        assert!(p
            .can_send(MediaType::Image, MessageStatus::Pending, 0, None, 0)
            .is_err());
    }
    #[test]
    fn test_send_policy_queue_full() {
        let p = SendPolicy {
            max_queue_depth: 1,
            ..SendPolicy::default_for_instance()
        };
        assert!(p
            .can_send(MediaType::None, MessageStatus::Processing, 1, None, 0)
            .is_err());
    }
    #[test]
    fn test_send_policy_cooldown() {
        let p = SendPolicy {
            cooldown_ms: 1000,
            ..SendPolicy::default_for_instance()
        };
        assert!(p
            .can_send(MediaType::None, MessageStatus::Pending, 0, Some(1000), 1500)
            .is_err());
        assert!(p
            .can_send(MediaType::None, MessageStatus::Pending, 0, Some(1000), 2500)
            .is_ok());
    }
}
