use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::entity::suggested_question;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

pub async fn find_for_connection(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    connection_id: Uuid,
) -> Result<Vec<suggested_question::Model>, DbErr> {
    suggested_question::Entity::find()
        .filter(suggested_question::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(suggested_question::Column::ConnectionId.eq(connection_id))
        .order_by(suggested_question::Column::GeneratedAt, sea_orm::Order::Asc)
        .all(db)
        .await
}

pub async fn latest_generated_at(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    connection_id: Uuid,
) -> Result<Option<sea_orm::prelude::DateTimeWithTimeZone>, DbErr> {
    let row = suggested_question::Entity::find()
        .filter(suggested_question::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(suggested_question::Column::ConnectionId.eq(connection_id))
        .order_by(
            suggested_question::Column::GeneratedAt,
            sea_orm::Order::Desc,
        )
        .one(db)
        .await?;
    Ok(row.map(|m| m.generated_at))
}

pub async fn delete_for_connection(
    db: &impl ConnectionTrait,
    tenant_id: &TenantId,
    connection_id: Uuid,
) -> Result<(), DbErr> {
    suggested_question::Entity::delete_many()
        .filter(suggested_question::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(suggested_question::Column::ConnectionId.eq(connection_id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn insert(
    db: &impl ConnectionTrait,
    tenant_id: &TenantId,
    connection_id: Uuid,
    question_text: String,
    now: sea_orm::prelude::DateTimeWithTimeZone,
    actor: &Actor,
) -> Result<(), DbErr> {
    suggested_question::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id.get_uuid()),
        connection_id: Set(connection_id),
        question_text: Set(question_text),
        generated_at: Set(now),
        created_by_user_id: Set(actor.user_id()),
        created_by_actor: Set(actor.kind_name()),
        modified_by_user_id: Set(actor.user_id()),
        modified_by_actor: Set(actor.kind_name()),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;
    Ok(())
}
