//! RBAC service — extracted from `middleware/instance.rs:41..107` + `application/auth/token.rs:42` (#9)
//! Pure, unit-testable without DB. Handlers will call this via port.

use crate::domain::identity::{InstancePermission, UserRole};

/// Permission check input
#[derive(Debug, Clone)]
pub struct PermissionCheck {
    pub is_superadmin: bool,
    pub role: Option<UserRole>,
    pub granted: Option<InstancePermission>,
    pub required: InstancePermission,
}

impl PermissionCheck {
    pub fn allowed(&self) -> bool {
        if self.is_superadmin {
            return true;
        }
        if matches!(self.role, Some(UserRole::Admin)) {
            return true;
        }
        crate::domain::identity::has_permission(self.granted, self.required, false)
    }
}

pub struct RbacService;

impl RbacService {
    pub fn can_access(check: PermissionCheck) -> bool {
        check.allowed()
    }
    pub fn require_admin(is_admin: bool, is_superadmin: bool) -> bool {
        is_admin || is_superadmin
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::identity::InstancePermission::*;
    #[test]
    fn test_rbac() {
        let c = PermissionCheck {
            is_superadmin: false,
            role: Some(UserRole::Admin),
            granted: None,
            required: Viewer,
        };
        assert!(c.allowed());
        let c = PermissionCheck {
            is_superadmin: true,
            role: None,
            granted: None,
            required: Viewer,
        };
        assert!(c.allowed());
        let c = PermissionCheck {
            is_superadmin: false,
            role: Some(UserRole::User),
            granted: Some(Owner),
            required: Viewer,
        };
        assert!(c.allowed());
        let c = PermissionCheck {
            is_superadmin: false,
            role: Some(UserRole::User),
            granted: Some(Viewer),
            required: Operator,
        };
        assert!(!c.allowed());
    }
}
