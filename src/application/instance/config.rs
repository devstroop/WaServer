//! Instance Config application logic — validates and applies `UpdateInstanceConfigRequest`
//! Extracted from `services/whatsapp/instance.rs:360` `update_config`

use crate::domain::instance::{InstanceConfig, UpdateInstanceConfigRequest};

pub fn apply_config_update(
    current: &InstanceConfig,
    req: UpdateInstanceConfigRequest,
) -> InstanceConfig {
    let mut next = current.clone();
    if let Some(name) = req.instance_name {
        next.instance_name = Some(name);
    }
    if let Some(idle) = req.idle_timeout {
        next.idle_timeout = idle;
    }
    if let Some(browser) = req.browser {
        if let Some(headless) = browser.headless {
            next.browser.headless = headless;
        }
        if let Some(timeout) = browser.timeout_ms {
            next.browser.timeout_ms = timeout;
        }
        if let Some(args) = browser.extra_args {
            next.browser.extra_args = args;
        }
    }
    if let Some(limits) = req.rate_limits {
        if let Some(mpm) = limits.messages_per_minute {
            next.rate_limits.messages_per_minute = mpm;
        }
        if let Some(rpm) = limits.requests_per_minute {
            next.rate_limits.requests_per_minute = rpm;
        }
        if let Some(cooldown) = limits.message_cooldown_ms {
            next.rate_limits.message_cooldown_ms = cooldown;
        }
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_apply_config_update() {
        let cur = InstanceConfig::default();
        let req = UpdateInstanceConfigRequest {
            instance_name: Some("new-name".into()),
            idle_timeout: Some(999),
            browser: None,
            rate_limits: None,
        };
        let next = apply_config_update(&cur, req);
        assert_eq!(next.instance_name, Some("new-name".into()));
        assert_eq!(next.idle_timeout, 999);
    }
}
