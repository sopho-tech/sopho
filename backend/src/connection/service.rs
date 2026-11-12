use crate::attribution::service::{self as attribution_service, AttributionNames};
use crate::authorization::constants::{Action, ResourceType, Visibility};
use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::authorization::dto::{AuthorizationSubject, TenantContext};
use crate::authorization::errors::AuthorizationError;
use crate::authorization::service as authorization_service;
use crate::common::AppState;
use crate::common::constants::CreatedByFilter;
use crate::common::errors::{
    CreateConnectionError, ExecuteSqlError, GetDatabaseConnectionError, UpdateConnectionError,
};
use crate::common::time_utils;
use crate::connection::constants::CONNECTION_NOT_FOUND_MESSAGE;
use crate::connection::constants::ConnectionStatus;
use crate::connection::constants::SourceType;
use crate::connection::cryptography_utils;
use crate::connection::dto;
use crate::connection::error::ConnectionError;
use crate::connection::repository;
use crate::database::constants::QueryResult;
use crate::database::service as database_service;
use crate::entity;

use tracing::info;
use uuid::Uuid;

pub(crate) fn reachable_connection_condition(tenant_context: &TenantContext) -> sea_orm::Condition {
    authorization_service::reachable_resource_condition(
        tenant_context,
        ResourceType::Connection,
        entity::connection::Column::Id,
        entity::connection::Column::OwnerUserId,
        entity::connection::Column::Visibility,
    )
}

async fn authorize_connection(
    app_state: &AppState,
    tenant_context: &TenantContext,
    id: Uuid,
    action: Action,
) -> Result<entity::connection::Model, ConnectionError> {
    let connection = repository::get_connection(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        id,
    )
    .await?
    .ok_or(ConnectionError::NotFound)?;

    let subject = authorization_service::subject_from_parts(
        tenant_context.tenant_id(),
        ResourceType::Connection,
        connection.id,
        connection.owner_user_id,
        &connection.visibility,
    )
    .map_err(ConnectionError::Internal)?;

    authorization_service::require_access(app_state, tenant_context, &subject, action).await?;

    Ok(connection)
}

pub async fn get_connection(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
) -> Result<dto::ConnectionDto, ConnectionError> {
    let authorized = authorize_connection(&app_state, tenant_context, id, Action::Read).await?;

    let connection = execute_get_connection(&app_state, tenant_context.tenant_id(), id).await?;

    let subject = connection_subject(tenant_context, &authorized)?;
    connection_response_dto(&app_state, tenant_context, &subject, connection).await
}

async fn connection_response_dto(
    app_state: &AppState,
    tenant_context: &TenantContext,
    subject: &AuthorizationSubject,
    connection: entity::connection::Model,
) -> Result<dto::ConnectionDto, ConnectionError> {
    let names = attribution_service::load_names(
        app_state,
        [
            connection.created_by_user_id,
            connection.modified_by_user_id,
        ],
    )
    .await?;
    connection_response_dto_with_names(app_state, tenant_context, subject, connection, &names).await
}

async fn connection_response_dto_with_names(
    app_state: &AppState,
    tenant_context: &TenantContext,
    subject: &AuthorizationSubject,
    connection: entity::connection::Model,
    names: &AttributionNames,
) -> Result<dto::ConnectionDto, ConnectionError> {
    let created_by = names.describe(&connection.created_by_actor, connection.created_by_user_id)?;
    let modified_by = names.describe(
        &connection.modified_by_actor,
        connection.modified_by_user_id,
    )?;
    let source_type =
        SourceType::from_str(&connection.source_type).map_err(ConnectionError::Internal)?;
    let is_source_type_available =
        database_service::is_source_type_available(app_state, source_type);
    let mut response_dto =
        dto::ConnectionDto::from_connection(connection, source_type, is_source_type_available);
    response_dto.capabilities = Some(
        authorization_service::resolve_capabilities(app_state, tenant_context, subject).await?,
    );
    response_dto.created_by = Some(created_by);
    response_dto.modified_by = Some(modified_by);
    Ok(response_dto)
}

fn connection_subject(
    tenant_context: &TenantContext,
    connection: &entity::connection::Model,
) -> Result<AuthorizationSubject, ConnectionError> {
    authorization_service::subject_from_parts(
        tenant_context.tenant_id(),
        ResourceType::Connection,
        connection.id,
        connection.owner_user_id,
        &connection.visibility,
    )
    .map_err(ConnectionError::Internal)
}

pub fn offered_source_types(app_state: &AppState) -> Vec<dto::OfferedSourceTypeDto> {
    database_service::offered_source_types(app_state)
        .into_iter()
        .map(|source_type| dto::OfferedSourceTypeDto {
            source_type,
            display_name: source_type.display_name(),
        })
        .collect()
}

