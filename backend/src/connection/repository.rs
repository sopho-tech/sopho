use sea_orm::{ActiveModelTrait, EntityTrait, DatabaseConnection, DbErr};
use crate::entity::connection;
use uuid::Uuid;
use sea_orm::Set;
use crate::connection::dto;
use chrono::Utc;

pub async fn save_connection(db: &DatabaseConnection, connection: connection::Model) -> Result<connection::Model, DbErr> {
    let connection_active_model: connection::ActiveModel = connection.into();
    let connection_active_model = connection_active_model.insert(db).await?;
    Ok(connection_active_model.into())
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

pub async fn update_connection(db: &DatabaseConnection, connection_id: Uuid, payload: dto::ConnectionDto) -> Result<connection::Model, DbErr> {
    let connection = get_connection(&db, connection_id).await;
    match connection {
        Ok(connection) => {
            let mut connection_entity: connection::ActiveModel = connection.into();
            connection_entity.name = Set(payload.name);
            connection_entity.description = Set(payload.description);
            connection_entity.status = Set(payload.status.to_string());
            connection_entity.database = Set(payload.database);
            connection_entity.host = Set(payload.host);
            connection_entity.password = Set(payload.password);
            connection_entity.port = Set(payload.port);
            connection_entity.schema = Set(payload.schema);
            connection_entity.username = Set(payload.username);
            connection_entity.source_type = Set(payload.source_type.to_string());
            connection_entity.updated_at = Set(Utc::now().into());
            connection_entity.created_at = Set(payload.created_at);

            let connection_entity = connection_entity.update(db).await;
            return connection_entity;
        }
        Err(e) => Err(e),
    }
}
