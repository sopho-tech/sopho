use crate::authorization::constants::{
    AccessLevel, Action, MembershipStatus, Permission, ResourceType, SystemRole, Visibility,
};
use crate::authorization::dto::{
    AuthorizationSubject, PermissionSet, ResourceCapabilities, TenantContext, TenantId, UserId,
};
use crate::authorization::errors::{AuthorizationError, TenantContextError};
use crate::authorization::repository;
use crate::common::AppState;
use sea_orm::{ColumnTrait, Condition};
use std::collections::HashSet;
use uuid::Uuid;

pub async fn seed_system_roles_for_tenant(
    app_state: &AppState,
    tenant_id: &TenantId,
) -> anyhow::Result<()> {
    for system_role in SystemRole::all() {
        let role_name = system_role.as_str();
        let role = match repository::find_role_by_name(
            &app_state.database_connection,
            tenant_id,
            role_name,
        )
        .await?
        {
            Some(existing_role) => existing_role,
            None => {
                repository::insert_role(&app_state.database_connection, tenant_id, role_name, true)
                    .await?
            }
        };

        let existing_permissions: HashSet<String> =
            repository::find_permissions_for_role(&app_state.database_connection, role.id)
                .await?
                .into_iter()
                .map(|role_permission| role_permission.permission)
                .collect();

        for permission in system_role.permissions() {
            if existing_permissions.contains(permission.as_str()) {
                continue;
            }
            repository::insert_role_permission(
                &app_state.database_connection,
                role.id,
                permission.as_str(),
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn resolve_tenant_context(
    app_state: &AppState,
    user_id: UserId,
    tenant_id: Uuid,
) -> Result<TenantContext, TenantContextError> {
    let membership = repository::find_membership(
        &app_state.database_connection,
        tenant_id,
        user_id.get_uuid(),
    )
    .await
    .map_err(TenantContextError::Database)?
    .ok_or(TenantContextError::NotAMember)?;

    if membership.status != MembershipStatus::Active.as_str() {
        return Err(TenantContextError::MembershipSuspended);
    }

    let permissions = resolve_permissions_for_role(app_state, membership.role_id).await?;

    Ok(TenantContext::new(
        user_id,
        TenantId::from_verified_membership(tenant_id),
        permissions,
    ))
}

pub async fn resolve_permissions_for_role(
    app_state: &AppState,
    role_id: Uuid,
) -> Result<PermissionSet, TenantContextError> {
    let role_permissions =
        repository::find_permissions_for_role(&app_state.database_connection, role_id)
            .await
            .map_err(TenantContextError::Database)?;

    let mut permissions = HashSet::with_capacity(role_permissions.len());
    for role_permission in role_permissions {
        let permission = Permission::from_str(&role_permission.permission)
            .map_err(|_| TenantContextError::UnknownPermission(role_permission.permission))?;
        permissions.insert(permission);
    }
    Ok(PermissionSet::from_permissions(permissions))
}

pub async fn create_custom_role(
    app_state: &AppState,
    tenant_id: &TenantId,
    name: &str,
    permissions: &[Permission],
) -> Result<crate::entity::role::Model, sea_orm::DbErr> {
    let role =
        repository::insert_role(&app_state.database_connection, tenant_id, name, false).await?;
    for permission in permissions {
        repository::insert_role_permission(
            &app_state.database_connection,
            role.id,
            permission.as_str(),
        )
        .await?;
    }
    Ok(role)
}

pub async fn find_role_by_name(
    app_state: &AppState,
    tenant_id: &TenantId,
    name: &str,
) -> Result<crate::entity::role::Model, sea_orm::DbErr> {
    repository::find_role_by_name(&app_state.database_connection, tenant_id, name)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(format!("Role not found: {}", name)))
}

pub async fn find_membership(
    app_state: &AppState,
    tenant_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> Result<Option<crate::entity::tenant_membership::Model>, sea_orm::DbErr> {
    repository::find_membership(&app_state.database_connection, tenant_id, user_id).await
}

pub fn require_capability(
    tenant_context: &TenantContext,
    permission: Permission,
) -> Result<(), AuthorizationError> {
    if tenant_context.permissions().contains(permission) {
        return Ok(());
    }
    Err(AuthorizationError::MissingPermission(
        permission.as_str().to_string(),
    ))
}

pub async fn require_access(
    app_state: &AppState,
    tenant_context: &TenantContext,
    subject: &AuthorizationSubject,
    action: Action,
) -> Result<(), AuthorizationError> {
    if subject.tenant_id().get_uuid() != tenant_context.tenant_id().get_uuid() {
        return Err(AuthorizationError::WrongTenant);
    }

    let decision = decide_without_grant(
        tenant_context.permissions(),
        subject.resource_type(),
        action,
        subject.owner_user_id() == tenant_context.user_id().get_uuid(),
        subject.visibility(),
    );

    match decision {
        AccessDecision::Allowed => Ok(()),
        AccessDecision::MissingCapability => Err(AuthorizationError::MissingPermission(
            subject
                .resource_type()
                .permission_for(action)
                .as_str()
                .to_string(),
        )),
        AccessDecision::NotReachable => Err(AuthorizationError::NotReachable),
        AccessDecision::RequiresGrant => {
            let granted = find_grant_for(app_state, tenant_context, subject)
                .await?
                .and_then(|found| AccessLevel::from_str(&found.access_level).ok());
            if grant_satisfies(action, granted) {
                Ok(())
            } else {
                Err(AuthorizationError::NotReachable)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessDecision {
    Allowed,
    RequiresGrant,
    MissingCapability,
    NotReachable,
}

/// Both gates, minus the grant lookup, so the decision model can be exercised
/// without a database. `RequiresGrant` means the caller must consult
/// `resource_grant` and feed the result to `grant_satisfies`.
pub fn decide_without_grant(
    permissions: &PermissionSet,
    resource_type: ResourceType,
    action: Action,
    is_owner: bool,
    visibility: Visibility,
) -> AccessDecision {
    if !permissions.contains(resource_type.permission_for(action)) {
        return AccessDecision::MissingCapability;
    }
    if is_owner {
        return AccessDecision::Allowed;
    }
    match action {
        Action::Read => {
            if visibility == Visibility::Tenant
                || permissions.contains(resource_type.read_private_permission())
            {
                AccessDecision::Allowed
            } else {
                AccessDecision::RequiresGrant
            }
        }
        Action::Write | Action::Delete => {
            if permissions.contains(resource_type.write_all_permission()) {
                AccessDecision::Allowed
            } else {
                AccessDecision::RequiresGrant
            }
        }
        Action::Share => {
            if permissions.contains(resource_type.write_all_permission()) {
                AccessDecision::Allowed
            } else {
                AccessDecision::NotReachable
            }
        }
    }
}

pub fn grant_satisfies(action: Action, granted: Option<AccessLevel>) -> bool {
    match action {
        Action::Read => granted.is_some(),
        Action::Write | Action::Delete => granted == Some(AccessLevel::Editor),
        Action::Share => false,
    }
}

async fn find_grant_for(
    app_state: &AppState,
    tenant_context: &TenantContext,
    subject: &AuthorizationSubject,
) -> Result<Option<crate::entity::resource_grant::Model>, AuthorizationError> {
    repository::find_grant(
        &app_state.database_connection,
        subject.tenant_id(),
        subject.resource_type().as_str(),
        subject.resource_id(),
        tenant_context.user_id().get_uuid(),
    )
    .await
    .map_err(AuthorizationError::Database)
}

pub fn reachable_resource_condition<C: ColumnTrait>(
    tenant_context: &TenantContext,
    resource_type: ResourceType,
    id_column: C,
    owner_user_id_column: C,
    visibility_column: C,
) -> Condition {
    if tenant_context
        .permissions()
        .contains(resource_type.read_private_permission())
    {
        return Condition::all();
    }

    Condition::any()
        .add(owner_user_id_column.eq(tenant_context.user_id().get_uuid()))
        .add(visibility_column.eq(Visibility::Tenant.as_str()))
        .add(
            id_column.in_subquery(repository::grant_resource_ids_subquery(
                tenant_context.tenant_id(),
                tenant_context.user_id().get_uuid(),
                resource_type,
            )),
        )
}

pub fn authorization_error_response(
    error: AuthorizationError,
    not_found_message: &str,
) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    match error {
        AuthorizationError::MissingPermission(permission) => (
            axum::http::StatusCode::FORBIDDEN,
            axum::Json(serde_json::json!({
                "error": format!("Permission denied: {}", permission)
            })),
        ),
        AuthorizationError::WrongTenant | AuthorizationError::NotReachable => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": not_found_message })),
        ),
        AuthorizationError::Database(database_error) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": database_error.to_string() })),
        ),
    }
}

pub fn subject_from_parts(
    tenant_id: &TenantId,
    resource_type: crate::authorization::constants::ResourceType,
    resource_id: uuid::Uuid,
    owner_user_id: uuid::Uuid,
    visibility: &str,
) -> Result<AuthorizationSubject, String> {
    Ok(AuthorizationSubject::new(
        tenant_id.clone(),
        resource_type,
        resource_id,
        owner_user_id,
        Visibility::from_str(visibility)?,
    ))
}

pub async fn resolve_capabilities(
    app_state: &AppState,
    tenant_context: &TenantContext,
    subject: &AuthorizationSubject,
) -> Result<ResourceCapabilities, AuthorizationError> {
    Ok(ResourceCapabilities {
        can_edit: is_allowed(app_state, tenant_context, subject, Action::Write).await?,
        can_delete: is_allowed(app_state, tenant_context, subject, Action::Delete).await?,
        can_share: is_allowed(app_state, tenant_context, subject, Action::Share).await?,
    })
}

async fn is_allowed(
    app_state: &AppState,
    tenant_context: &TenantContext,
    subject: &AuthorizationSubject,
    action: Action,
) -> Result<bool, AuthorizationError> {
    match require_access(app_state, tenant_context, subject, action).await {
        Ok(()) => Ok(true),
        Err(AuthorizationError::Database(database_error)) => {
            Err(AuthorizationError::Database(database_error))
        }
        Err(_) => Ok(false),
    }
}
