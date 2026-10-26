use crate::authentication::service::UserStatus;
use crate::authorization::constants::{Permission, SystemRole};
use crate::authorization::dto::{Actor, PermissionSet, TenantContext, TenantId, UserId};
use crate::authorization::service as authorization_service;
use crate::common::AppState;
use crate::entity::{role, tenant_membership};
use crate::tenant::dto::TenantMemberDto;
use crate::tenant::membership::dto::{MemberCapabilities, MemberDto, RoleDto};
use crate::tenant::membership::error::MembershipError;
use crate::tenant::membership::repository;
use crate::tenant::membership::repository::TenantMemberRow;
use sea_orm::TransactionTrait;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::str::FromStr;
use uuid::Uuid;

pub async fn add_member(
    app_state: &AppState,
    tenant_id: &TenantId,
    user_id: UserId,
    role_id: Uuid,
    actor: &Actor,
) -> Result<(), MembershipError> {
    repository::insert_membership(
        &app_state.database_connection,
        tenant_id,
        user_id.get_uuid(),
        role_id,
        actor,
    )
    .await?;
    Ok(())
}

pub async fn add_member_transaction(
    txn: &sea_orm::DatabaseTransaction,
    tenant_id: &TenantId,
    user_id: UserId,
    role_id: Uuid,
    actor: &Actor,
) -> Result<(), MembershipError> {
    repository::insert_membership(txn, tenant_id, user_id.get_uuid(), role_id, actor).await?;
    Ok(())
}

pub async fn add_member_with_role_name(
    app_state: &AppState,
    tenant_id: &TenantId,
    user_id: UserId,
    role_name: &str,
    actor: &Actor,
) -> Result<(), MembershipError> {
    let role = find_role(app_state, tenant_id, role_name).await?;
    add_member(app_state, tenant_id, user_id, role.id, actor).await
}

pub async fn is_active_member(
    app_state: &AppState,
    tenant_id: &TenantId,
    user_id: Uuid,
) -> Result<bool, MembershipError> {
    Ok(
        repository::find_active_membership(&app_state.database_connection, tenant_id, user_id)
            .await?
            .is_some(),
    )
}

pub async fn require_grantable_role(
    app_state: &AppState,
    tenant_context: &TenantContext,
    role_name: &str,
) -> Result<role::Model, MembershipError> {
    let role = find_role(app_state, tenant_context.tenant_id(), role_name).await?;
    if !role_permissions(app_state, role.id)
        .await?
        .is_subset_of(tenant_context.permissions())
    {
        return Err(MembershipError::RoleTooPowerful);
    }
    Ok(role)
}

pub async fn list_member_identities(
    app_state: &AppState,
    tenant_id: &TenantId,
) -> Result<Vec<TenantMemberDto>, MembershipError> {
    let rows = repository::list_active_members(&app_state.database_connection, tenant_id).await?;
    let mut members = rows
        .iter()
        .map(member_identity)
        .collect::<Result<Vec<_>, _>>()?;
    members.sort_by(|first, second| first.full_name.cmp(&second.full_name));
    Ok(members)
}

pub async fn list_members(
    app_state: &AppState,
    tenant_context: &TenantContext,
) -> Result<Vec<MemberDto>, MembershipError> {
    let tenant_id = tenant_context.tenant_id();
    let rows = repository::list_active_members(&app_state.database_connection, tenant_id).await?;
    let owner_role = owner_role(app_state, tenant_id).await?;
    let owner_count = rows
        .iter()
        .filter(|row| row.membership.role_id == owner_role.id)
        .filter(|row| {
            row.user
                .as_ref()
                .is_some_and(|user| user.status == UserStatus::Active.as_str())
        })
        .count();
    let may_change_roles = tenant_context
        .permissions()
        .contains(Permission::MemberUpdateRole);
    let may_remove_members = tenant_context
        .permissions()
        .contains(Permission::MemberRemove);

    let mut permissions_by_role_id: HashMap<Uuid, PermissionSet> = HashMap::new();
    for row in &rows {
        if let Entry::Vacant(entry) = permissions_by_role_id.entry(row.membership.role_id) {
            entry.insert(role_permissions(app_state, row.membership.role_id).await?);
        }
    }

    let mut members = Vec::with_capacity(rows.len());
    for row in &rows {
        let role_id = row.membership.role_id;
        let manageable =
            permissions_by_role_id[&role_id].is_subset_of(tenant_context.permissions());
        let is_active_account = row
            .user
            .as_ref()
            .is_some_and(|user| user.status == UserStatus::Active.as_str());
        let is_last_owner = role_id == owner_role.id && is_active_account && owner_count <= 1;
        members.push(MemberDto {
            member: member_identity(row)?,
            capabilities: MemberCapabilities {
                can_change_role: may_change_roles && manageable && !is_last_owner,
                can_remove: may_remove_members && manageable && !is_last_owner,
            },
        });
    }
    members.sort_by(|first, second| first.member.full_name.cmp(&second.member.full_name));
    Ok(members)
}

