use crate::authorization::constants::{AccessLevel, Action, ResourceType, Visibility};
use crate::authorization::dto::{AuthorizationSubject, TenantContext, TenantId};
use crate::authorization::service as authorization_service;
use crate::common::AppState;
use crate::grant::constants::{PRINCIPAL_IS_OWNER_MESSAGE, PRINCIPAL_NOT_A_MEMBER_MESSAGE};
use crate::grant::dto::{CreateGrantDto, GrantDto, ResourceSharingDto, UpdateVisibilityDto};
use crate::grant::error::GrantError;
use crate::grant::repository;
use crate::tenant::dto::TenantMemberDto;
use crate::tenant::membership::service as membership_service;
use std::collections::HashMap;
use uuid::Uuid;

async fn load_subject(
    app_state: &AppState,
    tenant_context: &TenantContext,
    resource_type: ResourceType,
    resource_id: Uuid,
) -> Result<AuthorizationSubject, GrantError> {
    let tenant_id = tenant_context.tenant_id();
    let loaded = match resource_type {
        ResourceType::Canvas => {
            crate::canvas::service::load_authorization_subject(app_state, tenant_id, resource_id)
                .await
        }
        ResourceType::Conversation => {
            crate::conversational_analytics::conversation::service::load_authorization_subject(
                app_state,
                tenant_id,
                resource_id,
            )
            .await
        }
        ResourceType::Connection => {
            crate::connection::service::load_authorization_subject(
                app_state,
                tenant_id,
                resource_id,
            )
            .await
        }
    };

    loaded.map_err(|_| GrantError::ResourceNotFound)
}

async fn require_share_rights(
    app_state: &AppState,
    tenant_context: &TenantContext,
    subject: &AuthorizationSubject,
) -> Result<(), GrantError> {
    authorization_service::require_access(app_state, tenant_context, subject, Action::Share)
        .await?;
    Ok(())
}

async fn members_by_user_id(
    app_state: &AppState,
    tenant_context: &TenantContext,
) -> Result<HashMap<Uuid, TenantMemberDto>, GrantError> {
    let members = membership_service::list_member_identities(app_state, tenant_context.tenant_id())
        .await
        .map_err(|error| GrantError::Internal(error.to_string()))?;
    Ok(members
        .into_iter()
        .map(|member| (member.user_id, member))
        .collect())
}

fn grant_dto(
    grant: crate::entity::resource_grant::Model,
    members: &HashMap<Uuid, TenantMemberDto>,
) -> GrantDto {
    GrantDto {
        id: grant.id,
        principal: members.get(&grant.principal_id).cloned(),
        resource_type: grant.resource_type,
        resource_id: grant.resource_id,
        principal_user_id: grant.principal_id,
        access_level: grant.access_level,
    }
}

async fn sharing_state(
    app_state: &AppState,
    tenant_context: &TenantContext,
    subject: &AuthorizationSubject,
) -> Result<ResourceSharingDto, GrantError> {
    let members = members_by_user_id(app_state, tenant_context).await?;
    let grants = repository::list_grants(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        subject.resource_type().as_str(),
        subject.resource_id(),
    )
    .await?;

    Ok(ResourceSharingDto {
        resource_type: subject.resource_type().as_str().to_string(),
        resource_id: subject.resource_id(),
        visibility: subject.visibility().as_str().to_string(),
        owner: members.get(&subject.owner_user_id()).cloned(),
        grants: grants
            .into_iter()
            .map(|grant| grant_dto(grant, &members))
            .collect(),
    })
}

async fn set_resource_visibility(
    app_state: &AppState,
    tenant_context: &TenantContext,
    resource_type: ResourceType,
    resource_id: Uuid,
    visibility: Visibility,
) -> Result<(), GrantError> {
    let tenant_id = tenant_context.tenant_id();
    let updated = match resource_type {
        ResourceType::Canvas => {
            crate::canvas::service::set_visibility(app_state, tenant_id, resource_id, visibility)
                .await
        }
        ResourceType::Conversation => {
            crate::conversational_analytics::conversation::service::set_visibility(
                app_state,
                tenant_id,
                resource_id,
                visibility,
            )
            .await
        }
        ResourceType::Connection => {
            crate::connection::service::set_visibility(
                app_state,
                tenant_id,
                resource_id,
                visibility,
            )
            .await
        }
    };
    updated.map_err(|error| match error {
        sea_orm::DbErr::RecordNotFound(_) => GrantError::ResourceNotFound,
        other => GrantError::Database(other),
    })
}

