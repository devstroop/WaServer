//! Instance DTOs — versioned, `utoipa::Schema`, validated
//! Extracted from `models/instance.rs` + `handlers/api/instances.rs:36` (part of #11)
//! Handlers map `InstanceInfo` (domain) ↔ DTO via `From`.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// `POST /api/v1/instances` request — validated at DTO boundary
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateInstanceRequestDto {
    /// Friendly name
    pub name: String,
    /// Phone in E.164 (optional)
    pub phone_number: Option<String>,
}

impl CreateInstanceRequestDto {
    /// Validate phone via `domain::instance::InstancePhone::validate` shape
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("name is required".into());
        }
        if let Some(phone) = &self.phone_number {
            if !phone.starts_with('+') || phone.len() < 8 {
                return Err("phone_number must be E.164 (+...)".into());
            }
        }
        Ok(())
    }
}

/// `POST /api/v1/instances` response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateInstanceResponseDto {
    pub instance_id: String,
    pub name: String,
    pub message: String,
}

/// `GET /api/v1/instances` list envelope
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceListResponseDto {
    pub instances: Vec<InstanceInfoDto>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceInfoDto {
    pub id: String,
    pub name: String,
    pub phone_number: Option<String>,
    pub status: String,
    pub authorized: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_dto_validate() {
        let ok = CreateInstanceRequestDto {
            name: "bot".into(),
            phone_number: Some("+12345678".into()),
        };
        assert!(ok.validate().is_ok());
        let bad = CreateInstanceRequestDto {
            name: "".into(),
            phone_number: None,
        };
        assert!(bad.validate().is_err());
    }
    #[test]
    fn test_dto_serde() {
        let dto = CreateInstanceResponseDto {
            instance_id: "id".into(),
            name: "n".into(),
            message: "ok".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("instance_id"));
    }
}
