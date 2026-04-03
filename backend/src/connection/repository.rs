use crate::common::time_utils;
use crate::connection::dto;
use crate::entity::connection;
use sea_orm::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait};
use uuid::Uuid;

pub async fn save_connection(
    db: &DatabaseConnection,
    connection: connection::Model,
) -> Result<connection::Model, DbErr> {
    let connection_active_model: connection::ActiveModel = connection.into();
    let connection_active_model = connection_active_model.insert(db).await?;
    Ok(connection_active_model)
}

pub async fn get_connection(db: &DatabaseConnection, id: Uuid) -> Result<connection::Model, DbErr> {
    let connection = connection::Entity::find_by_id(id).one(db).await?;
    match connection {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Connection not found".into())),
    }
}

pub async fn get_all_connections(db: &DatabaseConnection) -> Result<Vec<connection::Model>, DbErr> {
    let connections = connection::Entity::find().all(db).await?;
    Ok(connections)
}

pub async fn delete_connection(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<sea_orm::DeleteResult, DbErr> {
    connection::Entity::delete_by_id(id).exec(db).await
}

pub async fn update_connection(
    db: &DatabaseConnection,
    connection_id: Uuid,
    payload: dto::ConnectionDto,
) -> Result<connection::Model, DbErr> {
    let connection = get_connection(db, connection_id).await?;
    let mut connection_entity: connection::ActiveModel = connection.into();
    connection_entity.name = Set(payload.name);
    connection_entity.description = Set(payload.description);
    connection_entity.status = Set(payload.status.to_string());
    connection_entity.database = Set(payload.database);
    connection_entity.host = Set(Some(payload.host));
    connection_entity.password = Set(Some(payload.password));
    connection_entity.port = Set(Some(payload.port));
    connection_entity.schema = Set(payload.schema);
    connection_entity.username = Set(Some(payload.username));
    connection_entity.source_type = Set(payload.source_type.to_string());
    connection_entity.updated_at = Set(time_utils::now_utc_into());
    connection_entity.created_at = Set(payload.created_at);
    connection_entity.update(db).await
}
