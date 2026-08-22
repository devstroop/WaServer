//! Instance Config validation — typed errors per #6
//! Extracted from `models/instance.rs:446` + `handlers/api/instances.rs:289,325`.
//! Config updates return `ConfigError` (→ 400) instead of generic 500.

use crate::application::instance::config::apply_config_update;
use crate::domain::instance::{InstanceConfig, UpdateInstanceConfigRequest};

/// Typed config errors — map to HTTP 400 with `{error, message}` envelope
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("idle_timeout must be between 30 and 86400 seconds, got {0}")]
    InvalidIdleTimeout(u64),
    #[error("browser timeout_ms must be between 1000 and 300000, got {0}")]
    InvalidBrowserTimeout(u64),
    #[error("messages_per_minute must be between 1 and 1000, got {0}")]
    InvalidMessagesPerMinute(u32),
    #[error("requests_per_minute must be between 1 and 5000, got {0}")]
    InvalidRequestsPerMinute(u32),
    #[error("instance_name cannot be empty")]
    EmptyName,
}

impl ConfigError {
    /// Stable error code for API envelope
    pub fn code(&self) -> &'static str {
        match self {
            ConfigError::InvalidIdleTimeout(_) => "invalid_idle_timeout",
            ConfigError::InvalidBrowserTimeout(_) => "invalid_browser_timeout",
            ConfigError::InvalidMessagesPerMinute(_) => "invalid_messages_per_minute",
            ConfigError::InvalidRequestsPerMinute(_) => "invalid_requests_per_minute",
            ConfigError::EmptyName => "empty_instance_name",
        }
    }
}

/// Validate an `InstanceConfig` — pure, no DB
pub fn validate_config(config: &InstanceConfig) -> Result<(), ConfigError> {
    if !(30..=86400).contains(&config.idle_timeout) {
        return Err(ConfigError::InvalidIdleTimeout(config.idle_timeout));
    }
    if !(1000..=300000).contains(&config.browser.timeout_ms) {
        return Err(ConfigError::InvalidBrowserTimeout(
            config.browser.timeout_ms,
        ));
    }
    if !(1..=1000).contains(&config.rate_limits.messages_per_minute) {
        return Err(ConfigError::InvalidMessagesPerMinute(
            config.rate_limits.messages_per_minute,
        ));
    }
    if !(1..=5000).contains(&config.rate_limits.requests_per_minute) {
        return Err(ConfigError::InvalidRequestsPerMinute(
            config.rate_limits.requests_per_minute,
        ));
    }
    Ok(())
}

/// Validate then apply update; returns new config or typed error.
/// `restart_required` is true when browser-level fields changed (`headless`, `timeout_ms`, `extra_args`).
pub fn validated_apply_config_update(
    current: &InstanceConfig,
    req: UpdateInstanceConfigRequest,
) -> Result<(InstanceConfig, bool), ConfigError> {
    if let Some(name) = &req.instance_name {
        if name.trim().is_empty() {
            return Err(ConfigError::EmptyName);
        }
    }
    let next = apply_config_update(current, req);
    validate_config(&next)?;
    let restart_required = next.browser.headless != current.browser.headless
        || next.browser.timeout_ms != current.browser.timeout_ms
        || next.browser.extra_args != current.browser.extra_args;
    Ok((next, restart_required))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::instance::{UpdateBrowserConfig, UpdateRateLimits};

    #[test]
    fn test_validate_default_ok() {
        assert!(validate_config(&InstanceConfig::default()).is_ok());
    }

    #[test]
    fn test_invalid_idle_timeout() {
        let mut cfg = InstanceConfig::default();
        cfg.idle_timeout = 0;
        let err = validate_config(&cfg).unwrap_err();
        assert_eq!(err.code(), "invalid_idle_timeout");
        cfg.idle_timeout = 100_000;
        assert!(validate_config(&cfg).is_err());
    }

    #[test]
    fn test_validated_apply_ok() {
        let cur = InstanceConfig::default();
        let req = UpdateInstanceConfigRequest {
            instance_name: Some("renamed".into()),
            idle_timeout: Some(600),
            browser: None,
            rate_limits: Some(UpdateRateLimits {
                messages_per_minute: Some(120),
                requests_per_minute: None,
                message_cooldown_ms: None,
            }),
        };
        let (next, restart) = validated_apply_config_update(&cur, req).unwrap();
        assert_eq!(next.instance_name.as_deref(), Some("renamed"));
        assert_eq!(next.idle_timeout, 600);
        assert_eq!(next.rate_limits.messages_per_minute, 120);
        assert!(!restart); // no browser fields changed
    }

    #[test]
    fn test_validated_apply_restart_required() {
        let cur = InstanceConfig::default();
        let req = UpdateInstanceConfigRequest {
            instance_name: None,
            idle_timeout: None,
            browser: Some(UpdateBrowserConfig {
                headless: Some(false),
                timeout_ms: None,
                extra_args: None,
            }),
            rate_limits: None,
        };
        let (_, restart) = validated_apply_config_update(&cur, req).unwrap();
        assert!(restart);
    }

    #[test]
    fn test_empty_name_rejected() {
        let cur = InstanceConfig::default();
        let req = UpdateInstanceConfigRequest {
            instance_name: Some("   ".into()),
            idle_timeout: None,
            browser: None,
            rate_limits: None,
        };
        let err = validated_apply_config_update(&cur, req).unwrap_err();
        assert_eq!(err.code(), "empty_instance_name");
    }
}
