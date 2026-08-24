//! Health DTOs — versioned copy of `handlers/api/health.rs` responses
//! Keeps OpenAPI stable if `domain` changes.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// `GET /api/health` response — mirrors `handlers/api/health::HealthResponse`
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponseDto {
    pub status: String,
    pub timestamp: u64,
    pub version: String,
    pub uptime_seconds: u64,
    pub instances_count: usize,
}

/// `GET /api/ready|live` — simple status envelope
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StatusResponseDto {
    pub status: String,
}

/// Standard error envelope — `{error, message, correlation_id}` (reuse `middleware/correlation_id`)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorEnvelopeDto {
    pub error: String,
    pub message: String,
    pub correlation_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_health_dto_serde() {
        let dto = HealthResponseDto {
            status: "healthy".into(),
            timestamp: 1,
            version: "0.5.1".into(),
            uptime_seconds: 10,
            instances_count: 1,
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: HealthResponseDto = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, "0.5.1");
    }
}
