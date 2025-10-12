use crate::connection::repository;
use crate::connection::dto;
use crate::entity;
use crate::connection::constants::ConnectionStatus;
use uuid::Uuid;
use axum::http::StatusCode;
use chrono::Utc;
use crate::common::AppState;
use axum::response::IntoResponse;
use sqlx::Connection;
use crate::connection::constants::SourceType;
use tracing::info;
use crate::common::database_utils;


pub async fn does_connection_exist(
    app_state: &AppState,
    id: Uuid,
) -> bool {
    repository::get_connection(&app_state.database_connection, id).await.is_ok()
}

pub async fn get_connection(
    app_state: AppState,
    id: Uuid,
) -> impl IntoResponse {
    let connection = execute_get_connection(app_state, id).await;
    match connection {
        Ok(connection) => {
            let response_dto = dto::ConnectionDto::from(connection);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        },
        Err(e) => {
            match e {
                sea_orm::DbErr::RecordNotFound(_) => {
                    (StatusCode::NOT_FOUND, axum::Json(serde_json::json!({ "error": "Connection not found" })))
                },
                _ => {
                    (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
                }
            }
        }
    }
}

pub async fn execute_get_connection(
    app_state: AppState,
    id: Uuid,
) -> Result<entity::connection::Model, sea_orm::DbErr> {
    return repository::get_connection(&app_state.database_connection, id).await;
}

pub async fn get_all_connections(
    app_state: AppState,
) -> impl IntoResponse {
    let connections = repository::get_all_connections(&app_state.database_connection).await;
    match connections {
        Ok(connections) => {
            let response_dto_list: Vec<dto::ConnectionDto> = connections.iter()
                .map(|connection| dto::ConnectionDto::from(connection.clone()))
                .collect();
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto_list)))
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

pub async fn create_connection(
    app_state: AppState,
    payload: dto::CreateConnectionDto,
) -> impl IntoResponse {
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
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };

    let database_url = database_utils::get_database_url(&connection_entity);
    let connection_status = get_connection_status(&connection_entity.source_type, &database_url).await;
    connection_entity.status = connection_status.to_string();

    match repository::save_connection(&app_state.database_connection, connection_entity).await {
        Ok(connection) => {
            let response_dto = dto::ConnectionDto::from(connection);
            info!("Connection created: {:?}", response_dto);
            (StatusCode::CREATED, axum::Json(serde_json::json!(response_dto)))
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}


pub async fn update_connection(
    app_state: AppState,
    id: Uuid,
    payload: dto::ConnectionDto,
) -> impl IntoResponse {
    let mut payload = payload;
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        payload.username,
        payload.password,
        payload.host,
        payload.port,
        payload.database
    );
    payload.status = get_connection_status(&payload.source_type.to_string(), &database_url).await;
    let connection = repository::update_connection(&app_state.database_connection, id, payload).await;
    match connection {
        Ok(connection) => {
            let response_dto = dto::ConnectionDto::from(connection);
            info!("Connection updated: {:?}", response_dto);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}


pub async fn get_connection_status(
    source_type: &str,
    database_url: &str,
) -> ConnectionStatus {
    let source_type = SourceType::from_str(&source_type).unwrap();
    match source_type {
        SourceType::Postgresql => get_connection_status_postgres(database_url).await,
        _ => panic!("Unsupported source type"),
    }
}

async fn get_connection_status_postgres(
    database_url: &str,
) -> ConnectionStatus {
    let sqlx_connection_result = sqlx::postgres::PgConnection::connect(database_url).await;

    match sqlx_connection_result {
        Ok(mut conn) => {
            let query_execution_result = sqlx::query!(
                "SELECT 1 as result"
            )
            .fetch_optional(&mut conn)
            .await;
            match query_execution_result {
                Ok(Some(_row)) => {
                    ConnectionStatus::Active
                }
                Ok(None) => {
                    ConnectionStatus::Failed
                }
                Err(_e) => {
                    ConnectionStatus::Failed
                }
            }
        }
        Err(_e) => {
            ConnectionStatus::Failed
        }
    }
}
