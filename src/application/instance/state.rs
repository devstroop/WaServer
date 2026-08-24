//! Instance State Machine — pure application logic
//! Extracted from `services/whatsapp/instance.rs:234` `InstanceStatus`
//!
//! No browser/DB deps — only `crate::domain::instance::InstanceStatus` (pure).

use crate::domain::instance::InstanceStatus;

/// View projection for `InstanceStatus` (API stable)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceStatusView {
    Sleeping,
    WarmingUp,
    Active,
    Error(String),
}

impl From<InstanceStatus> for InstanceStatusView {
    fn from(s: InstanceStatus) -> Self {
        match s {
            InstanceStatus::Sleeping => Self::Sleeping,
            InstanceStatus::WarmingUp => Self::WarmingUp,
            InstanceStatus::Active => Self::Active,
            InstanceStatus::Error(e) => Self::Error(e),
        }
    }
}

/// Pure state machine — validates transitions
pub struct InstanceState {
    current: InstanceStatus,
}

impl InstanceState {
    pub fn new(initial: InstanceStatus) -> Self {
        Self { current: initial }
    }

    pub fn current(&self) -> &InstanceStatus {
        &self.current
    }

    /// Validate `Sleeping → WarmingUp → Active → Sleeping` + Error
    pub fn can_transition(&self, next: &InstanceStatus) -> bool {
        use InstanceStatus::*;
        matches!(
            (&self.current, next),
            (Sleeping, WarmingUp)
                | (WarmingUp, Active)
                | (WarmingUp, Error(_))
                | (Active, Sleeping)
                | (Active, Error(_))
                | (Error(_), Sleeping)
                | (Error(_), WarmingUp)
                | (Sleeping, Error(_))
        )
    }

    pub fn transition(&mut self, next: InstanceStatus) -> Result<(), String> {
        if self.can_transition(&next) {
            self.current = next;
            Ok(())
        } else {
            Err(format!(
                "invalid transition {:?} -> {:?}",
                self.current, next
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_state_transitions() {
        let mut s = InstanceState::new(InstanceStatus::Sleeping);
        assert!(s.transition(InstanceStatus::WarmingUp).is_ok());
        assert!(s.transition(InstanceStatus::Active).is_ok());
        assert!(s.transition(InstanceStatus::Sleeping).is_ok());
        // invalid: Sleeping -> Active directly
        let mut s2 = InstanceState::new(InstanceStatus::Sleeping);
        assert!(s2.transition(InstanceStatus::Active).is_err());
    }
}
