//! Auth throttling — sliding-window failure counter per key (IP|username) (#44)
//!
//! Login: record a hit only on FAILURE, clear on success. Register: hit on
//! every attempt (anti account-spam). Blocked keys report remaining wait.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct AuthRateLimiter {
    max_failures: u32,
    window: Duration,
    entries: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl AuthRateLimiter {
    pub fn new(max_failures: u32, window_minutes: u64) -> Self {
        Self {
            max_failures,
            window: Duration::from_secs(window_minutes * 60),
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Record an occurrence; `Err(remaining)` when the key is over the limit.
    pub fn hit(&self, key: &str) -> Result<(), Duration> {
        self.hit_at(key, Instant::now())
    }

    /// Check-only: `Some(remaining)` when the key is currently over the limit.
    pub fn is_blocked(&self, key: &str) -> Option<Duration> {
        self.is_blocked_at(key, Instant::now())
    }

    pub fn is_blocked_at(&self, key: &str, now: Instant) -> Option<Duration> {
        let map = self.entries.lock().unwrap();
        let deque = map.get(key)?;
        let live = deque
            .iter()
            .filter(|t| now.duration_since(**t) <= self.window)
            .copied()
            .collect::<Vec<_>>();
        if live.len() >= self.max_failures as usize {
            let oldest = live.first().copied().unwrap_or(now);
            let elapsed = now.duration_since(oldest);
            Some(
                self.window
                    .saturating_sub(elapsed)
                    .max(Duration::from_secs(1)),
            )
        } else {
            None
        }
    }

    fn hit_at(&self, key: &str, now: Instant) -> Result<(), Duration> {
        let mut map = self.entries.lock().unwrap();
        let deque = map.entry(key.to_string()).or_default();
        // prune
        while let Some(front) = deque.front() {
            if now.duration_since(*front) > self.window {
                deque.pop_front();
            } else {
                break;
            }
        }
        if deque.len() >= self.max_failures as usize {
            // retry after the oldest failure ages out of the window
            let oldest = deque.front().copied().unwrap_or(now);
            let elapsed = now.duration_since(oldest);
            return Err(self
                .window
                .saturating_sub(elapsed)
                .max(Duration::from_secs(1)));
        }
        deque.push_back(now);
        Ok(())
    }

    /// Successful auth — forget failures for this key
    pub fn clear(&self, key: &str) {
        self.entries.lock().unwrap().remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocks_after_max_failures() {
        let t = AuthRateLimiter::new(3, 15);
        assert!(t.hit("k").is_ok());
        assert!(t.hit("k").is_ok());
        assert!(t.hit("k").is_ok());
        assert!(t.hit("k").is_err());
    }

    #[test]
    fn test_clear_on_success() {
        let t = AuthRateLimiter::new(2, 15);
        assert!(t.hit("k").is_ok());
        assert!(t.hit("k").is_ok());
        assert!(t.hit("k").is_err());
        t.clear("k");
        assert!(t.hit("k").is_ok());
    }

    #[test]
    fn test_keys_isolated() {
        let t = AuthRateLimiter::new(1, 15);
        assert!(t.hit("a").is_ok());
        assert!(t.hit("a").is_err());
        assert!(t.hit("b").is_ok());
    }

    #[test]
    fn test_window_expiry_via_hit_at() {
        let t = AuthRateLimiter::new(1, 15);
        let t0 = Instant::now();
        assert!(t.hit_at("k", t0).is_ok());
        assert!(t.hit_at("k", t0 + Duration::from_secs(60)).is_err());
        // after window passes, old entry pruned → allowed again
        assert!(t.hit_at("k", t0 + Duration::from_secs(901)).is_ok());
    }
}