pub async fn update_member_role(
    app_state: &AppState,
    tenant_context: &TenantContext,
    user_id: Uuid,
    role_name: &str,
) -> Result<MemberDto, MembershipError> {
    authorization_service::require_capability(tenant_context, Permission::MemberUpdateRole)?;
    let tenant_id = tenant_context.tenant_id();
    let membership = find_member(app_state, tenant_id, user_id).await?;
    let target_role = require_grantable_role(app_state, tenant_context, role_name).await?;
    require_manageable(app_state, tenant_context, &membership).await?;

    let owner_role = owner_role(app_state, tenant_id).await?;
    if target_role.id != owner_role.id
        && is_last_owner(app_state, tenant_id, &membership, owner_role.id).await?
    {
        return Err(MembershipError::LastOwner);
    }

    repository::update_membership_role(
        &app_state.database_connection,
        tenant_id,
        membership.id,
        target_role.id,
        &tenant_context.actor(),
    )
    .await?;

    list_members(app_state, tenant_context)
        .await?
        .into_iter()
        .find(|member| member.member.user_id == user_id)
        .ok_or(MembershipError::MemberNotFound)
}

pub async fn remove_member(
    app_state: &AppState,
    tenant_context: &TenantContext,
    user_id: Uuid,
) -> Result<(), MembershipError> {
    authorization_service::require_capability(tenant_context, Permission::MemberRemove)?;
    let tenant_id = tenant_context.tenant_id();
    let membership = find_member(app_state, tenant_id, user_id).await?;
    require_manageable(app_state, tenant_context, &membership).await?;

    let owner_role = owner_role(app_state, tenant_id).await?;
    if is_last_owner(app_state, tenant_id, &membership, owner_role.id).await? {
        return Err(MembershipError::LastOwner);
    }

    let txn = app_state.database_connection.begin().await?;
    repository::delete_membership_transaction(&txn, tenant_id, membership.id).await?;
    crate::grant::service::delete_grants_for_principal_transaction(&txn, tenant_id, user_id)
        .await?;
    crate::tenant::invitation::service::revoke_invitations_sent_by_transaction(
        &txn, tenant_id, user_id,
    )
    .await?;
    txn.commit().await?;
    Ok(())
}

