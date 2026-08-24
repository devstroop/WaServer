//! In-memory sliding window rate limiter per InstanceId.
//! Pure `RateLimitPort` adapter — no DB, no browser.
//! Optional `RateLimitConfig` resolves per-instance limits (default 60/min).

use crate::application::instance::InstanceRegistry;
use crate::application::messaging::ports::{RateLimitConfig, RateLimitPort};
use crate::domain::instance::InstanceId;
use crate::domain::messaging::MessageStatus;
use crate::domain::shared::error::{DomainError, DomainResult};
use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct InMemoryRateLimiter {
    default_max_per_minute: u32,
    cooldown: Duration,
    config: Option<Arc<dyn RateLimitConfig>>,
    // InstanceId -> deque of send instants (only last 60s kept)
    windows: Arc<RwLock<HashMap<InstanceId, VecDeque<Instant>>>>,
    // InstanceId -> last send instant (for cooldown)
    last: Arc<RwLock<HashMap<InstanceId, Instant>>>,
}

impl InMemoryRateLimiter {
    pub fn new(max_per_minute: u32, cooldown_ms: u64) -> Self {
        Self {
            default_max_per_minute: max_per_minute,
            cooldown: Duration::from_millis(cooldown_ms),
            config: None,
            windows: Arc::new(RwLock::new(HashMap::new())),
            last: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Limit resolved per-send via `RateLimitConfig` (falls back to default)
    pub fn configured(
        config: Arc<dyn RateLimitConfig>,
        default_max_per_minute: u32,
        cooldown_ms: u64,
    ) -> Self {
        Self {
            default_max_per_minute,
            cooldown: Duration::from_millis(cooldown_ms),
            config: Some(config),
            windows: Arc::new(RwLock::new(HashMap::new())),
            last: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn default_for_instance() -> Self {
        Self::new(60, 1000)
    }

    async fn check_with_limit(&self, instance: InstanceId, limit: u32) -> DomainResult<()> {
        let now = Instant::now();
        // cooldown check
        {
            let last = self.last.read().await;
            if let Some(prev) = last.get(&instance) {
                let elapsed = now.duration_since(*prev);
                if elapsed < self.cooldown {
                    let retry = (self.cooldown - elapsed).as_secs() as u32 + 1;
                    return Err(DomainError::RateLimited {
                        operation: "send".to_string(),
                        retry_after_seconds: retry,
                    });
                }
            }
        }
        // sliding window check
        {
            let mut windows = self.windows.write().await;
            let deque = windows.entry(instance).or_default();
            // prune > 60s
            while let Some(front) = deque.front() {
                if now.duration_since(*front) > Duration::from_secs(60) {
                    deque.pop_front();
                } else {
                    break;
                }
            }
            if deque.len() as u32 >= limit {
                let retry = match deque.front() {
                    Some(front) => 60 - now.duration_since(*front).as_secs() as u32,
                    None => 60,
                };
                return Err(DomainError::RateLimited {
                    operation: "send".to_string(),
                    retry_after_seconds: retry.max(1),
                });
            }
            deque.push_back(now);
        }
        // record last
        {
            let mut last = self.last.write().await;
            last.insert(instance, now);
        }
        Ok(())
    }
}

/// Resolves limits from instance registry config (`rate_limits.messages_per_minute`)
pub struct RegistryRateLimits(pub Arc<InstanceRegistry>);

#[async_trait]
impl RateLimitConfig for RegistryRateLimits {
    async fn max_per_minute(&self, instance: InstanceId) -> u32 {
        self.0
            .get_config(instance)
            .await
            .map(|c| c.rate_limits.messages_per_minute)
            .unwrap_or(60)
    }
}

#[async_trait]
impl RateLimitPort for InMemoryRateLimiter {
    async fn check_and_record(&self, instance: InstanceId) -> DomainResult<()> {
        let limit = match &self.config {
            Some(c) => c.max_per_minute(instance).await,
            None => self.default_max_per_minute,
        };
        self.check_with_limit(instance, limit).await
    }

    async fn get_status(&self, _instance: InstanceId) -> MessageStatus {
        // For now, always return Pending (could be extended to check queue depth)
        MessageStatus::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    struct FixedLimit(u32);
    #[async_trait]
    impl RateLimitConfig for FixedLimit {
        async fn max_per_minute(&self, _id: InstanceId) -> u32 {
            self.0
        }
    }

    #[tokio::test]
    async fn test_check_and_record_ok() {
        let limiter = InMemoryRateLimiter::new(2, 0);
        let id = InstanceId::new_v4();
        assert!(limiter.check_and_record(id).await.is_ok());
        assert!(limiter.check_and_record(id).await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limited() {
        let limiter = InMemoryRateLimiter::new(1, 0);
        let id = InstanceId::new_v4();
        assert!(limiter.check_and_record(id).await.is_ok());
        let err = limiter.check_and_record(id).await.unwrap_err();
        assert!(matches!(err, DomainError::RateLimited { .. }));
    }

    #[tokio::test]
    async fn test_cooldown() {
        let limiter = InMemoryRateLimiter::new(10, 100);
        let id = InstanceId::new_v4();
        assert!(limiter.check_and_record(id).await.is_ok());
        assert!(limiter.check_and_record(id).await.is_err());
        sleep(Duration::from_millis(110)).await;
        assert!(limiter.check_and_record(id).await.is_ok());
    }

    #[tokio::test]
    async fn test_isolation_per_instance() {
        let limiter = InMemoryRateLimiter::new(1, 0);
        let a = InstanceId::new_v4();
        let b = InstanceId::new_v4();
        assert!(limiter.check_and_record(a).await.is_ok());
        assert!(limiter.check_and_record(b).await.is_ok());
        assert!(limiter.check_and_record(a).await.is_err());
        assert!(limiter.check_and_record(b).await.is_err());
    }

    #[tokio::test]
    async fn test_config_resolved_limit() {
        let limiter = InMemoryRateLimiter::configured(Arc::new(FixedLimit(2)), 60, 0);
        let id = InstanceId::new_v4();
        assert!(limiter.check_and_record(id).await.is_ok());
        assert!(limiter.check_and_record(id).await.is_ok());
        assert!(limiter.check_and_record(id).await.is_err());
    }
}