pub async fn create_grant(
    app_state: AppState,
    tenant_context: &TenantContext,
    payload: CreateGrantDto,
) -> Result<GrantDto, GrantError> {
    let resource_type =
        ResourceType::from_str(&payload.resource_type).map_err(GrantError::InvalidRequest)?;
    let access_level =
        AccessLevel::from_str(&payload.access_level).map_err(GrantError::InvalidRequest)?;

    let subject = load_subject(
        &app_state,
        tenant_context,
        resource_type,
        payload.resource_id,
    )
    .await?;
    require_share_rights(&app_state, tenant_context, &subject).await?;

    if payload.principal_user_id == subject.owner_user_id() {
        return Err(GrantError::InvalidRequest(
            PRINCIPAL_IS_OWNER_MESSAGE.to_string(),
        ));
    }
    let members = members_by_user_id(&app_state, tenant_context).await?;
    if !members.contains_key(&payload.principal_user_id) {
        return Err(GrantError::InvalidRequest(
            PRINCIPAL_NOT_A_MEMBER_MESSAGE.to_string(),
        ));
    }

    let grant = repository::upsert_grant(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        resource_type.as_str(),
        payload.resource_id,
        payload.principal_user_id,
        access_level.as_str(),
        tenant_context.user_id().get_uuid(),
    )
    .await?;

    Ok(grant_dto(grant, &members))
}

pub async fn get_sharing(
    app_state: AppState,
    tenant_context: &TenantContext,
    resource_type_value: String,
    resource_id: Uuid,
) -> Result<ResourceSharingDto, GrantError> {
    let resource_type =
        ResourceType::from_str(&resource_type_value).map_err(GrantError::InvalidRequest)?;

    let subject = load_subject(&app_state, tenant_context, resource_type, resource_id).await?;
    require_share_rights(&app_state, tenant_context, &subject).await?;

    sharing_state(&app_state, tenant_context, &subject).await
}

pub async fn update_visibility(
    app_state: AppState,
    tenant_context: &TenantContext,
    resource_type_value: String,
    resource_id: Uuid,
    payload: UpdateVisibilityDto,
) -> Result<ResourceSharingDto, GrantError> {
    let resource_type =
        ResourceType::from_str(&resource_type_value).map_err(GrantError::InvalidRequest)?;
    let visibility =
        Visibility::from_str(&payload.visibility).map_err(GrantError::InvalidRequest)?;

    let subject = load_subject(&app_state, tenant_context, resource_type, resource_id).await?;
    require_share_rights(&app_state, tenant_context, &subject).await?;

    set_resource_visibility(
        &app_state,
        tenant_context,
        resource_type,
        resource_id,
        visibility,
    )
    .await?;

    let updated_subject =
        load_subject(&app_state, tenant_context, resource_type, resource_id).await?;
    sharing_state(&app_state, tenant_context, &updated_subject).await
}

pub async fn delete_grants_for_principal_transaction(
    txn: &sea_orm::DatabaseTransaction,
    tenant_id: &TenantId,
    principal_id: Uuid,
) -> Result<u64, sea_orm::DbErr> {
    repository::delete_grants_for_principal_transaction(txn, tenant_id, principal_id).await
}

pub async fn revoke_grant(
    app_state: AppState,
    tenant_context: &TenantContext,
    grant_id: Uuid,
) -> Result<(), GrantError> {
    let grant = repository::find_grant_by_id(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        grant_id,
    )
    .await?
    .ok_or(GrantError::GrantNotFound)?;

    let resource_type =
        ResourceType::from_str(&grant.resource_type).map_err(GrantError::InvalidRequest)?;
    let subject =
        load_subject(&app_state, tenant_context, resource_type, grant.resource_id).await?;
    require_share_rights(&app_state, tenant_context, &subject).await?;

    repository::revoke_grant(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        grant_id,
    )
    .await?;

    Ok(())
}