pub async fn execute_get_connection(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<entity::connection::Model, GetDatabaseConnectionError> {
    let mut connection = repository::get_connection(&app_state.database_connection, tenant_id, id)
        .await
        .map_err(GetDatabaseConnectionError::Connection)?
        .ok_or(GetDatabaseConnectionError::ConnectionNotFound)?;
    cryptography_utils::decrypt_connection(&mut connection, &app_state.config.encryption_key)
        .map_err(GetDatabaseConnectionError::Decryption)?;
    Ok(connection)
}

pub fn ensure_connection_can_be_used(
    app_state: &AppState,
    connection: &entity::connection::Model,
) -> Result<(), GetDatabaseConnectionError> {
    let source_type = SourceType::from_str(&connection.source_type)
        .map_err(GetDatabaseConnectionError::SourceTypeParse)?;
    Ok(database_service::ensure_source_type_is_enabled(
        app_state,
        source_type,
    )?)
}

pub async fn execute_get_all_connections(
    app_state: &AppState,
    tenant_id: &TenantId,
) -> Result<Vec<entity::connection::Model>, GetDatabaseConnectionError> {
    let mut connections =
        repository::get_all_connections(&app_state.database_connection, tenant_id)
            .await
            .map_err(GetDatabaseConnectionError::Connection)?;
    for connection in connections.iter_mut() {
        cryptography_utils::decrypt_connection(connection, &app_state.config.encryption_key)
            .map_err(GetDatabaseConnectionError::Decryption)?;
    }
    Ok(connections)
}

pub async fn get_all_connections(
    app_state: AppState,
    tenant_context: &TenantContext,
    created_by: Option<CreatedByFilter>,
) -> Result<Vec<dto::ConnectionDto>, ConnectionError> {
    authorization_service::require_capability(
        tenant_context,
        ResourceType::Connection.permission_for(Action::Read),
    )?;

    let created_by_user_id = created_by.map(|filter| match filter {
        CreatedByFilter::Me => tenant_context.user_id().get_uuid(),
    });
    let mut connections = repository::get_visible_connections(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        reachable_connection_condition(tenant_context),
        created_by_user_id,
    )
    .await?;

    for connection in connections.iter_mut() {
        cryptography_utils::decrypt_connection(connection, &app_state.config.encryption_key)
            .map_err(GetDatabaseConnectionError::Decryption)?;
    }

    let names = attribution_service::load_names(
        &app_state,
        connections.iter().flat_map(|connection| {
            [
                connection.created_by_user_id,
                connection.modified_by_user_id,
            ]
        }),
    )
    .await?;
    let mut response_dto_list = Vec::with_capacity(connections.len());
    for connection in connections {
        let subject = connection_subject(tenant_context, &connection)?;
        response_dto_list.push(
            connection_response_dto_with_names(
                &app_state,
                tenant_context,
                &subject,
                connection,
                &names,
            )
            .await?,
        );
    }
    Ok(response_dto_list)
}

pub async fn execute_create_connection(
    app_state: &AppState,
    tenant_id: &TenantId,
    owner_user_id: Uuid,
    payload: dto::CreateConnectionDto,
    actor: &Actor,
) -> Result<entity::connection::Model, CreateConnectionError> {
    let mut connection_entity = entity::connection::Model {
        id: Uuid::new_v4(),
        tenant_id: tenant_id.get_uuid(),
        owner_user_id,
        visibility: Visibility::Tenant.as_str().to_string(),
        name: payload.name,
        description: payload.description,
        status: ConnectionStatus::Active.to_string(),
        database: payload.database,
        host: payload.host,
        password: payload.password,
        port: payload.port,
        schema: payload.schema,
        username: payload.username,
        source_type: payload.source_type.to_string(),
        created_by_user_id: actor.user_id(),
        created_by_actor: actor.kind_name(),
        modified_by_user_id: actor.user_id(),
        modified_by_actor: actor.kind_name(),
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
    };

    let connection_status =
        database_service::probe_connection_status(app_state, &connection_entity).await?;
    connection_entity.status = connection_status.to_string();

    cryptography_utils::encrypt_connection(
        &mut connection_entity,
        &app_state.config.encryption_key,
    )
    .map_err(|_| CreateConnectionError::Encryption)?;

    let mut connection =
        repository::save_connection(&app_state.database_connection, tenant_id, connection_entity)
            .await
            .map_err(CreateConnectionError::Repository)?;

    cryptography_utils::decrypt_connection(&mut connection, &app_state.config.encryption_key)
        .map_err(CreateConnectionError::Decryption)?;

    Ok(connection)
}

pub async fn create_connection(
    app_state: &AppState,
    tenant_context: &TenantContext,
    payload: dto::CreateConnectionDto,
) -> Result<dto::ConnectionDto, ConnectionError> {
    crate::authorization::service::require_capability(
        tenant_context,
        crate::authorization::constants::Permission::ConnectionCreate,
    )?;

    let connection = execute_create_connection(
        app_state,
        tenant_context.tenant_id(),
        tenant_context.user_id().get_uuid(),
        payload,
        &tenant_context.actor(),
    )
    .await?;

    let subject = connection_subject(tenant_context, &connection)?;
    connection_response_dto(app_state, tenant_context, &subject, connection).await
}

async fn execute_update_connection(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
    payload: dto::UpdateConnectionDto,
    actor: &Actor,
) -> Result<entity::connection::Model, UpdateConnectionError> {
    let stored_connection = execute_get_connection(app_state, tenant_id, id).await?;
    let password = payload.password.or(stored_connection.password);
    let mut connection_entity = entity::connection::Model {
        name: payload.name,
        description: payload.description,
        database: payload.database,
        host: payload.host,
        port: payload.port,
        schema: payload.schema,
        username: payload.username,
        password,
        updated_at: time_utils::now_utc_into(),
        modified_by_user_id: actor.user_id(),
        modified_by_actor: actor.kind_name(),
        ..stored_connection
    };

    let connection_status =
        database_service::probe_connection_status(app_state, &connection_entity).await?;
    connection_entity.status = connection_status.to_string();

    cryptography_utils::encrypt_connection(
        &mut connection_entity,
        &app_state.config.encryption_key,
    )
    .map_err(|_| UpdateConnectionError::Encryption)?;

    let mut connection =
        repository::update_connection(&app_state.database_connection, connection_entity).await?;

    cryptography_utils::decrypt_connection(&mut connection, &app_state.config.encryption_key)
        .map_err(UpdateConnectionError::Decryption)?;

    Ok(connection)
}

pub async fn update_connection(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
    payload: dto::UpdateConnectionDto,
) -> Result<dto::ConnectionDto, ConnectionError> {
    authorize_connection(&app_state, tenant_context, id, Action::Write).await?;

    let connection = execute_update_connection(
        &app_state,
        tenant_context.tenant_id(),
        id,
        payload,
        &tenant_context.actor(),
    )
    .await?;

    let subject = connection_subject(tenant_context, &connection)?;
    let response_dto =
        connection_response_dto(&app_state, tenant_context, &subject, connection).await?;
    info!("Connection updated: {:?}", response_dto);
    Ok(response_dto)
}

pub async fn execute_query(
    app_state: &AppState,
    tenant_context: &TenantContext,
    id: Uuid,
    payload: dto::ExecuteQueryDto,
) -> Result<QueryResult, ExecuteSqlError> {
    authorize_connection(app_state, tenant_context, id, Action::Read)
        .await
        .map_err(|_| {
            ExecuteSqlError::GetConnection(
                crate::common::errors::GetDatabaseConnectionError::ConnectionNotFound,
            )
        })?;
    let connection = execute_get_connection(app_state, tenant_context.tenant_id(), id).await?;
    database_service::execute_sql_query(app_state, &connection, &payload.query).await
}

pub async fn load_authorization_subject(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<AuthorizationSubject, sea_orm::DbErr> {
    let connection = repository::get_connection(&app_state.database_connection, tenant_id, id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(CONNECTION_NOT_FOUND_MESSAGE.into()))?;
    authorization_service::subject_from_parts(
        tenant_id,
        ResourceType::Connection,
        connection.id,
        connection.owner_user_id,
        &connection.visibility,
    )
    .map_err(sea_orm::DbErr::Custom)
}

pub async fn require_connection_access(
    app_state: &AppState,
    tenant_context: &TenantContext,
    id: Uuid,
    action: Action,
) -> Result<(), AuthorizationError> {
    let subject = match load_authorization_subject(app_state, tenant_context.tenant_id(), id).await
    {
        Ok(subject) => subject,
        Err(sea_orm::DbErr::RecordNotFound(_)) => return Err(AuthorizationError::NotReachable),
        Err(database_error) => return Err(AuthorizationError::Database(database_error)),
    };
    authorization_service::require_access(app_state, tenant_context, &subject, action).await
}

pub async fn set_visibility(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
    visibility: Visibility,
    actor: &Actor,
) -> Result<(), sea_orm::DbErr> {
    let updated_rows = repository::update_connection_visibility(
        &app_state.database_connection,
        tenant_id,
        id,
        visibility.as_str(),
        actor,
    )
    .await?;
    if updated_rows == 0 {
        return Err(sea_orm::DbErr::RecordNotFound(
            CONNECTION_NOT_FOUND_MESSAGE.into(),
        ));
    }
    Ok(())
}
