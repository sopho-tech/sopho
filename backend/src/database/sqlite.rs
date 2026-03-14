use crate::common::errors::{ExecuteQueryError, ValidationError};
use crate::data_catalog::dto::{Column as CatalogColumn, Database, Schema, Table};
use crate::database::constants::DatabaseConnection;
use crate::database::constants::QueryResult;
use crate::{connection::constants::ConnectionStatus, entity};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use sqlx::Column;
use sqlx::Connection;
use sqlx::Error;
use sqlx::Row;
use sqlx::TypeInfo;
use sqlx::ValueRef;
use uuid::Uuid;

fn quote_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn get_database_url(connection_entity: &entity::connection::Model) -> String {
    format!("sqlite:///{}?mode=rwc", connection_entity.database)
}

pub fn validate_sqlite_path(path: &str) -> Result<(), ValidationError> {
    if path.trim().is_empty() {
        return Err(ValidationError("Database path cannot be empty".to_string()));
    }
    if path.contains("..") {
        return Err(ValidationError(
            "Database path cannot contain path traversal (..)".to_string(),
        ));
    }
    let path = path.trim();
    let is_absolute = path.starts_with('/');
    let is_relative = path.starts_with("./");
    if !is_absolute && !is_relative {
        return Err(ValidationError(format!(
            "Database path must be absolute (/path/to/db.sqlite) or relative (./path/to/db.sqlite), got: {}",
            path
        )));
    }
    if !path.ends_with(".sqlite") && !path.ends_with(".db") && !path.ends_with(".sqlite3") {
        return Err(ValidationError(format!(
            "Database path must end with .sqlite, .sqlite3, or .db extension, got: {}",
            path
        )));
    }
    Ok(())
}

pub async fn get_connection_status(
    connection_entity: &entity::connection::Model,
) -> Result<ConnectionStatus, ValidationError> {
    validate_sqlite_path(&connection_entity.database)?;
    let database_url = get_database_url(connection_entity);
    let sqlx_connection_result = sqlx::sqlite::SqliteConnection::connect(&database_url).await;

    match sqlx_connection_result {
        Ok(mut conn) => {
            let query_execution_result = sqlx::query("SELECT 1 as result")
                .fetch_optional(&mut conn)
                .await;
            Ok(match query_execution_result {
                Ok(Some(_row)) => ConnectionStatus::Active,
                Ok(None) => ConnectionStatus::Failed,
                Err(_e) => ConnectionStatus::Failed,
            })
        }
        Err(_e) => Ok(ConnectionStatus::Failed),
    }
}

pub async fn get_database_connection(
    connection: &entity::connection::Model,
) -> Result<DatabaseConnection, Error> {
    let database_url = get_database_url(&connection);
    let database_connection = sqlx::sqlite::SqliteConnection::connect(&database_url).await?;
    Ok(DatabaseConnection::Sqlite(database_connection))
}

