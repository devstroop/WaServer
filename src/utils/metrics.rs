// Metrics and Observability
//
// Data structures and utilities for tracking service metrics and performance

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Debug, Clone, Default)]
pub struct ServiceMetrics {
    pub total_messages_sent: Arc<AtomicU64>,
    pub total_auth_attempts: Arc<AtomicU64>,
    pub error_count: Arc<AtomicU64>,
    pub last_activity: Arc<AtomicU64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub total_messages_sent: u64,
    pub total_auth_attempts: u64,
    pub error_count: u64,
    pub last_activity: Option<u64>,
}

impl ServiceMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_messages_sent(&self) {
        self.total_messages_sent.fetch_add(1, Ordering::Relaxed);
        self.update_last_activity();
    }

    pub fn increment_auth_attempts(&self) {
        self.total_auth_attempts.fetch_add(1, Ordering::Relaxed);
        self.update_last_activity();
    }

    pub fn increment_error_count(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
        self.update_last_activity();
    }

    pub fn update_last_activity(&self) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_activity.store(now, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let last_activity = self.last_activity.load(Ordering::Relaxed);
        MetricsSnapshot {
            total_messages_sent: self.total_messages_sent.load(Ordering::Relaxed),
            total_auth_attempts: self.total_auth_attempts.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            last_activity: if last_activity == 0 {
                None
            } else {
                Some(last_activity)
            },
        }
    }

    pub fn reset(&self) {
        self.total_messages_sent.store(0, Ordering::Relaxed);
        self.total_auth_attempts.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.last_activity.store(0, Ordering::Relaxed);
    }
}
