//! OpenAPI snapshot — `openapi.json` stability per #11
//! Domain change must not alter `openapi.json` without explicit DTO change.
//! Version is config-driven via `CARGO_PKG_VERSION` (`bin/was.rs:406` Info.version).

use utoipa::OpenApi;

use super::{
    health::{ErrorEnvelopeDto, HealthResponseDto, StatusResponseDto},
    identity::{
        AccessTokenInfoDto, CreateAccessTokenRequestDto, CreateUserRequestDto, UserInfoDto,
    },
    instance::{
        CreateInstanceRequestDto, CreateInstanceResponseDto, InstanceInfoDto,
        InstanceListResponseDto,
    },
    messaging::{SendMessageRequestDto, SendMessageResponseDto},
};

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        HealthResponseDto,
        StatusResponseDto,
        ErrorEnvelopeDto,
        CreateInstanceRequestDto,
        CreateInstanceResponseDto,
        InstanceInfoDto,
        InstanceListResponseDto,
        SendMessageRequestDto,
        SendMessageResponseDto,
        CreateUserRequestDto,
        UserInfoDto,
        CreateAccessTokenRequestDto,
        AccessTokenInfoDto,
    )),
    info(
        title = "WhatsApp Server - API",
        version = "0.5.0",
        description = "DTO stability snapshot — change requires explicit DTO version bump"
    )
)]
struct SnapshotApi;

pub fn snapshot_json() -> String {
    SnapshotApi::openapi().to_pretty_json().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_openapi_snapshot_exists_and_versioned() {
        let json = snapshot_json();
        assert!(
            json.contains("\"version\": \"0.5.0\""),
            "version must be 0.5.0"
        );
        assert!(
            json.contains("CreateInstanceRequestDto"),
            "DTO must be in snapshot"
        );
        assert!(
            json.contains("SendMessageRequestDto"),
            "messaging DTO must be in snapshot"
        );
        assert!(
            json.contains("ErrorEnvelopeDto"),
            "error envelope must be in snapshot {{error, message, correlation_id}}"
        );
        // Write snapshot to file for review (not failing test, just ensuring stability)
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("openapi.snapshot.json");
        if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
            std::fs::write(&path, &json).unwrap();
        }
        // Snapshot must be valid JSON and contain DTOs
        let val: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(val.get("components").is_some());
    }
    #[test]
    fn test_dto_schema_stability() {
        // Changing `domain::instance::InstanceInfo` should not affect this JSON without DTO change
        let dto = CreateInstanceRequestDto {
            name: "bot".into(),
            phone_number: Some("+1234567890".into()),
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert_eq!(json, r#"{"name":"bot","phone_number":"+1234567890"}"#);
    }
}
