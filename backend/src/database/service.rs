use crate::common::errors::{ExecuteSqlError, GetDatabaseConnectionError};
use crate::common::AppState;
use crate::database::constants::QueryResult;
use crate::database::pool::{get_database_pool, DatabasePool};
use crate::database::{postgres, sqlite};
use crate::entity;
use futures_util::future::join_all;

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
    let database_pool = get_database_pool(&app_state.database_pools, connection).await?;
    execute_query_on_pool(&database_pool, query).await
}

pub async fn execute_sql_queries_in_parallel(
    app_state: &AppState,
    connection: &entity::connection::Model,
    queries: &[&str],
) -> Vec<Result<QueryResult, ExecuteSqlError>> {
    let database_pool = match get_database_pool(&app_state.database_pools, connection).await {
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
