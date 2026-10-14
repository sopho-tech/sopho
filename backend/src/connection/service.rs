use crate::authorization::constants::{Action, ResourceType, Visibility};
use crate::authorization::dto::TenantId;
use crate::authorization::dto::{AuthorizationSubject, TenantContext};
use crate::authorization::service as authorization_service;
use crate::common::errors::{CreateConnectionError, ExecuteSqlError, GetDatabaseConnectionError};
use crate::common::time_utils;
use crate::common::AppState;
use crate::connection::constants::ConnectionStatus;
use crate::connection::constants::SourceType;
use crate::connection::constants::CONNECTION_NOT_FOUND_MESSAGE;
use crate::connection::cryptography_utils;
use crate::connection::dto;
use crate::connection::error::ConnectionError;
use crate::connection::repository;
use crate::database::constants::QueryResult;
use crate::database::postgres;
use crate::database::service as database_service;
use crate::database::sqlite;
use crate::entity;
use crate::suggested_question;

use tracing::info;
use uuid::Uuid;

pub async fn does_connection_exist(app_state: &AppState, tenant_id: &TenantId, id: Uuid) -> bool {
    repository::get_connection(&app_state.database_connection, tenant_id, id)
        .await
        .is_ok()
}

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

    let connection = execute_get_connection(&app_state, tenant_context.tenant_id(), id)
        .await
        .map_err(|_| ConnectionError::NotFound)?;

    let subject = connection_subject(tenant_context, &authorized)?;
    let mut response_dto = dto::ConnectionDto::from(connection);
    response_dto.capabilities = Some(
        authorization_service::resolve_capabilities(&app_state, tenant_context, &subject).await,
    );
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

pub async fn execute_get_connection(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<entity::connection::Model, sea_orm::DbErr> {
    let mut connection = repository::get_connection(&app_state.database_connection, tenant_id, id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(CONNECTION_NOT_FOUND_MESSAGE.into()))?;
    cryptography_utils::decrypt_connection(&mut connection, &app_state.config.encryption_key);
    Ok(connection)
}

pub async fn execute_get_all_connections(
    app_state: &AppState,
    tenant_id: &TenantId,
) -> Result<Vec<entity::connection::Model>, sea_orm::DbErr> {
    let mut connections =
        repository::get_all_connections(&app_state.database_connection, tenant_id).await?;
    for connection in connections.iter_mut() {
        cryptography_utils::decrypt_connection(connection, &app_state.config.encryption_key);
    }
    Ok(connections)
}

pub async fn get_all_connections(
    app_state: AppState,
    tenant_context: &TenantContext,
) -> Result<Vec<dto::ConnectionDto>, ConnectionError> {
    authorization_service::require_capability(
        tenant_context,
        ResourceType::Connection.permission_for(Action::Read),
    )?;

    let mut connections = repository::get_visible_connections(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        reachable_connection_condition(tenant_context),
    )
    .await?;

    for connection in connections.iter_mut() {
        cryptography_utils::decrypt_connection(connection, &app_state.config.encryption_key);
    }

    let mut response_dto_list = Vec::with_capacity(connections.len());
    for connection in connections {
        let subject = connection_subject(tenant_context, &connection)?;
        let mut response_dto = dto::ConnectionDto::from(connection);
        response_dto.capabilities = Some(
            authorization_service::resolve_capabilities(&app_state, tenant_context, &subject).await,
        );
        response_dto_list.push(response_dto);
    }
    Ok(response_dto_list)
}

async fn get_connection_status(
    connection_entity: &entity::connection::Model,
) -> Result<ConnectionStatus, crate::common::errors::ValidationError> {
    let source_type = SourceType::from_str(&connection_entity.source_type).unwrap();
    match source_type {
        SourceType::Postgresql | SourceType::Supabase => {
            Ok(postgres::get_connection_status(connection_entity).await)
        }
        SourceType::Sqlite => sqlite::get_connection_status(connection_entity).await,
        _ => panic!("Unsupported source type"),
    }
}

pub async fn execute_create_connection(
    app_state: &AppState,
    tenant_id: &TenantId,
    owner_user_id: Uuid,
    payload: dto::CreateConnectionDto,
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
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
    };

    let connection_status = get_connection_status(&connection_entity)
        .await
        .map_err(CreateConnectionError::ConnectionStatus)?;
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

    cryptography_utils::decrypt_connection(&mut connection, &app_state.config.encryption_key);

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
    )
    .await?;

    Ok(dto::ConnectionDto::from(connection))
}

pub async fn execute_delete_connection(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<sea_orm::DeleteResult, sea_orm::DbErr> {
    suggested_question::service::delete_for_connection(app_state, tenant_id, id).await?;
    repository::delete_connection(&app_state.database_connection, tenant_id, id).await
}

pub async fn update_connection(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
    mut payload: dto::ConnectionDto,
) -> Result<dto::ConnectionDto, ConnectionError> {
    authorize_connection(&app_state, tenant_context, id, Action::Write).await?;

    let tenant_id = tenant_context.tenant_id();
    let connection = repository::get_connection(&app_state.database_connection, tenant_id, id)
        .await?
        .ok_or(ConnectionError::NotFound)?;

    let mut status_check_entity = connection.clone();
    status_check_entity.database = payload.database.clone();
    payload.status = get_connection_status(&status_check_entity)
        .await
        .map_err(|error| ConnectionError::InvalidRequest(error.to_string()))?;

    if cryptography_utils::encrypt_connection_dto(&mut payload, &app_state.config.encryption_key)
        .is_err()
    {
        return Err(ConnectionError::Encryption);
    }

    let mut connection =
        repository::update_connection(&app_state.database_connection, tenant_id, id, payload)
            .await?
            .ok_or(ConnectionError::NotFound)?;
    cryptography_utils::decrypt_connection(&mut connection, &app_state.config.encryption_key);

    let response_dto = dto::ConnectionDto::from(connection);
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
    let tenant_id = tenant_context.tenant_id();
    let connection = execute_get_connection(app_state, tenant_id, id)
        .await
        .map_err(|e| -> ExecuteSqlError {
            match e {
                sea_orm::DbErr::RecordNotFound(_) => {
                    GetDatabaseConnectionError::ConnectionNotFound.into()
                }
                other => GetDatabaseConnectionError::Connection(other).into(),
            }
        })?;
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
