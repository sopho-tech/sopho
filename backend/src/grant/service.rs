use crate::authorization::constants::{AccessLevel, Action, ResourceType};
use crate::authorization::dto::{AuthorizationSubject, TenantContext};
use crate::authorization::service as authorization_service;
use crate::common::AppState;
use crate::grant::dto::{CreateGrantDto, GrantDto};
use crate::grant::error::GrantError;
use crate::grant::repository;
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

    Ok(GrantDto::from(grant))
}

pub async fn list_grants(
    app_state: AppState,
    tenant_context: &TenantContext,
    resource_type_value: String,
    resource_id: Uuid,
) -> Result<Vec<GrantDto>, GrantError> {
    let resource_type =
        ResourceType::from_str(&resource_type_value).map_err(GrantError::InvalidRequest)?;

    let subject = load_subject(&app_state, tenant_context, resource_type, resource_id).await?;
    require_share_rights(&app_state, tenant_context, &subject).await?;

    let grants = repository::list_grants(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        resource_type.as_str(),
        resource_id,
    )
    .await?;

    Ok(grants.into_iter().map(GrantDto::from).collect())
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
