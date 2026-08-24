//! Permission domain — extracted from `domain/identity/mod.rs:42` + `middleware/instance.rs:57` (#9)
//! Pure, no DB. RBAC hierarchy: Owner > Operator > Viewer.

use super::InstancePermission;

/// Check if `granted` satisfies `required` — mirrors `application/auth/token.rs:42` `can_access`
pub fn has_permission(
    granted: Option<InstancePermission>,
    required: InstancePermission,
    is_admin: bool,
) -> bool {
    if is_admin {
        return true;
    }
    matches!(
        (granted, required),
        (Some(InstancePermission::Owner), _)
            | (
                Some(InstancePermission::Operator),
                InstancePermission::Viewer
            )
            | (
                Some(InstancePermission::Operator),
                InstancePermission::Operator
            )
    )
}

/// Helpers on permission for capability checks (same as `domain/identity/mod.rs:50`)
impl InstancePermission {
    // kept here for cohesion; methods already in mod.rs but re-exported
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::identity::InstancePermission::*;
    #[test]
    fn test_has_permission() {
        assert!(has_permission(Some(Owner), Viewer, false));
        assert!(has_permission(Some(Owner), Operator, false));
        assert!(has_permission(Some(Operator), Viewer, false));
        assert!(!has_permission(Some(Viewer), Operator, false));
        assert!(!has_permission(None, Viewer, false));
        assert!(has_permission(None, Viewer, true));
    }
}
