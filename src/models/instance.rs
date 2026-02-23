//! Instance Models
//!
//! Types for multi-instance WhatsApp management.
//! Instance ID is a UUID, phone_number is a unique field in E.164 format.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utoipa::ToSchema;
use uuid::Uuid;

/// Unique identifier for a WhatsApp instance (UUID)
pub type InstanceId = Uuid;

/// Instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceSetupConfig {
    /// Unique instance identifier (UUID)
    pub id: InstanceId,
    /// Phone number in E.164 format (populated after WhatsApp login)
    pub phone_number: Option<String>,
    /// Optional display name for the instance (populated after WhatsApp login)
    pub display_name: Option<String>,
    /// Data directory for this instance (browser profile, database, etc.)
    pub data_dir: PathBuf,
    /// Browser configuration overrides
    #[serde(default)]
    pub browser: BrowserOverrides,
    /// Whether to auto-start the browser on server startup
    #[serde(default)]
    pub auto_start: bool,
}

/// Browser configuration specific to an instance
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct BrowserOverrides {
    /// Override headless mode for this instance
    pub headless: Option<bool>,
    /// Additional browser args specific to this instance
    #[serde(default)]
    pub extra_args: Vec<String>,
}

/// Instance-level runtime configuration (managed via API)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceConfig {
    /// Instance identifier (read-only)
    #[schema(value_type = Option<String>, format = "uuid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<InstanceId>,

    /// Display name for this instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Auto-start browser on server startup
    #[serde(default)]
    pub auto_start: bool,

    /// Browser configuration
    #[serde(default)]
    pub browser: InstanceBrowserConfig,

    /// Webhook configuration for this instance
    #[serde(default)]
    pub webhooks: InstanceWebhookConfig,

    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limits: InstanceRateLimits,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            instance_id: None,
            display_name: None,
            auto_start: false,
            browser: InstanceBrowserConfig::default(),
            webhooks: InstanceWebhookConfig::default(),
            rate_limits: InstanceRateLimits::default(),
        }
    }
}

/// Browser-specific configuration for an instance
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceBrowserConfig {
    /// Run browser in headless mode
    #[serde(default = "default_true")]
    pub headless: bool,

    /// Browser operation timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// Additional browser arguments
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

/// Webhook configuration for an instance
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceWebhookConfig {
    /// Enable webhooks for this instance
    #[serde(default)]
    pub enabled: bool,

    /// Webhook endpoints
    #[serde(default)]
    pub endpoints: Vec<WebhookEndpoint>,

    /// Request timeout in milliseconds
    #[serde(default = "default_webhook_timeout")]
    pub timeout_ms: u64,

    /// Number of retry attempts on failure
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
}

impl Default for InstanceWebhookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoints: vec![],
            timeout_ms: 5000,
            retry_count: 3,
        }
    }
}

fn default_webhook_timeout() -> u64 {
    5000
}

fn default_retry_count() -> u32 {
    3
}

/// Webhook endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WebhookEndpoint {
    /// Webhook URL
    pub url: String,

    /// Secret for HMAC signature verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,

    /// Events to subscribe to (e.g., ["message.received", "message.sent"])
    #[serde(default)]
    pub events: Vec<String>,

    /// Custom headers to include
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
}

/// Rate limiting configuration for an instance
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceRateLimits {
    /// Maximum messages per minute
    #[serde(default = "default_messages_per_minute")]
    pub messages_per_minute: u32,

    /// Maximum API requests per minute  
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,

    /// Cooldown between messages in milliseconds
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

/// Partial update for browser configuration (all fields optional)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct UpdateBrowserConfig {
    /// Run browser in headless mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headless: Option<bool>,

    /// Browser operation timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,

    /// Additional browser arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
}

/// Partial update for webhook configuration (all fields optional)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct UpdateWebhookConfig {
    /// Enable webhooks for this instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Webhook endpoints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<WebhookEndpoint>>,

    /// Request timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,

    /// Number of retry attempts on failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<u32>,
}

/// Partial update for rate limits (all fields optional)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct UpdateRateLimits {
    /// Maximum messages per minute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_per_minute: Option<u32>,

    /// Maximum API requests per minute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_per_minute: Option<u32>,

    /// Cooldown between messages in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_cooldown_ms: Option<u64>,
}

/// Request to update instance configuration (all fields optional)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateInstanceConfigRequest {
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Auto-start on server startup
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_start: Option<bool>,

    /// Browser configuration updates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<UpdateBrowserConfig>,

    /// Webhook configuration updates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<UpdateWebhookConfig>,

    /// Rate limits updates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<UpdateRateLimits>,
}

