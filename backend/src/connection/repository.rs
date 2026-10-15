use crate::entity::connection;
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

pub async fn update_connection(
    db: &DatabaseConnection,
    connection: connection::Model,
) -> Result<connection::Model, DbErr> {
    let connection_active_model: connection::ActiveModel = connection.into();
    connection_active_model.reset_all().update(db).await
}
