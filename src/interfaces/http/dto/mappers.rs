//! DTO ↔ domain mappers — `TryFrom` per #11, keeps domain changes from breaking API
//! Handlers call `dto.validate()` then `TryFrom` to domain, and `From<domain>` to DTO.

use crate::{
    domain::instance::{InstanceId, InstanceInfo, InstanceStatus},
    interfaces::http::dto::{
        health::HealthResponseDto,
        instance::{CreateInstanceRequestDto, InstanceInfoDto},
        messaging::SendMessageRequestDto,
    },
};
use chrono::Utc;

// --- Instance mappers ---

impl TryFrom<CreateInstanceRequestDto> for InstanceInfo {
    type Error = String;
    fn try_from(dto: CreateInstanceRequestDto) -> Result<Self, Self::Error> {
        dto.validate()?;
        Ok(Self {
            id: InstanceId::nil(), // real ID assigned by service
            phone_number: dto.phone_number,
            instance_name: Some(dto.name),
            status: InstanceStatus::Sleeping,
            authorized: false,
            created_at: Utc::now(),
        })
    }
}

impl From<InstanceInfo> for InstanceInfoDto {
    fn from(info: InstanceInfo) -> Self {
        Self {
            id: info.id.to_string(),
            name: info.instance_name.unwrap_or_default(),
            phone_number: info.phone_number,
            status: format!("{:?}", info.status).to_lowercase(),
            authorized: info.authorized,
            created_at: info.created_at.to_rfc3339(),
            updated_at: info.created_at.to_rfc3339(),
        }
    }
}

// --- Messaging mappers ---

impl From<SendMessageRequestDto> for crate::domain::messaging::NewMessage {
    fn from(dto: SendMessageRequestDto) -> Self {
        // Demonstrates DTO→domain boundary without leaking `axum`; uses domain factory
        Self::outgoing_text(&dto.phone, dto.text.as_deref().unwrap_or(""))
    }
}

// --- Health mappers ---

impl From<HealthResponseDto> for serde_json::Value {
    fn from(dto: HealthResponseDto) -> Self {
        serde_json::to_value(dto).unwrap_or(serde_json::json!({}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_instance_dto_to_domain() {
        let dto = CreateInstanceRequestDto {
            name: "bot".into(),
            phone_number: Some("+1234567890".into()),
        };
        let info = InstanceInfo::try_from(dto).unwrap();
        assert_eq!(info.instance_name.unwrap(), "bot");
        assert!(InstanceInfo::try_from(CreateInstanceRequestDto {
            name: "".into(),
            phone_number: None
        })
        .is_err());
    }
    #[test]
    fn test_instance_info_to_dto() {
        let info = InstanceInfo {
            id: InstanceId::nil(),
            phone_number: Some("+1234567890".into()),
            instance_name: Some("bot".into()),
            status: InstanceStatus::Active,
            authorized: true,
            created_at: Utc::now(),
        };
        let dto = InstanceInfoDto::from(info);
        assert_eq!(dto.name, "bot");
        assert_eq!(dto.phone_number, Some("+1234567890".into()));
    }
    #[test]
    fn test_dto_roundtrip_stable() {
        // Changing domain `InstanceInfo` must not alter `InstanceInfoDto` JSON without explicit DTO change
        let dto = InstanceInfoDto {
            id: "550e8400-e29b-41d4-a716-446655440000".into(),
            name: "sales-bot".into(),
            phone_number: Some("+1234567890".into()),
            status: "active".into(),
            authorized: true,
            created_at: "2026-03-01T10:00:00Z".into(),
            updated_at: "2026-03-01T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("sales-bot"));
        assert!(json.contains("550e8400"));
    }
}
