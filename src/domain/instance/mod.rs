//! Instance Domain — pure entities, value objects, validation
//!
//! No `axum`/`tokio`/`chromiumoxide` here. Only `serde`, `chrono`, `uuid`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utoipa::ToSchema;
use uuid::Uuid;

/// Unique identifier for a WhatsApp instance (UUID)
pub type InstanceId = Uuid;

/// Instance runtime status — domain state machine
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    #[default]
    Sleeping,
    WarmingUp,
    Active,
    Error(String),
}

/// Persistent instance metadata (stored in account.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceMetadata {
    pub id: InstanceId,
    pub phone_number: Option<String>,
    pub instance_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub first_linked_at: Option<DateTime<Utc>>,
}

impl InstanceMetadata {
    pub fn new(id: InstanceId, phone_number: Option<String>, instance_name: Option<String>) -> Self {
        Self {
            id,
            phone_number,
            instance_name,
            created_at: Utc::now(),
            first_linked_at: None,
        }
    }
}

/// Instance configuration value objects (domain)
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct BrowserOverrides {
    pub headless: Option<bool>,
    #[serde(default)]
    pub extra_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceSetupConfig {
    pub id: InstanceId,
    pub phone_number: Option<String>,
    pub instance_name: Option<String>,
    pub data_dir: PathBuf,
    #[serde(default)]
    pub browser: BrowserOverrides,
}

impl Default for InstanceSetupConfig {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            phone_number: None,
            instance_name: None,
            data_dir: PathBuf::new(),
            browser: BrowserOverrides::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceConfig {
    #[schema(value_type = Option<String>, format = "uuid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<InstanceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_name: Option<String>,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,
    #[serde(default)]
    pub browser: InstanceBrowserConfig,
    #[serde(default)]
    pub rate_limits: InstanceRateLimits,
}

fn default_idle_timeout() -> u64 {
    300
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            instance_id: None,
            instance_name: None,
            idle_timeout: 300,
            browser: InstanceBrowserConfig::default(),
            rate_limits: InstanceRateLimits::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceBrowserConfig {
    #[serde(default = "default_true")]
    pub headless: bool,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub extra_args: Vec<String>,
}

impl Default for InstanceBrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            timeout_ms: 30000,
            extra_args: vec![],
        }
    }
}

fn default_timeout() -> u64 {
    30000
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceRateLimits {
    #[serde(default = "default_messages_per_minute")]
    pub messages_per_minute: u32,
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,
    #[serde(default = "default_message_cooldown")]
    pub message_cooldown_ms: u64,
}

impl Default for InstanceRateLimits {
    fn default() -> Self {
        Self {
            messages_per_minute: 60,
            requests_per_minute: 120,
            message_cooldown_ms: 1000,
        }
    }
}

fn default_messages_per_minute() -> u32 {
    60
}
fn default_requests_per_minute() -> u32 {
    120
}
fn default_message_cooldown() -> u64 {
    1000
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct UpdateBrowserConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headless: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct UpdateRateLimits {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_per_minute: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_per_minute: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_cooldown_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateInstanceConfigRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<UpdateBrowserConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<UpdateRateLimits>,
}

/// Instance status view (returned by API — will move to DTO in feat/api)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceInfo {
    #[schema(value_type = String, format = "uuid")]
    pub id: InstanceId,
    pub phone_number: Option<String>,
    pub instance_name: Option<String>,
    pub status: InstanceStatus,
    pub authorized: bool,
    pub created_at: DateTime<Utc>,
}

// --- Validation (pure domain) ---

/// Validate phone number, returns digits only without '+'
pub fn validate_phone_number(phone: &str) -> Result<String, String> {
    let phone = phone.trim();
    if phone.is_empty() {
        return Err("Phone number cannot be empty".to_string());
    }
    let digits_only: String = phone
        .strip_prefix('+')
        .unwrap_or(phone)
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    if digits_only.len() < 7 {
        return Err("Phone number too short (minimum 7 digits)".to_string());
    }
    if digits_only.len() > 15 {
        return Err("Phone number too long (maximum 15 digits)".to_string());
    }
    Ok(digits_only)
}

pub fn phone_to_dir_name(phone: &str) -> String {
    phone.chars().filter(|c| c.is_ascii_digit()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validate_phone_number() {
        assert_eq!(validate_phone_number("+1234567890").unwrap(), "1234567890");
        assert_eq!(validate_phone_number("1234567890").unwrap(), "1234567890");
        assert!(validate_phone_number("").is_err());
        assert!(validate_phone_number("123456").is_err());
        assert!(validate_phone_number("+1234567890123456").is_err());
    }
    #[test]
    fn test_phone_to_dir_name() {
        assert_eq!(phone_to_dir_name("+1234567890"), "1234567890");
    }
}
