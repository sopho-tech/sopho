use crate::common::errors::ExecuteQueryError;
use crate::data_catalog::dto::{Column as CatalogColumn, Database, Schema, Table};
use crate::database::constants::{DatabaseConnection, QueryResult};
use crate::{connection::constants::ConnectionStatus, entity};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use sqlx::Column;
use sqlx::Connection;
use sqlx::Error;
use sqlx::Row;
use uuid::Uuid;

fn quote_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

pub fn get_database_url(payload: &entity::connection::Model) -> String {
    format!(
        "postgres://{}:{}@{}:{}/{}",
        payload.username.as_deref().unwrap_or(""),
        payload.password.as_deref().unwrap_or(""),
        payload.host.as_deref().unwrap_or(""),
        payload.port.as_deref().unwrap_or(""),
        payload.database
    )
}

pub async fn get_connection_status(
    connection_entity: &entity::connection::Model,
) -> ConnectionStatus {
    let database_url = get_database_url(connection_entity);
    let sqlx_connection_result = sqlx::postgres::PgConnection::connect(&database_url).await;

    match sqlx_connection_result {
        Ok(mut conn) => {
            let query_execution_result = sqlx::query("SELECT 1 as result")
                .fetch_optional(&mut conn)
                .await;
            match query_execution_result {
                Ok(Some(_row)) => ConnectionStatus::Active,
                Ok(None) => ConnectionStatus::Failed,
                Err(_e) => ConnectionStatus::Failed,
            }
        }
        Err(_e) => ConnectionStatus::Failed,
    }
}

pub async fn get_database_connection(
    connection: &entity::connection::Model,
) -> Result<DatabaseConnection, Error> {
    let database_url = get_database_url(connection);
    let conn = sqlx::postgres::PgConnection::connect(&database_url).await?;
    Ok(DatabaseConnection::Postgres(conn))
}

pub async fn execute_query(
    conn: &mut sqlx::postgres::PgConnection,
    query: &str,
) -> Result<QueryResult, ExecuteQueryError> {
    let rows = sqlx::query(query)
        .fetch_all(conn)
        .await
        .map_err(ExecuteQueryError::Database)?;

    let mut columns: Vec<serde_json::Value> = Vec::new();
    if let Some(first_row) = rows.first() {
        columns = first_row
            .columns()
            .iter()
            .map(|col| {
                serde_json::json!({
                    "column_name": col.name().to_string(),
                    "data_type": col.type_info().to_string()
                })
            })
            .collect();
    }

    let mut json_rows: Vec<serde_json::Value> = Vec::new();
    for row in rows {
        let mut map = serde_json::Map::new();
        for (i, col) in row.columns().iter().enumerate() {
            let value: Result<serde_json::Value, sqlx::Error> =
                match col.type_info().to_string().as_str() {
                    "BOOL" => {
                        let value = row.try_get::<bool, _>(i);
                        value.map(serde_json::Value::Bool)
                    }
                    "UUID" => row.try_get::<Uuid, _>(i).map(|v| serde_json::json!(v)),
                    "TEXT" => {
                        let value = row.try_get::<String, _>(i);
                        value.map(serde_json::Value::String)
                    }
                    "JSON" | "JSONB" => row
                        .try_get::<sqlx::types::Json<serde_json::Value>, _>(i)
                        .map(|j| {
                            serde_json::Value::String(
                                serde_json::to_string(&j.0).unwrap_or_default(),
                            )
                        }),
                    "VARCHAR" => {
                        let value = row.try_get::<String, _>(i);
                        value.map(serde_json::Value::String)
                    }
                    "DATE" => row.try_get::<NaiveDate, _>(i).map(|v| serde_json::json!(v)),
                    "TIMESTAMP" => row
                        .try_get::<NaiveDateTime, _>(i)
                        .map(|v| serde_json::json!(v)),
                    "TIMESTAMPTZ" => row
                        .try_get::<DateTime<FixedOffset>, _>(i)
                        .map(|v| serde_json::json!(v)),
                    "INT4" => row.try_get::<i32, _>(i).map(|v| serde_json::json!(v)),
                    "INT8" => row.try_get::<i64, _>(i).map(|v| serde_json::json!(v)),
                    "NUMERIC" => row.try_get::<Decimal, _>(i).map(|v| {
                        let f: f64 = v.try_into().unwrap_or(0.0);
                        serde_json::Number::from_f64(f)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null)
                    }),
                    _ => {
                        return Err(ExecuteQueryError::UnhandledDataType(
                            col.type_info().to_string(),
                        ));
                    }
                };
            match value {
                Ok(value) => {
                    map.insert(col.name().to_string(), value);
                }
                Err(err) => {
                    if let sqlx::Error::ColumnDecode {
                        index: _index,
                        source: _source,
                    } = &err
                    {
                        map.insert(col.name().to_string(), serde_json::Value::Null);
                    } else {
                        return Err(ExecuteQueryError::Database(err));
                    }
                }
            }
        }
        json_rows.push(serde_json::Value::Object(map));
    }
    Ok(QueryResult {
        columns,
        data: json_rows,
    })
}

