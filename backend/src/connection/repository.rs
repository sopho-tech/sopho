use crate::authorization::dto::TenantId;
use crate::common::time_utils;
use crate::connection::dto;
use crate::entity::connection;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::QueryFilter;
use sea_orm::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait};
use uuid::Uuid;

pub async fn save_connection(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    connection: connection::Model,
) -> Result<connection::Model, DbErr> {
    let mut connection_active_model: connection::ActiveModel = connection.into();
    connection_active_model.tenant_id = Set(tenant_id.get_uuid());
    let connection_active_model = connection_active_model.insert(db).await?;
    Ok(connection_active_model)
}

pub async fn get_connection(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<Option<connection::Model>, DbErr> {
    connection::Entity::find()
        .filter(connection::Column::Id.eq(id))
        .filter(connection::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(db)
        .await
}

pub async fn get_all_connections(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
) -> Result<Vec<connection::Model>, DbErr> {
    let connections = connection::Entity::find()
        .filter(connection::Column::TenantId.eq(tenant_id.get_uuid()))
        .all(db)
        .await?;
    Ok(connections)
}

pub async fn get_visible_connections(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    reachable: Condition,
) -> Result<Vec<connection::Model>, DbErr> {
    let connections = connection::Entity::find()
        .filter(connection::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(reachable)
        .all(db)
        .await?;
    Ok(connections)
}

pub async fn delete_connection(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<sea_orm::DeleteResult, DbErr> {
    connection::Entity::delete_many()
        .filter(connection::Column::Id.eq(id))
        .filter(connection::Column::TenantId.eq(tenant_id.get_uuid()))
        .exec(db)
        .await
}

pub async fn update_connection(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    connection_id: Uuid,
    payload: dto::ConnectionDto,
) -> Result<Option<connection::Model>, DbErr> {
    let Some(connection) = get_connection(db, tenant_id, connection_id).await? else {
        return Ok(None);
    };
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
    connection_entity.update(db).await.map(Some)
}
