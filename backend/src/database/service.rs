use crate::common::errors::{ExecuteQueryError, ExecuteSqlError, GetDatabaseConnectionError};
use crate::connection::constants::SourceType;
use crate::database::constants::{DatabaseConnection, QueryResult};
use crate::database::{postgres, sqlite};
use crate::entity;

pub async fn get_database_connection(
    connection: &entity::connection::Model,
) -> Result<DatabaseConnection, GetDatabaseConnectionError> {
    let database_connection = match SourceType::from_str(&connection.source_type).unwrap() {
        SourceType::Postgresql | SourceType::Supabase => {
            postgres::get_database_connection(connection).await?
        }
        SourceType::Sqlite => sqlite::get_database_connection(connection).await?,
        _ => panic!("Not implemented"),
    };
    Ok(database_connection)
}

pub async fn execute_query(
    database_connection: &mut DatabaseConnection,
    query: &str,
) -> Result<QueryResult, ExecuteQueryError> {
    match database_connection {
        DatabaseConnection::Postgres(conn) => postgres::execute_query(conn, query).await,
        DatabaseConnection::Sqlite(conn) => sqlite::execute_query(conn, query).await,
    }
}

pub async fn execute_sql_query(
    connection: &entity::connection::Model,
    query: &str,
) -> Result<QueryResult, ExecuteSqlError> {
    let mut database_connection = get_database_connection(connection).await?;
    execute_query(&mut database_connection, query)
        .await
        .map_err(Into::into)
}

pub async fn execute_sql_queries_in_parallel(
    connection: &entity::connection::Model,
    queries: &Vec<&str>,
) -> Vec<Result<QueryResult, ExecuteSqlError>> {
    let mut database_connection = match get_database_connection(connection).await {
        Ok(conn) => conn,
        Err(err) => return vec![Err(err.into())],
    };
    let mut results = Vec::with_capacity(queries.len());
    for query in queries {
        let result = execute_query(&mut database_connection, query)
            .await
            .map_err(Into::into);
        results.push(result);
    }
    results
}