pub async fn get_data_catalog(
    connection: &entity::connection::Model,
) -> Result<Database, sqlx::Error> {
    let mut conn = match get_database_connection(connection).await? {
        DatabaseConnection::Postgres(conn) => conn,
        _ => unreachable!(),
    };

    let database_name = connection.name.clone();

    let metadata_rows = sqlx::query(
        r#"
        SELECT
            t.table_schema AS schema_name,
            t.table_name AS table_name,
            c.column_name AS column_name,
            c.data_type AS data_type
        FROM information_schema.tables t
        JOIN information_schema.columns c
            ON t.table_schema = c.table_schema
            AND t.table_name = c.table_name
        WHERE t.table_type = 'BASE TABLE'
            AND t.table_schema NOT IN ('pg_catalog', 'information_schema')
        ORDER BY t.table_schema, t.table_name, c.ordinal_position
        "#,
    )
    .fetch_all(&mut conn)
    .await?;

    let mut schema_map: std::collections::HashMap<String, Schema> =
        std::collections::HashMap::new();

    for row in metadata_rows {
        let schema_name: String = row.try_get("schema_name")?;
        let table_name: String = row.try_get("table_name")?;
        let column_name: String = row.try_get("column_name")?;
        let data_type: String = row.try_get("data_type")?;

        let quoted_schema = quote_ident(&schema_name);
        let quoted_table = quote_ident(&table_name);
        let quoted_column = quote_ident(&column_name);
        let sample_query = format!(
            "SELECT DISTINCT {column}::text AS value FROM {schema}.{table} WHERE {column} IS NOT NULL LIMIT 5",
            column = quoted_column,
            schema = quoted_schema,
            table = quoted_table
        );
        let sample_rows = sqlx::query(&sample_query).fetch_all(&mut conn).await?;
        let sample_values = sample_rows
            .into_iter()
            .filter_map(|sample_row| {
                sample_row
                    .try_get::<Option<String>, _>("value")
                    .ok()
                    .flatten()
            })
            .collect::<Vec<_>>();

        let schema = schema_map.entry(schema_name.clone()).or_insert_with(|| {
            Schema::new(
                schema_name.clone(),
                "",
                std::iter::empty::<Table>(),
                false,
                false,
            )
        });
        let table = schema.tables.entry(table_name.clone()).or_insert_with(|| {
            Table::new(
                table_name.clone(),
                "",
                std::iter::empty::<(String, CatalogColumn)>(),
                false,
                false,
            )
        });
        table.columns.insert(
            column_name.clone(),
            CatalogColumn::with_samples(column_name, "", sample_values, data_type, false, false),
        );
    }

    Ok(Database::new(
        database_name,
        connection.description.as_deref().unwrap_or(""),
        schema_map.into_values().collect::<Vec<_>>(),
    ))
}
