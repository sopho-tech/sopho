use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::common::time_utils;
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
    created_by_user_id: Option<Uuid>,
) -> Result<Vec<connection::Model>, DbErr> {
    let mut query = connection::Entity::find()
        .filter(connection::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(reachable);
    if let Some(created_by_user_id) = created_by_user_id {
        query = query.filter(connection::Column::CreatedByUserId.eq(created_by_user_id));
    }
    query.all(db).await
}

pub async fn update_connection(
    db: &DatabaseConnection,
    connection: connection::Model,
) -> Result<connection::Model, DbErr> {
    let connection_active_model: connection::ActiveModel = connection.into();
    connection_active_model.reset_all().update(db).await
}

pub async fn update_connection_visibility(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    id: Uuid,
    visibility: &str,
    actor: &Actor,
) -> Result<u64, DbErr> {
    let result = connection::Entity::update_many()
        .col_expr(
            connection::Column::Visibility,
            sea_orm::sea_query::Expr::value(visibility.to_string()),
        )
        .col_expr(
            connection::Column::UpdatedAt,
            sea_orm::sea_query::Expr::value(time_utils::now_utc_into::<
                sea_orm::prelude::DateTimeWithTimeZone,
            >()),
        )
        .col_expr(
            connection::Column::ModifiedByUserId,
            sea_orm::sea_query::Expr::value(actor.user_id()),
        )
        .col_expr(
            connection::Column::ModifiedByActor,
            sea_orm::sea_query::Expr::value(actor.kind_name()),
        )
        .filter(connection::Column::Id.eq(id))
        .filter(connection::Column::TenantId.eq(tenant_id.get_uuid()))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}
