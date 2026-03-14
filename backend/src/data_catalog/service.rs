use crate::common::errors::GetDataCatalogError;
use crate::connection::constants::SourceType;
use crate::data_catalog::dto::Database;
use crate::database::{postgres, sqlite};
use crate::entity;

pub async fn get_data_catalog_of_connection(
    connection: &entity::connection::Model,
) -> Result<Database, GetDataCatalogError> {
    let source_type = SourceType::from_str(&connection.source_type)
        .map_err(GetDataCatalogError::SourceTypeParse)?;
    match source_type {
        SourceType::Postgresql | SourceType::Supabase => postgres::get_data_catalog(connection)
            .await
            .map_err(Into::into),
        SourceType::Sqlite => sqlite::get_data_catalog(connection)
            .await
            .map_err(Into::into),
        _ => Err(GetDataCatalogError::UnsupportedSourceType(
            connection.source_type.clone(),
        )),
    }
}
