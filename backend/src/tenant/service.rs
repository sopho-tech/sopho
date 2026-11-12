use crate::authorization::constants::{PlatformPermission, SystemRole};
use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::authorization::dto::UserId;
use crate::authorization::service as authorization_service;
use crate::common::AppState;
use crate::configuration::constants::ConfigurationKey;
use crate::entity::tenant;
use crate::feature_flag::constants::FeatureFlag;
use crate::feature_flag::service as feature_flag_service;
use crate::tenant::constants::{DEFAULT_TENANT_NAME, MAXIMUM_TENANT_NAME_CHARACTERS};
use crate::tenant::dto::{
    TenantCreationCapabilityDto, TenantCreationDecision, TenantMembershipDto,
    UserTenantMembershipDto,
};
use crate::tenant::error::TenantError;
use crate::tenant::membership::service as membership_service;
use crate::tenant::repository;
use uuid::Uuid;

pub async fn create_tenant(
    app_state: &AppState,
    name: String,
    actor: &Actor,
) -> anyhow::Result<tenant::Model> {
    let created_tenant =
        repository::insert_tenant(&app_state.database_connection, name, actor).await?;
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
    let is_self_serve_creation = match decide_tenant_creation(app_state, user_id).await? {
        TenantCreationDecision::AllowedByPlatformPermission => false,
        TenantCreationDecision::AllowedBySelfServe => true,
        TenantCreationDecision::RefusedWithoutPermission {
            authorization_error,
        } => return Err(TenantError::Unauthorized(authorization_error)),
        TenantCreationDecision::RefusedAtLimit { limit } => {
            return Err(TenantError::SelfServeLimitReached { limit });
        }
    };
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(TenantError::NameRequired);
    }
    if name.chars().count() > MAXIMUM_TENANT_NAME_CHARACTERS {
        return Err(TenantError::NameTooLong);
    }

    let creator = Actor::human(user_id);
    let created_tenant = create_tenant(app_state, name, &creator).await?;
    membership_service::add_member_with_role_name(
        app_state,
        &TenantId::from_system_unchecked(created_tenant.id),
        user_id,
        SystemRole::Owner.as_str(),
        &creator,
    )
    .await
    .map_err(anyhow::Error::from)?;
    if is_self_serve_creation {
        repository::insert_self_serve_tenant_creation(
            &app_state.database_connection,
            user_id.get_uuid(),
            created_tenant.id,
        )
        .await
        .map_err(anyhow::Error::from)?;
    }
    Ok(created_tenant)
}

async fn decide_tenant_creation(
    app_state: &AppState,
    user_id: UserId,
) -> Result<TenantCreationDecision, TenantError> {
    let platform_role =
        crate::authentication::service::find_platform_role(app_state, user_id).await?;
    let Err(authorization_error) = authorization_service::require_platform_permission(
        platform_role,
        PlatformPermission::TenantCreate,
    ) else {
        return Ok(TenantCreationDecision::AllowedByPlatformPermission);
    };
    if !feature_flag_service::is_feature_enabled(app_state, FeatureFlag::SelfServeTenantCreation) {
        return Ok(TenantCreationDecision::RefusedWithoutPermission {
            authorization_error,
        });
    }
    let limit = app_state.config.self_serve_tenant_limit;
    let created_tenant_count = repository::count_self_serve_tenant_creations(
        &app_state.database_connection,
        user_id.get_uuid(),
    )
    .await
    .map_err(anyhow::Error::from)?;
    if created_tenant_count >= limit {
        return Ok(TenantCreationDecision::RefusedAtLimit { limit });
    }
    Ok(TenantCreationDecision::AllowedBySelfServe)
}

