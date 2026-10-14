use crate::authorization::constants::{PlatformPermission, SystemRole};
use crate::authorization::dto::TenantId;
use crate::authorization::dto::UserId;
use crate::authorization::service as authorization_service;
use crate::common::AppState;
use crate::configuration::constants::ConfigurationKey;
use crate::entity::tenant;
use crate::tenant::constants::DEFAULT_TENANT_NAME;
use crate::tenant::dto::TenantMembershipDto;
use crate::tenant::error::TenantError;
use crate::tenant::membership::service as membership_service;
use crate::tenant::repository;
use uuid::Uuid;

pub async fn create_tenant(app_state: &AppState, name: String) -> anyhow::Result<tenant::Model> {
    let created_tenant = repository::insert_tenant(&app_state.database_connection, name).await?;
    authorization_service::seed_system_roles_for_tenant(
        app_state,
        &TenantId::from_system_unchecked(created_tenant.id),
    )
    .await?;
    Ok(created_tenant)
}

pub async fn create_tenant_for_user(
    app_state: &AppState,
    user_id: UserId,
    name: String,
) -> Result<tenant::Model, TenantError> {
    let platform_role =
        crate::authentication::service::find_platform_role(app_state, user_id).await?;
    authorization_service::require_platform_permission(
        platform_role,
        PlatformPermission::TenantCreate,
    )?;
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(TenantError::NameRequired);
    }

    let created_tenant = create_tenant(app_state, name).await?;
    membership_service::add_member_with_role_name(
        app_state,
        &TenantId::from_system_unchecked(created_tenant.id),
        user_id,
        SystemRole::Owner.as_str(),
    )
    .await
    .map_err(anyhow::Error::from)?;
    Ok(created_tenant)
}

pub async fn find_tenant_names(
    app_state: &AppState,
    tenant_ids: Vec<Uuid>,
) -> Result<std::collections::HashMap<Uuid, String>, sea_orm::DbErr> {
    Ok(
        repository::find_tenants_by_ids(&app_state.database_connection, tenant_ids)
            .await?
            .into_iter()
            .map(|found| (found.id, found.name))
            .collect(),
    )
}

pub async fn list_tenants_for_user(
    app_state: &AppState,
    user_id: UserId,
) -> anyhow::Result<Vec<TenantMembershipDto>> {
    let memberships =
        repository::list_memberships_for_user(&app_state.database_connection, user_id.get_uuid())
            .await?;

    let mut response_dto_list = Vec::with_capacity(memberships.len());
    for row in memberships {
        let tenant = row.tenant.ok_or_else(|| {
            anyhow::anyhow!(
                "Membership {} points at a missing tenant",
                row.membership.id
            )
        })?;
        let role = row.role.ok_or_else(|| {
            anyhow::anyhow!("Membership {} points at a missing role", row.membership.id)
        })?;
        let permissions = authorization_service::resolve_permissions_for_role(app_state, role.id)
            .await?
            .as_strings();
        response_dto_list.push(TenantMembershipDto {
            tenant_id: tenant.id,
            tenant_name: tenant.name,
            role_name: role.name,
            permissions,
        });
    }
    Ok(response_dto_list)
}

pub async fn seed_default_tenant(app_state: &AppState) -> anyhow::Result<Uuid> {
    if let Some(existing_tenant_id) = crate::configuration::service::get_configuration_value(
        app_state,
        ConfigurationKey::DefaultTenantId,
    )
    .await?
    {
        return Ok(Uuid::parse_str(&existing_tenant_id)?);
    }

    let created_tenant = repository::insert_tenant(
        &app_state.database_connection,
        DEFAULT_TENANT_NAME.to_string(),
    )
    .await?;

    crate::configuration::service::set_configuration_value(
        app_state,
        ConfigurationKey::DefaultTenantId,
        &created_tenant.id.to_string(),
    )
    .await?;

    Ok(created_tenant.id)
}

pub async fn ensure_default_tenant_membership(app_state: &AppState) -> anyhow::Result<()> {
    let default_tenant_id = seed_default_tenant(app_state).await?;

    let admin_email = app_state.config.admin_email.as_ref();
    let admin_user =
        crate::authentication::service::find_user_by_email(app_state, admin_email).await?;

    let existing_membership =
        authorization_service::find_membership(app_state, default_tenant_id, admin_user.id).await?;

    if existing_membership.is_some() {
        return Ok(());
    }

    let owner_role = authorization_service::find_role_by_name(
        app_state,
        &TenantId::from_system_unchecked(default_tenant_id),
        SystemRole::Owner.as_str(),
    )
    .await?;

    membership_service::add_member(
        app_state,
        &TenantId::from_system_unchecked(default_tenant_id),
        UserId::from_system_unchecked(admin_user.id),
        owner_role.id,
    )
    .await?;

    tracing::info!(
        "Seeded OWNER membership for admin user in tenant {}",
        default_tenant_id
    );
    Ok(())
}

pub async fn list_all_tenant_ids(app_state: &AppState) -> anyhow::Result<Vec<Uuid>> {
    Ok(repository::list_all_tenant_ids(&app_state.database_connection).await?)
}

pub async fn reconcile_system_roles_for_all_tenants(app_state: &AppState) -> anyhow::Result<()> {
    for tenant_id in list_all_tenant_ids(app_state).await? {
        authorization_service::seed_system_roles_for_tenant(
            app_state,
            &TenantId::from_system_unchecked(tenant_id),
        )
        .await?;
    }
    Ok(())
}