pub async fn appoint_owner(
    app_state: &AppState,
    tenant_id: &TenantId,
    user_id: Uuid,
    actor: &Actor,
) -> Result<(), MembershipError> {
    let owner_role = owner_role(app_state, tenant_id).await?;
    match repository::find_membership(&app_state.database_connection, tenant_id, user_id).await? {
        Some(membership) => {
            repository::make_owner(
                &app_state.database_connection,
                membership.id,
                owner_role.id,
                actor,
            )
            .await?;
        }
        None => {
            repository::insert_membership(
                &app_state.database_connection,
                tenant_id,
                user_id,
                owner_role.id,
                actor,
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn active_owner_counts(
    app_state: &AppState,
) -> Result<HashMap<Uuid, u64>, MembershipError> {
    let mut owner_counts: HashMap<Uuid, u64> = HashMap::new();
    for membership in repository::list_active_memberships_with_role_name(
        &app_state.database_connection,
        SystemRole::Owner.as_str(),
    )
    .await?
    {
        *owner_counts.entry(membership.tenant_id).or_default() += 1;
    }
    Ok(owner_counts)
}

pub async fn list_roles(
    app_state: &AppState,
    tenant_context: &TenantContext,
) -> Result<Vec<RoleDto>, MembershipError> {
    let mut roles = Vec::new();
    for role in authorization_service::list_roles(app_state, tenant_context.tenant_id()).await? {
        let assignable = role_permissions(app_state, role.id)
            .await?
            .is_subset_of(tenant_context.permissions());
        roles.push(RoleDto {
            name: role.name,
            is_system: role.is_system,
            assignable,
        });
    }
    roles.sort_by_key(|role| (system_role_rank(&role.name), role.name.clone()));
    Ok(roles)
}

fn system_role_rank(role_name: &str) -> usize {
    SystemRole::all()
        .iter()
        .position(|system_role| system_role.as_str() == role_name)
        .unwrap_or(SystemRole::all().len())
}

fn member_identity(row: &TenantMemberRow) -> Result<TenantMemberDto, MembershipError> {
    let user = row.user.as_ref().ok_or_else(|| {
        MembershipError::Internal(format!(
            "Membership {} points at a missing user",
            row.membership.id
        ))
    })?;
    let role = row.role.as_ref().ok_or_else(|| {
        MembershipError::Internal(format!(
            "Membership {} points at a missing role",
            row.membership.id
        ))
    })?;
    let account_status = UserStatus::from_str(&user.status).map_err(|_| {
        MembershipError::Internal(format!(
            "User {} has an unknown status {}",
            user.id, user.status
        ))
    })?;
    Ok(TenantMemberDto {
        user_id: user.id,
        full_name: user.full_name.clone(),
        email: user.email.clone(),
        role_name: role.name.clone(),
        account_status,
    })
}

async fn find_member(
    app_state: &AppState,
    tenant_id: &TenantId,
    user_id: Uuid,
) -> Result<tenant_membership::Model, MembershipError> {
    repository::find_active_membership(&app_state.database_connection, tenant_id, user_id)
        .await?
        .ok_or(MembershipError::MemberNotFound)
}

async fn find_role(
    app_state: &AppState,
    tenant_id: &TenantId,
    role_name: &str,
) -> Result<role::Model, MembershipError> {
    authorization_service::find_role_by_name(app_state, tenant_id, role_name)
        .await
        .map_err(|error| match error {
            sea_orm::DbErr::RecordNotFound(_) => MembershipError::UnknownRole,
            other => MembershipError::Database(other),
        })
}

async fn owner_role(
    app_state: &AppState,
    tenant_id: &TenantId,
) -> Result<role::Model, MembershipError> {
    authorization_service::find_role_by_name(app_state, tenant_id, SystemRole::Owner.as_str())
        .await
        .map_err(|error| MembershipError::Internal(error.to_string()))
}

async fn role_permissions(
    app_state: &AppState,
    role_id: Uuid,
) -> Result<PermissionSet, MembershipError> {
    authorization_service::resolve_permissions_for_role(app_state, role_id)
        .await
        .map_err(|error| MembershipError::Internal(error.to_string()))
}

async fn require_manageable(
    app_state: &AppState,
    tenant_context: &TenantContext,
    membership: &tenant_membership::Model,
) -> Result<(), MembershipError> {
    if role_permissions(app_state, membership.role_id)
        .await?
        .is_subset_of(tenant_context.permissions())
    {
        return Ok(());
    }
    Err(MembershipError::MemberTooPowerful)
}

async fn is_last_owner(
    app_state: &AppState,
    tenant_id: &TenantId,
    membership: &tenant_membership::Model,
    owner_role_id: Uuid,
) -> Result<bool, MembershipError> {
    if membership.role_id != owner_role_id {
        return Ok(false);
    }
    let active_owner_user_ids = repository::find_active_member_user_ids_with_role(
        &app_state.database_connection,
        tenant_id,
        owner_role_id,
    )
    .await?;
    Ok(active_owner_user_ids == [membership.user_id])
}
