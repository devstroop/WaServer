//! Account Models
//!
//! Types for multi-account WhatsApp management.
//! Account ID is a UUID, phone_number is a unique field in E.164 format.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utoipa::ToSchema;
use uuid::Uuid;

/// Unique identifier for a WhatsApp account (UUID)
pub type AccountId = Uuid;

/// Account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    /// Unique account identifier (UUID)
    pub id: AccountId,
    /// Phone number in E.164 format (e.g., "1234567890")
    pub phone_number: String,
    /// Optional display name for the account
    pub display_name: Option<String>,
    /// Data directory for this account (browser profile, database, etc.)
    pub data_dir: PathBuf,
    /// Browser configuration overrides
    #[serde(default)]
    pub browser: AccountBrowserConfig,
    /// Whether to auto-start the browser on server startup
    #[serde(default)]
    pub auto_start: bool,
}

/// Browser configuration specific to an account
#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct AccountBrowserConfig {
    /// Override headless mode for this account
    pub headless: Option<bool>,
    /// Additional browser args specific to this account
    #[serde(default)]
    pub extra_args: Vec<String>,
}

/// Instance-level runtime configuration (managed via API)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceConfig {
    /// Instance identifier (read-only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<AccountId>,
    
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

impl Default for AccountConfig {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            phone_number: String::new(),
            display_name: None,
            data_dir: PathBuf::new(),
            browser: AccountBrowserConfig::default(),
            auto_start: false,
        }
    }
}

/// Account status information (returned by API)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountInfo {
    /// Unique account identifier (UUID)
    pub id: AccountId,
    /// Phone number in E.164 format
    pub phone_number: String,
    /// Display name
    pub display_name: Option<String>,
    /// Current account status (stopped, starting, running, error)
    pub status: AccountStatus,
    /// Whether WhatsApp Web is authorized/authenticated
    pub authorized: bool,
    /// Timestamp when account was created
    pub created_at: DateTime<Utc>,
    /// Timestamp of last WhatsApp activity
    pub last_activity: Option<DateTime<Utc>>,
}

/// Account runtime status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    /// Account exists but browser is not running
    Stopped,
    /// Browser is starting up
    Starting,
    /// Browser is running and ready
    Running,
    /// Account has an error
    Error(String),
}

impl Default for AccountStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

/// Persistent account metadata (stored in account.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountMetadata {
    /// Unique account identifier (UUID)
    pub id: AccountId,
    /// Phone number in E.164 format
    pub phone_number: String,
    /// Display name
    pub display_name: Option<String>,
    /// Timestamp when account was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when WhatsApp was first linked
    pub first_linked_at: Option<DateTime<Utc>>,
}

impl AccountMetadata {
    /// Create new metadata for a fresh account
    pub fn new(id: AccountId, phone_number: &str, display_name: Option<String>) -> Self {
        Self {
            id,
            phone_number: phone_number.to_string(),
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

/// Request to create a new account
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAccountRequest {
    /// Phone number in E.164 format (e.g., "+1234567890")
    pub phone_number: String,
    /// Optional display name
    pub display_name: Option<String>,
    /// Browser configuration overrides
    pub browser: Option<AccountBrowserConfig>,
    /// Auto-start browser on server startup
    pub auto_start: Option<bool>,
}

/// Response after creating an account
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAccountResponse {
    /// Unique account identifier (UUID)
    pub id: AccountId,
    /// Phone number in E.164 format
    pub phone_number: String,
    pub status: String,
    pub data_directory: String,
    pub created_at: String,
}

/// Request to update account display name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    pub display_name: Option<String>,
}

/// Account list response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountListResponse {
    pub accounts: Vec<AccountInfo>,
    pub total: usize,
}

/// WhatsApp account status response (for /api/v1/account/{account_id}/status)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WhatsAppStatusResponse {
    /// Unique account identifier (UUID)
    pub account_id: AccountId,
    /// Phone number in E.164 format
    pub phone_number: String,
    /// Account status (stopped, starting, running, error)
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

/// Query parameters for list_accounts
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ListAccountsQuery {
    /// Filter by status
    pub status: Option<String>,
    /// Include stopped accounts
    #[serde(default = "default_true")]
    pub include_stopped: bool,
}

fn default_true() -> bool {
    true
}

/// Response for delete operation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeleteAccountResponse {
    pub message: String,
    pub account_id: AccountId,
    pub data_deleted: bool,
}

/// Query parameters for delete
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct DeleteAccountQuery {
    /// Delete all account data (browser profile, database, etc.)
    #[serde(default)]
    pub delete_data: bool,
}

/// Generic success response for account operations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountActionResponse {
    pub message: String,
    pub account_id: AccountId,
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
    phone
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_phone_number() {
        // Valid phone numbers - all return digits only
        assert_eq!(validate_phone_number("+1234567890").unwrap(), "1234567890");
        assert_eq!(validate_phone_number("1234567890").unwrap(), "1234567890");
        assert_eq!(validate_phone_number("+44 20 7123 4567").unwrap(), "442071234567");
        assert_eq!(validate_phone_number("919876543210").unwrap(), "919876543210");

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
