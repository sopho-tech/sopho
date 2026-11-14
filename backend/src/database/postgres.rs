use crate::common::errors::{ConnectionSettingsError, ExecuteQueryError};
use crate::data_catalog::dto::{Column as CatalogColumn, Database, Schema, Table};
use crate::database::constants::QueryResult;
use crate::database::constants::{
    CATALOG_SAMPLE_COLUMNS_PER_QUERY, CATALOG_SAMPLE_VALUES_PER_COLUMN,
    CATALOG_TABLE_FETCH_CONCURRENCY,
};
use crate::{connection::constants::ConnectionStatus, entity};
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use futures_util::StreamExt;
use rust_decimal::Decimal;
use sqlx::Column;
use sqlx::ConnectOptions;
use sqlx::Connection;
use sqlx::Row;
use sqlx::postgres::PgConnectOptions;
use uuid::Uuid;

type TableKey = (String, String);
type ColumnDefinition = (String, String);
type TableSamplingInput = (TableKey, Vec<ColumnDefinition>);
type TableSampleValues = (TableKey, Vec<Vec<String>>);

fn quote_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

pub fn connect_options(
    connection: &entity::connection::Model,
) -> Result<PgConnectOptions, ConnectionSettingsError> {
    let host = connection
        .host
        .as_deref()
        .filter(|host| !host.is_empty())
        .ok_or(ConnectionSettingsError::MissingHost)?;
    let port = connection
        .port
        .as_deref()
        .filter(|port| !port.is_empty())
        .ok_or(ConnectionSettingsError::MissingPort)?;
    let port_number = port
        .parse::<u16>()
        .ok()
        .filter(|port_number| *port_number != 0)
        .ok_or_else(|| ConnectionSettingsError::InvalidPort {
            port: port.to_string(),
        })?;
    Ok(PgConnectOptions::new_without_pgpass()
        .disable_statement_logging()
        .host(host)
        .port(port_number)
        .username(connection.username.as_deref().unwrap_or_default())
        .password(connection.password.as_deref().unwrap_or_default())
        .database(&connection.database))
}