pub async fn execute_query(
    conn: &mut sqlx::sqlite::SqliteConnection,
    query: &str,
) -> Result<QueryResult, ExecuteQueryError> {
    let rows = sqlx::query(query)
        .fetch_all(conn)
        .await
        .map_err(ExecuteQueryError::Database)?;

    for row in &rows {
        let mut raw: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        for col in row.columns() {
            let key = col.name().to_string();
            let val = row
                .try_get::<String, _>(col.ordinal())
                .map(serde_json::Value::String)
                .or_else(|_| {
                    row.try_get::<i64, _>(col.ordinal())
                        .map(|v| serde_json::json!(v))
                })
                .or_else(|_| {
                    row.try_get::<f64, _>(col.ordinal())
                        .map(|v| serde_json::json!(v))
                })
                .or_else(|_| {
                    row.try_get::<bool, _>(col.ordinal())
                        .map(|v| serde_json::json!(v))
                })
                .or_else(|_| {
                    row.try_get::<NaiveDate, _>(col.ordinal())
                        .map(|v| serde_json::json!(v))
                })
                .or_else(|_| {
                    row.try_get::<NaiveDateTime, _>(col.ordinal())
                        .map(|v| serde_json::json!(v))
                })
                .or_else(|_| {
                    row.try_get::<Uuid, _>(col.ordinal())
                        .map(|v| serde_json::json!(v))
                })
                .or_else(|_| {
                    row.try_get::<Vec<u8>, _>(col.ordinal())
                        .map(|v| serde_json::Value::String(STANDARD.encode(&v)))
                })
                .unwrap_or(serde_json::Value::Null);
            raw.insert(key, val);
        }
    }

    let mut columns: Vec<serde_json::Value> = Vec::new();
    if let Some(first_row) = rows.get(0) {
        columns = first_row
            .columns()
            .iter()
            .map(|col| {
                let data_type = first_row
                    .try_get_raw(col.ordinal())
                    .map(|val_ref| val_ref.type_info().name().to_string())
                    .unwrap_or_else(|_| col.type_info().to_string());
                serde_json::json!({
                    "column_name": col.name().to_string(),
                    "data_type": data_type
                })
            })
            .collect();
    }

    let mut json_rows: Vec<serde_json::Value> = Vec::new();
    for row in rows {
        let mut map = serde_json::Map::new();
        for (i, col) in row.columns().iter().enumerate() {
            let val_ref = row.try_get_raw(col.ordinal())?;
            let type_info = val_ref.type_info();
            let type_name = type_info.name();
            let value: Result<serde_json::Value, sqlx::Error> = match type_name {
                "INTEGER" | "INT" | "INT4" | "INT8" | "BIGINT" | "SMALLINT" | "TINYINT" => {
                    row.try_get::<i64, _>(i).map(|v| serde_json::json!(v))
                }
                "REAL" | "FLOAT" | "DOUBLE" => {
                    row.try_get::<f64, _>(i).map(|v| serde_json::json!(v))
                }
                "TEXT" | "VARCHAR" | "CHAR" | "CLOB" | "STRING" => row
                    .try_get::<Uuid, _>(i)
                    .map(|v| serde_json::json!(v))
                    .or_else(|_| row.try_get::<String, _>(i).map(serde_json::Value::String)),
                "BLOB" => row
                    .try_get::<Vec<u8>, _>(i)
                    .map(|v| serde_json::Value::String(STANDARD.encode(&v))),
                "BOOLEAN" | "BOOL" => {
                    let value = row.try_get::<bool, _>(i);
                    value.map(serde_json::Value::Bool)
                }
                "NUMERIC" | "DECIMAL" => row.try_get::<f64, _>(i).map(|v| {
                    serde_json::Number::from_f64(v)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null)
                }),
                "DATE" => row.try_get::<NaiveDate, _>(i).map(|v| serde_json::json!(v)),
                "DATETIME" | "TIMESTAMP" => row
                    .try_get::<NaiveDateTime, _>(i)
                    .map(|v| serde_json::json!(v)),
                "TIMESTAMPTZ" => row
                    .try_get::<DateTime<FixedOffset>, _>(i)
                    .map(|v| serde_json::json!(v)),
                "NULL" => Ok(serde_json::Value::Null),
                _ => {
                    if col
                        .type_info()
                        .to_string()
                        .to_uppercase()
                        .starts_with("TEXT")
                    {
                        let value = row.try_get::<String, _>(i);
                        value.map(serde_json::Value::String)
                    } else {
                        return Err(ExecuteQueryError::UnhandledDataType(
                            col.type_info().to_string(),
                        ));
                    }
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
        DatabaseConnection::Sqlite(conn) => conn,
        _ => unreachable!(),
    };

    let database_name = connection.name.clone();

    let table_rows = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(&mut conn)
    .await?;

    let mut tables: Vec<Table> = Vec::new();
    for table_row in table_rows {
        let table_name: String = table_row.try_get("name")?;
        let pragma_query = format!("PRAGMA table_info({})", quote_ident(&table_name));
        let column_rows = sqlx::query(&pragma_query).fetch_all(&mut conn).await?;

        let mut columns: std::collections::HashMap<String, CatalogColumn> =
            std::collections::HashMap::new();
        for column_row in column_rows {
            let column_name: String = column_row.try_get("name")?;
            let data_type: String = column_row
                .try_get::<Option<String>, _>("type")?
                .unwrap_or_default();

            let quoted_table = quote_ident(&table_name);
            let quoted_column = quote_ident(&column_name);
            let sample_query = format!(
                "SELECT DISTINCT CAST({column} AS TEXT) AS value FROM {table} WHERE {column} IS NOT NULL LIMIT 5",
                column = quoted_column,
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

            columns.insert(
                column_name.clone(),
                CatalogColumn::with_samples(
                    column_name,
                    "",
                    sample_values,
                    data_type,
                    false,
                    false,
                ),
            );
        }

        tables.push(Table {
            name: table_name,
            description: String::new(),
            columns,
            should_delete: false,
            should_select: false,
        });
    }

    Ok(Database::new(
        database_name,
        connection.description.as_deref().unwrap_or(""),
        [Schema::new("main", "", tables, false, false)],
    ))
}
