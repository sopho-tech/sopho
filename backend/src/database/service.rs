use crate::common::AppState;
use crate::common::errors::{
    ConnectionGuardError, ExecuteSqlError, GetDatabaseConnectionError, ProbeConnectionError,
    ValidationError,
};
use crate::connection::constants::{ConnectionStatus, SourceType};
use crate::database::constants::QueryResult;
use crate::database::pool::{DatabasePool, get_database_pool};
use crate::database::{guard, postgres, sqlite};
use crate::entity;
use axum::http::StatusCode;
use futures_util::future::join_all;
use strum::VariantArray;

pub fn offered_source_types(app_state: &AppState) -> Vec<SourceType> {
    SourceType::VARIANTS
        .iter()
        .copied()
        .filter(|source_type| is_source_type_available(app_state, *source_type))
        .collect()
}

pub fn is_source_type_available(app_state: &AppState, source_type: SourceType) -> bool {
    guard::ensure_source_type_is_enabled(&app_state.config, source_type).is_ok()
}

pub fn ensure_source_type_is_enabled(
    app_state: &AppState,
    source_type: SourceType,
) -> Result<(), ConnectionGuardError> {
    guard::ensure_source_type_is_enabled(&app_state.config, source_type)
}

pub fn connection_guard_error_response(
    error: &ConnectionGuardError,
) -> (StatusCode, axum::Json<serde_json::Value>) {
    (
        StatusCode::FORBIDDEN,
        axum::Json(serde_json::json!({
            "error": error.to_string(),
            "error_code": error.error_code().as_str(),
        })),
    )
}

pub async fn open_database_pool(
    app_state: &AppState,
    connection: &entity::connection::Model,
) -> Result<DatabasePool, GetDatabaseConnectionError> {
    get_database_pool(&app_state.config, &app_state.database_pools, connection).await
}

pub async fn probe_connection_status(
    app_state: &AppState,
    connection: &entity::connection::Model,
) -> Result<ConnectionStatus, ProbeConnectionError> {
    let source_type = SourceType::from_str(&connection.source_type)
        .map_err(|message| ValidationError { message })?;
    guard::ensure_source_type_is_enabled(&app_state.config, source_type)?;
    match source_type {
        SourceType::Postgresql | SourceType::Supabase => {
            let Ok(connect_options) = postgres::connect_options(connection) else {
                return Ok(ConnectionStatus::Failed);
            };
            guard::ensure_host_is_permitted(
                &app_state.config,
                connect_options.get_host(),
                connect_options.get_port(),
            )
            .await?;
            Ok(postgres::get_connection_status(&connect_options).await)
        }
        SourceType::Sqlite => Ok(sqlite::get_connection_status(connection).await?),
    }
}

async fn execute_query_on_pool(
    database_pool: &DatabasePool,
    query: &str,
) -> Result<QueryResult, ExecuteSqlError> {
    match database_pool {
        DatabasePool::Postgres(postgres_pool) => {
            let mut pooled_connection = postgres_pool
                .acquire()
                .await
                .map_err(GetDatabaseConnectionError::from)?;
            postgres::execute_query(&mut pooled_connection, query)
                .await
                .map_err(Into::into)
        }
        DatabasePool::Sqlite(sqlite_pool) => {
            let mut pooled_connection = sqlite_pool
                .acquire()
                .await
                .map_err(GetDatabaseConnectionError::from)?;
            sqlite::execute_query(&mut pooled_connection, query)
                .await
                .map_err(Into::into)
        }
    }
}

pub async fn execute_sql_query(
    app_state: &AppState,
    connection: &entity::connection::Model,
    query: &str,
) -> Result<QueryResult, ExecuteSqlError> {
    let database_pool = open_database_pool(app_state, connection).await?;
    execute_query_on_pool(&database_pool, query).await
}

pub async fn execute_sql_queries_in_parallel(
    app_state: &AppState,
    connection: &entity::connection::Model,
    queries: &[&str],
) -> Vec<Result<QueryResult, ExecuteSqlError>> {
    let database_pool = match open_database_pool(app_state, connection).await {
        Ok(database_pool) => database_pool,
        Err(pool_error) => {
            let pool_error_message = pool_error.to_string();
            return queries
                .iter()
                .map(|_| {
                    Err(ExecuteSqlError::from(
                        GetDatabaseConnectionError::PoolUnavailable(pool_error_message.clone()),
                    ))
                })
                .collect();
        }
    };
    join_all(
        queries
            .iter()
            .map(|query| execute_query_on_pool(&database_pool, query)),
    )
    .await
}