pub async fn get_connection_status(connect_options: &PgConnectOptions) -> ConnectionStatus {
    let sqlx_connection_result = sqlx::postgres::PgConnection::connect_with(connect_options).await;

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

async fn fetch_table_sample_values(
    postgres_pool: &sqlx::PgPool,
    schema_name: &str,
    table_name: &str,
    columns: &[ColumnDefinition],
) -> Result<Vec<Vec<String>>, sqlx::Error> {
    let quoted_schema = quote_ident(schema_name);
    let quoted_table = quote_ident(table_name);
    let mut sample_values_per_column: Vec<Vec<String>> = Vec::with_capacity(columns.len());

    for column_chunk in columns.chunks(CATALOG_SAMPLE_COLUMNS_PER_QUERY) {
        let projections: Vec<String> = column_chunk
            .iter()
            .enumerate()
            .map(|(chunk_index, (column_name, _))| {
                let quoted_column = quote_ident(column_name);
                format!(
                    "(SELECT array_agg(sampled_value) FROM (SELECT DISTINCT {column}::text AS sampled_value FROM {schema}.{table} WHERE {column} IS NOT NULL LIMIT {limit}) sample_{index}) AS sample_column_{index}",
                    column = quoted_column,
                    schema = quoted_schema,
                    table = quoted_table,
                    limit = CATALOG_SAMPLE_VALUES_PER_COLUMN,
                    index = chunk_index
                )
            })
            .collect();
        let sample_query = format!("SELECT {}", projections.join(", "));
        let sample_row = sqlx::query(&sample_query).fetch_one(postgres_pool).await?;
        for chunk_index in 0..column_chunk.len() {
            let column_alias = format!("sample_column_{}", chunk_index);
            let values: Option<Vec<String>> = sample_row.try_get(column_alias.as_str())?;
            sample_values_per_column.push(values.unwrap_or_default());
        }
    }

    Ok(sample_values_per_column)
}

pub async fn get_data_catalog(
    postgres_pool: &sqlx::PgPool,
    connection: &entity::connection::Model,
) -> Result<Database, sqlx::Error> {
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
    .fetch_all(postgres_pool)
    .await?;

    let mut columns_by_table: std::collections::HashMap<TableKey, Vec<ColumnDefinition>> =
        std::collections::HashMap::new();
    let mut table_order: Vec<TableKey> = Vec::new();
    for metadata_row in metadata_rows {
        let schema_name: String = metadata_row.try_get("schema_name")?;
        let table_name: String = metadata_row.try_get("table_name")?;
        let column_name: String = metadata_row.try_get("column_name")?;
        let data_type: String = metadata_row.try_get("data_type")?;
        let table_key = (schema_name, table_name);
        if !columns_by_table.contains_key(&table_key) {
            table_order.push(table_key.clone());
        }
        columns_by_table
            .entry(table_key)
            .or_default()
            .push((column_name, data_type));
    }

    let sampling_inputs: Vec<TableSamplingInput> = table_order
        .iter()
        .map(|table_key| {
            let columns = columns_by_table.get(table_key).cloned().unwrap_or_default();
            (table_key.clone(), columns)
        })
        .collect();

    let sampled_tables: Vec<Result<TableSampleValues, sqlx::Error>> =
        futures_util::stream::iter(sampling_inputs.into_iter().map(
            |(table_key, columns)| async move {
                let sample_values =
                    fetch_table_sample_values(postgres_pool, &table_key.0, &table_key.1, &columns)
                        .await?;
                Ok((table_key, sample_values))
            },
        ))
        .buffered(CATALOG_TABLE_FETCH_CONCURRENCY)
        .collect()
        .await;

    let mut sample_values_by_table: std::collections::HashMap<TableKey, Vec<Vec<String>>> =
        std::collections::HashMap::new();
    for sampled_table in sampled_tables {
        let (table_key, sample_values) = sampled_table?;
        sample_values_by_table.insert(table_key, sample_values);
    }

    let mut schema_map: std::collections::HashMap<String, Schema> =
        std::collections::HashMap::new();
    for table_key in table_order {
        let (schema_name, table_name) = table_key.clone();
        let columns = columns_by_table
            .get(&table_key)
            .cloned()
            .unwrap_or_default();
        let sample_values = sample_values_by_table
            .remove(&table_key)
            .unwrap_or_default();

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
        for (column_index, (column_name, data_type)) in columns.into_iter().enumerate() {
            let column_sample_values = sample_values.get(column_index).cloned().unwrap_or_default();
            table.columns.insert(
                column_name.clone(),
                CatalogColumn::with_samples(
                    column_name,
                    "",
                    column_sample_values,
                    data_type,
                    false,
                    false,
                ),
            );
        }
    }

    Ok(Database::new(
        database_name,
        connection.description.as_deref().unwrap_or(""),
        schema_map.into_values().collect::<Vec<_>>(),
    ))
}

#[cfg(test)]
mod tests {
    use super::connect_options;
    use crate::common::errors::ConnectionSettingsError;
    use crate::entity;
    use uuid::Uuid;

    fn postgres_connection(
        host: Option<&str>,
        port: Option<&str>,
        password: &str,
        database: &str,
    ) -> entity::connection::Model {
        let created_at = chrono::Utc::now().fixed_offset();
        entity::connection::Model {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            owner_user_id: Uuid::new_v4(),
            visibility: "TENANT".to_string(),
            name: "Warehouse".to_string(),
            description: None,
            status: "ACTIVE".to_string(),
            database: database.to_string(),
            host: host.map(str::to_string),
            password: Some(password.to_string()),
            port: port.map(str::to_string),
            schema: None,
            username: Some("analyst".to_string()),
            source_type: "POSTGRESQL".to_string(),
            created_by_user_id: None,
            created_by_actor: "USER".to_string(),
            modified_by_user_id: None,
            modified_by_actor: "USER".to_string(),
            created_at,
            updated_at: created_at,
        }
    }

    #[test]
    fn a_password_or_database_carrying_another_host_does_not_change_the_dialled_host() {
        for crafted_value in [
            "secret@10.0.0.5:5432/postgres#",
            "secret@169.254.169.254/",
            "x?host=10.0.0.5",
            "x/../y@127.0.0.1",
        ] {
            let with_crafted_password = connect_options(&postgres_connection(
                Some("db.example.com"),
                Some("6543"),
                crafted_value,
                "sales",
            ))
            .unwrap();
            assert_eq!(with_crafted_password.get_host(), "db.example.com");
            assert_eq!(with_crafted_password.get_port(), 6543);
            assert_eq!(with_crafted_password.get_database(), Some("sales"));

            let with_crafted_database = connect_options(&postgres_connection(
                Some("db.example.com"),
                Some("6543"),
                "secret",
                crafted_value,
            ))
            .unwrap();
            assert_eq!(with_crafted_database.get_host(), "db.example.com");
            assert_eq!(with_crafted_database.get_port(), 6543);
            assert_eq!(with_crafted_database.get_database(), Some(crafted_value));
        }
    }

    #[test]
    fn a_missing_host_or_an_invalid_port_is_refused() {
        assert!(matches!(
            connect_options(&postgres_connection(None, Some("5432"), "secret", "sales")),
            Err(ConnectionSettingsError::MissingHost)
        ));
        assert!(matches!(
            connect_options(&postgres_connection(
                Some(""),
                Some("5432"),
                "secret",
                "sales"
            )),
            Err(ConnectionSettingsError::MissingHost)
        ));
        assert!(matches!(
            connect_options(&postgres_connection(
                Some("db.example.com"),
                None,
                "secret",
                "sales"
            )),
            Err(ConnectionSettingsError::MissingPort)
        ));
        for invalid_port in ["0", "70000", "54a2", "-1"] {
            assert!(matches!(
                connect_options(&postgres_connection(
                    Some("db.example.com"),
                    Some(invalid_port),
                    "secret",
                    "sales"
                )),
                Err(ConnectionSettingsError::InvalidPort { .. })
            ));
        }
    }
}
