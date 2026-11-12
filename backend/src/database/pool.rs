use crate::common::Configurations;
use crate::common::errors::GetDatabaseConnectionError;
use crate::connection::constants::SourceType;
use crate::database::constants::{POSTGRES_POOL_MAX_CONNECTIONS, SQLITE_POOL_MAX_CONNECTIONS};
use crate::database::{guard, postgres, sqlite};
use crate::entity;
use sha2::{Digest, Sha256};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{PgPool, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub enum DatabasePool {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DatabasePoolKey {
    connection_id: Uuid,
    credential_fingerprint: String,
}

pub type DatabasePoolRegistry = Arc<RwLock<HashMap<DatabasePoolKey, DatabasePool>>>;

enum DatabaseTarget {
    Postgres {
        connect_options: Box<PgConnectOptions>,
    },
    Sqlite {
        database_url: String,
    },
}

pub fn new_database_pool_registry() -> DatabasePoolRegistry {
    Arc::new(RwLock::new(HashMap::new()))
}

fn credential_fingerprint(connection: &entity::connection::Model) -> String {
    let mut hasher = Sha256::new();
    for credential_field in [
        Some(connection.source_type.as_str()),
        connection.username.as_deref(),
        connection.password.as_deref(),
        connection.host.as_deref(),
        connection.port.as_deref(),
        Some(connection.database.as_str()),
    ] {
        match credential_field {
            Some(field_value) => {
                hasher.update([1u8]);
                hasher.update((field_value.len() as u64).to_be_bytes());
                hasher.update(field_value.as_bytes());
            }
            None => hasher.update([0u8]),
        }
    }
    hex::encode(hasher.finalize())
}

fn resolve_database_target(
    connection: &entity::connection::Model,
    source_type: SourceType,
) -> Result<DatabaseTarget, GetDatabaseConnectionError> {
    match source_type {
        SourceType::Postgresql | SourceType::Supabase => Ok(DatabaseTarget::Postgres {
            connect_options: Box::new(postgres::connect_options(connection)?),
        }),
        SourceType::Sqlite => Ok(DatabaseTarget::Sqlite {
            database_url: sqlite::get_database_url(connection),
        }),
    }
}

async fn build_database_pool(
    configurations: &Configurations,
    database_target: DatabaseTarget,
) -> Result<DatabasePool, GetDatabaseConnectionError> {
    match database_target {
        DatabaseTarget::Postgres { connect_options } => {
            guard::ensure_host_is_permitted(
                configurations,
                connect_options.get_host(),
                connect_options.get_port(),
            )
            .await?;
            Ok(DatabasePool::Postgres(
                PgPoolOptions::new()
                    .max_connections(POSTGRES_POOL_MAX_CONNECTIONS)
                    .connect_with(*connect_options)
                    .await?,
            ))
        }
        DatabaseTarget::Sqlite { database_url } => Ok(DatabasePool::Sqlite(
            SqlitePoolOptions::new()
                .max_connections(SQLITE_POOL_MAX_CONNECTIONS)
                .connect(&database_url)
                .await?,
        )),
    }
}

pub async fn get_database_pool(
    configurations: &Configurations,
    registry: &DatabasePoolRegistry,
    connection: &entity::connection::Model,
) -> Result<DatabasePool, GetDatabaseConnectionError> {
    let source_type = SourceType::from_str(&connection.source_type)
        .map_err(GetDatabaseConnectionError::SourceTypeParse)?;
    guard::ensure_source_type_is_enabled(configurations, source_type)?;
    let database_target = resolve_database_target(connection, source_type)?;
    let pool_key = DatabasePoolKey {
        connection_id: connection.id,
        credential_fingerprint: credential_fingerprint(connection),
    };

    if let Some(existing_pool) = registry.read().await.get(&pool_key) {
        return Ok(existing_pool.clone());
    }

    let mut registry_guard = registry.write().await;
    if let Some(existing_pool) = registry_guard.get(&pool_key) {
        return Ok(existing_pool.clone());
    }

    let database_pool = build_database_pool(configurations, database_target).await?;
    registry_guard.retain(|existing_key, _| existing_key.connection_id != connection.id);
    registry_guard.insert(pool_key, database_pool.clone());
    Ok(database_pool)
}