impl Default for InstanceSetupConfig {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            phone_number: None,
            display_name: None,
            data_dir: PathBuf::new(),
            browser: BrowserOverrides::default(),
            auto_start: false,
        }
    }
}

/// Instance status information (returned by API)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceInfo {
    /// Unique instance identifier (UUID)
    #[schema(value_type = String, format = "uuid")]
    pub id: InstanceId,
    /// Phone number in E.164 format (populated after WhatsApp login)
    pub phone_number: Option<String>,
    /// Display name (populated after WhatsApp login)
    pub display_name: Option<String>,
    /// Current instance status (stopped, starting, running, error)
    pub status: InstanceStatus,
    /// Whether WhatsApp Web is authorized/authenticated
    pub authorized: bool,
    /// Timestamp when instance was created
    pub created_at: DateTime<Utc>,
    /// Timestamp of last WhatsApp activity
    pub last_activity: Option<DateTime<Utc>>,
}

/// Instance runtime status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InstanceStatus {
    /// Instance exists but browser is not running
    Stopped,
    /// Browser is starting up
    Starting,
    /// Browser is running and ready
    Running,
    /// Instance has an error
    Error(String),
}

impl Default for InstanceStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

/// Persistent instance metadata (stored in instance.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceMetadata {
    /// Unique instance identifier (UUID)
    pub id: InstanceId,
    /// Phone number in E.164 format (populated after WhatsApp login)
    pub phone_number: Option<String>,
    /// Display name (populated after WhatsApp login)
    pub display_name: Option<String>,
    /// Timestamp when instance was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when WhatsApp was first linked
    pub first_linked_at: Option<DateTime<Utc>>,
}

impl InstanceMetadata {
    /// Create new metadata for a fresh instance
    pub fn new(id: InstanceId, phone_number: Option<String>, display_name: Option<String>) -> Self {
        Self {
            id,
            phone_number,
            display_name,
            created_at: Utc::now(),
            first_linked_at: None,
        }
    }
}

/// Privacy visibility setting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyVisibility {
    Everyone,
    Contacts,
    Nobody,
}

impl Default for PrivacyVisibility {
    fn default() -> Self {
        Self::Everyone
    }
}

/// Online status visibility (specific options)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OnlineVisibility {
    Everyone,
    SameAsLastSeen,
}

impl Default for OnlineVisibility {
    fn default() -> Self {
        Self::Everyone
    }
}

/// Group add permission
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum GroupAddPermission {
    Everyone,
    Contacts,
    ContactsExcept,
}

impl Default for GroupAddPermission {
    fn default() -> Self {
        Self::Everyone
    }
}

/// WhatsApp privacy settings
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct PrivacySettings {
    /// Who can see last seen
    pub last_seen: PrivacyVisibility,
    /// Who can see online status
    pub online: OnlineVisibility,
    /// Who can see profile photo
    pub profile_photo: PrivacyVisibility,
    /// Who can see about
    pub about: PrivacyVisibility,
    /// Read receipts enabled (blue ticks)
    pub read_receipts: bool,
    /// Who can add to groups
    pub groups: GroupAddPermission,
}

/// WhatsApp profile info
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProfileInfo {
    /// Display name
    pub name: Option<String>,
    /// About/status text
    pub about: Option<String>,
    /// Profile picture URL
    pub picture_url: Option<String>,
}

// === API Request/Response Types ===

/// Request to create a new instance
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateInstanceRequest {
    /// Browser configuration overrides
    #[serde(default)]
    pub browser: Option<BrowserOverrides>,
    /// Auto-start browser on server startup
    #[serde(default)]
    pub auto_start: Option<bool>,
}

/// Response after creating an instance
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateInstanceResponse {
    /// Unique instance identifier (UUID)
    #[schema(value_type = String, format = "uuid")]
    pub id: InstanceId,
    pub status: String,
    pub data_directory: String,
    pub created_at: String,
}

/// Request to update instance display name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInstanceRequest {
    pub display_name: Option<String>,
}

/// Instance list response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceListResponse {
    pub instances: Vec<InstanceInfo>,
    pub total: usize,
}

