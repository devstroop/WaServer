//! Messaging DTOs — `POST /api/v1/instances/:id/send` contract (part of #7 #11)
//! Stabilizes `SendMessageRequest/Response` under `interfaces/http/dto` per #7 acceptance.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Query params for `POST /api/v1/instances/:id/send?phone=&text=`
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SendMessageRequestDto {
    /// Recipient phone E.164
    pub phone: String,
    /// Message text or caption (at least one of text/file required)
    pub text: Option<String>,
}

impl SendMessageRequestDto {
    pub fn validate(&self) -> Result<(), String> {
        if self.phone.trim().is_empty() {
            return Err("phone is required".into());
        }
        if !self.phone.starts_with('+') {
            return Err("phone must be E.164".into());
        }
        Ok(())
    }
}

/// Response for successful send
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SendMessageResponseDto {
    pub success: bool,
    pub message_id: String,
    pub phone: String,
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dto_validate() {
        assert!(SendMessageRequestDto {
            phone: "+1234567890".into(),
            text: Some("hi".into())
        }
        .validate()
        .is_ok());
        assert!(SendMessageRequestDto {
            phone: "123".into(),
            text: None
        }
        .validate()
        .is_err());
    }
}
