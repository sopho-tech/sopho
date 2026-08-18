use crate::common::errors::GetDataCatalogError;
use crate::common::AppState;
use crate::data_catalog::dto::Database;
use crate::database::pool::{get_database_pool, DatabasePool};
use crate::database::{postgres, sqlite};
use crate::entity;

pub async fn get_data_catalog_of_connection(
    app_state: &AppState,
    connection: &entity::connection::Model,
) -> Result<Database, GetDataCatalogError> {
    let database_pool = get_database_pool(&app_state.database_pools, connection).await?;
    match database_pool {
        DatabasePool::Postgres(postgres_pool) => {
            postgres::get_data_catalog(&postgres_pool, connection)
                .await
                .map_err(Into::into)
        }
        DatabasePool::Sqlite(sqlite_pool) => sqlite::get_data_catalog(&sqlite_pool, connection)
            .await
            .map_err(Into::into),
    }
}