/// WhatsApp instance status response (for /api/v1/instance/{instance_id}/status)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WhatsAppStatusResponse {
    /// Unique instance identifier (UUID)
    #[schema(value_type = String, format = "uuid")]
    pub instance_id: InstanceId,
    /// Phone number in E.164 format (populated after WhatsApp login)
    pub phone_number: Option<String>,
    /// Instance status (stopped, starting, running, error)
    pub status: String,
    /// Whether WhatsApp Web is authorized
    pub authorized: bool,
    pub last_activity: Option<String>,
}

/// Combined profile update request (all fields optional - update what's provided)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// About/status text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    /// Profile picture (base64 encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
}

/// Combined privacy settings update request (all fields optional)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePrivacyRequest {
    /// Who can see last seen
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<PrivacyVisibility>,
    /// Who can see online status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub online: Option<OnlineVisibility>,
    /// Who can see profile photo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_photo: Option<PrivacyVisibility>,
    /// Who can see about
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<PrivacyVisibility>,
    /// Read receipts enabled (blue ticks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_receipts: Option<bool>,
    /// Who can add to groups
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<GroupAddPermission>,
}

// Legacy individual request types (kept for backwards compatibility)

/// Profile update request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateProfileNameRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateProfileAboutRequest {
    pub about: String,
}

/// Privacy setting update requests
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePrivacyLastSeenRequest {
    pub visibility: PrivacyVisibility,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePrivacyOnlineRequest {
    pub visibility: OnlineVisibility,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePrivacyProfilePhotoRequest {
    pub visibility: PrivacyVisibility,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePrivacyAboutRequest {
    pub visibility: PrivacyVisibility,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePrivacyReadReceiptsRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePrivacyGroupsRequest {
    pub permission: GroupAddPermission,
}

// === API Query/Response Types ===

/// Query parameters for list_instances
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ListInstancesQuery {
    /// Filter by status
    pub status: Option<String>,
    /// Include stopped instances
    #[serde(default = "default_true")]
    pub include_stopped: bool,
}

fn default_true() -> bool {
    true
}

/// Response for delete operation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeleteInstanceResponse {
    pub message: String,
    #[schema(value_type = String, format = "uuid")]
    pub instance_id: InstanceId,
    pub data_deleted: bool,
}

/// Query parameters for delete
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct DeleteInstanceQuery {
    /// Delete all instance data (browser profile, database, etc.)
    #[serde(default)]
    pub delete_data: bool,
}

/// Generic success response for instance operations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceActionResponse {
    pub message: String,
    #[schema(value_type = String, format = "uuid")]
    pub instance_id: InstanceId,
}

/// Request to link via phone number
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct PhoneLinkRequest {
    pub phone_number: String,
}

/// Validate phone number format
/// Accepts: +1234567890 (with +) or 1234567890 (without +)
/// Returns normalized phone number WITHOUT + prefix (digits only)
pub fn validate_phone_number(phone: &str) -> Result<String, String> {
    let phone = phone.trim();

    if phone.is_empty() {
        return Err("Phone number cannot be empty".to_string());
    }

    // Strip leading + for validation and normalization
    let digits_only: String = phone
        .strip_prefix('+')
        .unwrap_or(phone)
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();

    // Must have between 7 and 15 digits (E.164 standard)
    if digits_only.len() < 7 {
        return Err("Phone number too short (minimum 7 digits)".to_string());
    }
    if digits_only.len() > 15 {
        return Err("Phone number too long (maximum 15 digits)".to_string());
    }

    // Return digits only (no + prefix)
    Ok(digits_only)
}

/// Convert phone number to safe directory name
/// E.g., "+1234567890" -> "1234567890" or "1234567890" -> "1234567890"
pub fn phone_to_dir_name(phone: &str) -> String {
    phone.chars().filter(|c| c.is_ascii_digit()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_phone_number() {
        // Valid phone numbers - all return digits only
        assert_eq!(validate_phone_number("+1234567890").unwrap(), "1234567890");
        assert_eq!(validate_phone_number("1234567890").unwrap(), "1234567890");
        assert_eq!(
            validate_phone_number("+44 20 7123 4567").unwrap(),
            "442071234567"
        );
        assert_eq!(
            validate_phone_number("919876543210").unwrap(),
            "919876543210"
        );

        // Invalid phone numbers
        assert!(validate_phone_number("").is_err());
        assert!(validate_phone_number("123456").is_err()); // Too short
        assert!(validate_phone_number("+1234567890123456").is_err()); // Too long
    }

    #[test]
    fn test_phone_to_dir_name() {
        assert_eq!(phone_to_dir_name("+1234567890"), "1234567890");
        assert_eq!(phone_to_dir_name("1234567890"), "1234567890");
    }
}