pub async fn tenant_creation_capability(
    app_state: &AppState,
    user_id: UserId,
) -> Result<TenantCreationCapabilityDto, TenantError> {
    Ok(match decide_tenant_creation(app_state, user_id).await? {
        TenantCreationDecision::AllowedByPlatformPermission
        | TenantCreationDecision::AllowedBySelfServe => TenantCreationCapabilityDto {
            can_create_tenant: true,
            limit_reached_message: None,
        },
        TenantCreationDecision::RefusedWithoutPermission { .. } => TenantCreationCapabilityDto {
            can_create_tenant: false,
            limit_reached_message: None,
        },
        TenantCreationDecision::RefusedAtLimit { limit } => TenantCreationCapabilityDto {
            can_create_tenant: false,
            limit_reached_message: Some(TenantError::SelfServeLimitReached { limit }.to_string()),
        },
    })
}

pub async fn list_all_tenants(app_state: &AppState) -> Result<Vec<tenant::Model>, sea_orm::DbErr> {
    repository::list_all_tenants(&app_state.database_connection).await
}

pub async fn find_tenant(
    app_state: &AppState,
    tenant_id: Uuid,
) -> Result<Option<tenant::Model>, sea_orm::DbErr> {
    repository::find_tenant_by_id(&app_state.database_connection, tenant_id).await
}

pub async fn memberships_by_user(
    app_state: &AppState,
    user_ids: Vec<Uuid>,
) -> Result<std::collections::HashMap<Uuid, Vec<UserTenantMembershipDto>>, TenantError> {
    let mut memberships: std::collections::HashMap<Uuid, Vec<UserTenantMembershipDto>> =
        std::collections::HashMap::new();
    for row in repository::list_memberships_for_users(&app_state.database_connection, user_ids)
        .await
        .map_err(anyhow::Error::from)?
    {
        let (Some(tenant), Some(role)) = (row.tenant, row.role) else {
            return Err(TenantError::Internal(anyhow::anyhow!(
                "Membership {} points at a missing tenant or role",
                row.membership.id
            )));
        };
        memberships
            .entry(row.membership.user_id)
            .or_default()
            .push(UserTenantMembershipDto {
                tenant_id: tenant.id,
                tenant_name: tenant.name,
                role_display_name: authorization_service::role_display_name(&role.name),
                role_name: role.name,
            });
    }
    for user_memberships in memberships.values_mut() {
        user_memberships.sort_by(|first, second| first.tenant_name.cmp(&second.tenant_name));
    }
    Ok(memberships)
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
            role_display_name: authorization_service::role_display_name(&role.name),
            role_name: role.name,
            permissions,
        });
    }
    Ok(response_dto_list)
}

async fn seed_default_tenant(app_state: &AppState) -> anyhow::Result<Uuid> {
    if let Some(recorded_tenant_id) = crate::configuration::service::get_configuration_value(
        app_state,
        ConfigurationKey::DefaultTenantId,
    )
    .await?
    {
        let default_tenant_id = Uuid::parse_str(&recorded_tenant_id)?;
        if repository::find_tenant_by_id(&app_state.database_connection, default_tenant_id)
            .await?
            .is_none()
        {
            anyhow::bail!(
                "configuration {} points at tenant {default_tenant_id}, which does not exist",
                ConfigurationKey::DefaultTenantId
            );
        }
        return Ok(default_tenant_id);
    }

    let created_tenant =
        create_tenant(app_state, DEFAULT_TENANT_NAME.to_string(), &Actor::system()).await?;

    crate::configuration::service::set_configuration_value(
        app_state,
        ConfigurationKey::DefaultTenantId,
        &created_tenant.id.to_string(),
    )
    .await?;

    Ok(created_tenant.id)
}

pub async fn seed_default_tenant_with_owner(
    app_state: &AppState,
    owner_user_id: Uuid,
) -> anyhow::Result<Uuid> {
    let default_tenant_id = seed_default_tenant(app_state).await?;

    let existing_membership =
        authorization_service::find_membership(app_state, default_tenant_id, owner_user_id).await?;

    if existing_membership.is_some() {
        return Ok(default_tenant_id);
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
        UserId::from_system_unchecked(owner_user_id),
        owner_role.id,
        &Actor::system(),
    )
    .await?;

    tracing::info!(
        "Seeded OWNER membership for admin user in tenant {}",
        default_tenant_id
    );
    Ok(default_tenant_id)
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
