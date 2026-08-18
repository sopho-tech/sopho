use crate::common::errors::GetDatabaseConnectionError;
use crate::connection::constants::SourceType;
use crate::database::constants::{POSTGRES_POOL_MAX_CONNECTIONS, SQLITE_POOL_MAX_CONNECTIONS};
use crate::database::{postgres, sqlite};
use crate::entity;
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPoolOptions;
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

pub fn new_database_pool_registry() -> DatabasePoolRegistry {
    Arc::new(RwLock::new(HashMap::new()))
}

fn credential_fingerprint(database_url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(database_url.as_bytes());
    hex::encode(hasher.finalize())
}

fn resolve_database_url(
    connection: &entity::connection::Model,
    source_type: SourceType,
) -> Result<String, GetDatabaseConnectionError> {
    match source_type {
        SourceType::Postgresql | SourceType::Supabase => Ok(postgres::get_database_url(connection)),
        SourceType::Sqlite => Ok(sqlite::get_database_url(connection)),
        _ => Err(GetDatabaseConnectionError::UnsupportedSourceType(
            connection.source_type.clone(),
        )),
    }
}

async fn build_database_pool(
    database_url: &str,
    source_type: SourceType,
    connection: &entity::connection::Model,
) -> Result<DatabasePool, GetDatabaseConnectionError> {
    match source_type {
        SourceType::Postgresql | SourceType::Supabase => Ok(DatabasePool::Postgres(
            PgPoolOptions::new()
                .max_connections(POSTGRES_POOL_MAX_CONNECTIONS)
                .connect(database_url)
                .await?,
        )),
        SourceType::Sqlite => Ok(DatabasePool::Sqlite(
            SqlitePoolOptions::new()
                .max_connections(SQLITE_POOL_MAX_CONNECTIONS)
                .connect(database_url)
                .await?,
        )),
        _ => Err(GetDatabaseConnectionError::UnsupportedSourceType(
            connection.source_type.clone(),
        )),
    }
}

pub async fn get_database_pool(
    registry: &DatabasePoolRegistry,
    connection: &entity::connection::Model,
) -> Result<DatabasePool, GetDatabaseConnectionError> {
    let source_type = SourceType::from_str(&connection.source_type)
        .map_err(GetDatabaseConnectionError::SourceTypeParse)?;
    let database_url = resolve_database_url(connection, source_type)?;
    let pool_key = DatabasePoolKey {
        connection_id: connection.id,
        credential_fingerprint: credential_fingerprint(&database_url),
    };

    if let Some(existing_pool) = registry.read().await.get(&pool_key) {
        return Ok(existing_pool.clone());
    }

    let mut registry_guard = registry.write().await;
    if let Some(existing_pool) = registry_guard.get(&pool_key) {
        return Ok(existing_pool.clone());
    }

    let database_pool = build_database_pool(&database_url, source_type, connection).await?;
    registry_guard.retain(|existing_key, _| existing_key.connection_id != connection.id);
    registry_guard.insert(pool_key, database_pool.clone());
    Ok(database_pool)
}
