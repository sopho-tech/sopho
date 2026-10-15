use crate::common::AppState;
use crate::common::errors::{
    CreateConnectionError, ExecuteSqlError, GetDatabaseConnectionError, UpdateConnectionError,
};
use crate::common::time_utils;
use crate::connection::constants::ConnectionStatus;
use crate::connection::constants::SourceType;
use crate::connection::cryptography_utils;
use crate::connection::dto;
use crate::connection::repository;
use crate::database::constants::QueryResult;
use crate::database::postgres;
use crate::database::service as database_service;
use crate::database::sqlite;
use crate::entity;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use tracing::info;
use uuid::Uuid;

pub async fn get_connection(app_state: AppState, id: Uuid) -> impl IntoResponse {
    let connection = execute_get_connection(&app_state, id).await;
    match connection {
        Ok(connection) => {
            let response_dto = dto::ConnectionDto::from(connection);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        }
        Err(GetDatabaseConnectionError::ConnectionNotFound) => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Connection not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn execute_get_connection(
    app_state: &AppState,
    id: Uuid,
) -> Result<entity::connection::Model, GetDatabaseConnectionError> {
    let mut connection = repository::get_connection(&app_state.database_connection, id)
        .await
        .map_err(|e| match e {
            sea_orm::DbErr::RecordNotFound(_) => GetDatabaseConnectionError::ConnectionNotFound,
            other => GetDatabaseConnectionError::Connection(other),
        })?;
    cryptography_utils::decrypt_connection(&mut connection, &app_state.config.encryption_key)
        .map_err(GetDatabaseConnectionError::Decryption)?;
    Ok(connection)
}

pub async fn execute_get_all_connections(
    app_state: &AppState,
) -> Result<Vec<entity::connection::Model>, GetDatabaseConnectionError> {
    let mut connections = repository::get_all_connections(&app_state.database_connection)
        .await
        .map_err(GetDatabaseConnectionError::Connection)?;
    for connection in connections.iter_mut() {
        cryptography_utils::decrypt_connection(connection, &app_state.config.encryption_key)
            .map_err(GetDatabaseConnectionError::Decryption)?;
    }
    Ok(connections)
}

pub async fn get_all_connections(app_state: AppState) -> impl IntoResponse {
    let connections = execute_get_all_connections(&app_state).await;
    match connections {
        Ok(connections) => {
            let response_dto_list: Vec<dto::ConnectionDto> = connections
                .into_iter()
                .map(dto::ConnectionDto::from)
                .collect();
            (
                StatusCode::OK,
                axum::Json(serde_json::json!(response_dto_list)),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
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
    payload: dto::CreateConnectionDto,
) -> Result<entity::connection::Model, CreateConnectionError> {
    let mut connection_entity = entity::connection::Model {
        id: Uuid::new_v4(),
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
        repository::save_connection(&app_state.database_connection, connection_entity)
            .await
            .map_err(CreateConnectionError::Repository)?;

    cryptography_utils::decrypt_connection(&mut connection, &app_state.config.encryption_key)
        .map_err(CreateConnectionError::Decryption)?;

    Ok(connection)
}

pub async fn create_connection(
    app_state: &AppState,
    payload: dto::CreateConnectionDto,
) -> impl IntoResponse + use<> {
    match execute_create_connection(app_state, payload).await {
        Ok(connection) => {
            let response_dto = dto::ConnectionDto::from(connection);
            (
                StatusCode::CREATED,
                axum::Json(serde_json::json!(response_dto)),
            )
        }
        Err(CreateConnectionError::ConnectionStatus(e)) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "message": e.to_string() })),
        ),
        Err(CreateConnectionError::Encryption) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "message": "Failed to encrypt connection" })),
        ),
        Err(CreateConnectionError::Decryption(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "message": e.to_string() })),
        ),
        Err(CreateConnectionError::Repository(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

async fn execute_update_connection(
    app_state: &AppState,
    id: Uuid,
    payload: dto::UpdateConnectionDto,
) -> Result<entity::connection::Model, UpdateConnectionError> {
    let stored_connection = execute_get_connection(app_state, id).await?;
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
        ..stored_connection
    };

    let connection_status = get_connection_status(&connection_entity).await?;
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
    id: Uuid,
    payload: dto::UpdateConnectionDto,
) -> impl IntoResponse {
    match execute_update_connection(&app_state, id, payload).await {
        Ok(connection) => {
            let response_dto = dto::ConnectionDto::from(connection);
            info!("Connection updated: {:?}", response_dto);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        }
        Err(UpdateConnectionError::GetConnection(
            GetDatabaseConnectionError::ConnectionNotFound,
        )) => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Connection not found" })),
        ),
        Err(UpdateConnectionError::ConnectionStatus(e)) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn execute_query(
    app_state: &AppState,
    id: Uuid,
    payload: dto::ExecuteQueryDto,
) -> Result<QueryResult, ExecuteSqlError> {
    let connection = execute_get_connection(app_state, id).await?;
    database_service::execute_sql_query(app_state, &connection, &payload.query).await
}
