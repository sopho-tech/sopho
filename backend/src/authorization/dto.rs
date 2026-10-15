use crate::authorization::constants::{ActorKind, Permission, ResourceType, Visibility};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenantId {
    tenant_id: Uuid,
}

impl TenantId {
    pub(super) fn from_verified_membership(tenant_id: Uuid) -> Self {
        Self { tenant_id }
    }

    pub(crate) fn from_system_unchecked(tenant_id: Uuid) -> Self {
        Self { tenant_id }
    }

    pub fn get_uuid(&self) -> Uuid {
        self.tenant_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId {
    user_id: Uuid,
}

impl UserId {
    pub(super) fn from_validated_session(user_id: Uuid) -> Self {
        Self { user_id }
    }

    pub(crate) fn from_system_unchecked(user_id: Uuid) -> Self {
        Self { user_id }
    }

    pub fn get_uuid(&self) -> Uuid {
        self.user_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Actor {
    kind: ActorKind,
    user_id: Option<Uuid>,
}

impl Actor {
    pub fn human(user_id: UserId) -> Actor {
        Actor {
            kind: ActorKind::Human,
            user_id: Some(user_id.get_uuid()),
        }
    }

    pub fn ai_for(user_id: UserId) -> Actor {
        Actor {
            kind: ActorKind::Ai,
            user_id: Some(user_id.get_uuid()),
        }
    }

    pub fn system() -> Actor {
        Actor {
            kind: ActorKind::System,
            user_id: None,
        }
    }

    pub fn user_id(&self) -> Option<Uuid> {
        self.user_id
    }

    pub fn kind_name(&self) -> String {
        self.kind.as_str().to_string()
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    user_id: UserId,
}

impl AuthenticatedUser {
    pub(crate) fn from_validated_session(user_id: Uuid) -> Self {
        Self {
            user_id: UserId::from_validated_session(user_id),
        }
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }
}

#[derive(Debug, Clone, Default)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    pub(crate) fn from_permissions(permissions: HashSet<Permission>) -> Self {
        Self { permissions }
    }

    pub fn contains(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    pub fn is_subset_of(&self, other: &PermissionSet) -> bool {
        self.permissions.is_subset(&other.permissions)
    }

    pub fn as_strings(&self) -> Vec<String> {
        let mut values: Vec<String> = self
            .permissions
            .iter()
            .map(|permission| permission.as_str().to_string())
            .collect();
        values.sort();
        values
    }
}

#[derive(Debug, Clone)]
pub struct TenantContext {
    user_id: UserId,
    tenant_id: TenantId,
    permissions: PermissionSet,
}

impl TenantContext {
    pub(super) fn new(user_id: UserId, tenant_id: TenantId, permissions: PermissionSet) -> Self {
        Self {
            user_id,
            tenant_id,
            permissions,
        }
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    pub fn permissions(&self) -> &PermissionSet {
        &self.permissions
    }

    pub fn actor(&self) -> Actor {
        Actor::human(self.user_id)
    }

    pub fn ai_actor(&self) -> Actor {
        Actor::ai_for(self.user_id)
    }
}

#[derive(Debug, Clone)]
pub struct AuthorizationSubject {
    tenant_id: TenantId,
    resource_type: ResourceType,
    resource_id: Uuid,
    owner_user_id: Uuid,
    visibility: Visibility,
}

impl AuthorizationSubject {
    pub fn new(
        tenant_id: TenantId,
        resource_type: ResourceType,
        resource_id: Uuid,
        owner_user_id: Uuid,
        visibility: Visibility,
    ) -> Self {
        Self {
            tenant_id,
            resource_type,
            resource_id,
            owner_user_id,
            visibility,
        }
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }

    pub fn resource_id(&self) -> Uuid {
        self.resource_id
    }

    pub fn owner_user_id(&self) -> Uuid {
        self.owner_user_id
    }

    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    pub fn ownership(&self) -> ResourceOwnership {
        ResourceOwnership {
            owner_user_id: self.owner_user_id,
            visibility: self.visibility,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResourceOwnership {
    pub owner_user_id: Uuid,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct ResourceCapabilities {
    pub can_edit: bool,
    pub can_delete: bool,
    pub can_share: bool,
}
