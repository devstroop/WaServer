//! In-memory sliding window rate limiter per InstanceId.
//! Pure `RateLimitPort` adapter — no DB, no browser.

use crate::application::messaging::ports::RateLimitPort;
use crate::domain::instance::InstanceId;
use crate::domain::messaging::MessageStatus;
use crate::domain::shared::error::{DomainError, DomainResult};
use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct InMemoryRateLimiter {
    max_per_minute: u32,
    cooldown: Duration,
    // InstanceId -> deque of send instants (only last 60s kept)
    windows: Arc<RwLock<HashMap<InstanceId, VecDeque<Instant>>>>,
    // InstanceId -> last send instant (for cooldown)
    last: Arc<RwLock<HashMap<InstanceId, Instant>>>,
}

impl InMemoryRateLimiter {
    pub fn new(max_per_minute: u32, cooldown_ms: u64) -> Self {
        Self {
            max_per_minute,
            cooldown: Duration::from_millis(cooldown_ms),
            windows: Arc::new(RwLock::new(HashMap::new())),
            last: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn default_for_instance() -> Self {
        Self::new(60, 1000)
    }
}

#[async_trait]
impl RateLimitPort for InMemoryRateLimiter {
    async fn check_and_record(&self, instance: InstanceId) -> DomainResult<()> {
        let now = Instant::now();
        // cooldown check
        {
            let last = self.last.read().await;
            if let Some(prev) = last.get(&instance) {
                if now.duration_since(*prev) < self.cooldown {
                    let retry = (self.cooldown - now.duration_since(*prev)).as_secs() as u32 + 1;
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
            if deque.len() as u32 >= self.max_per_minute {
                return Err(DomainError::RateLimited {
                    operation: "send".to_string(),
                    retry_after_seconds: 60 - now.duration_since(*deque.front().unwrap()).as_secs() as u32,
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

    async fn get_status(&self, _instance: InstanceId) -> MessageStatus {
        // For now, always return Pending (could be extended to check queue depth)
        MessageStatus::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

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
}
