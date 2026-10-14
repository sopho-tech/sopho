use super::dto;
use crate::authorization::dto::TenantId;
use crate::common::time_utils;
use crate::entity::conversation;
use sea_orm::sea_query::{Expr, Func};
use sea_orm::Condition;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

pub async fn save_conversation_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    conversation: conversation::Model,
) -> Result<conversation::Model, DbErr> {
    let mut conversation_active_model: conversation::ActiveModel = conversation.into();
    conversation_active_model.tenant_id = Set(tenant_id.get_uuid());
    let conversation_active_model = conversation_active_model.insert(txn).await?;
    Ok(conversation_active_model)
}

pub async fn get_conversation(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<Option<conversation::Model>, DbErr> {
    conversation::Entity::find()
        .filter(conversation::Column::Id.eq(id))
        .filter(conversation::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(db)
        .await
}

fn escape_like_pattern(term: &str) -> String {
    term.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

pub async fn list_conversations(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    reachable: Condition,
    search: Option<&str>,
    page: u64,
    page_size: u64,
) -> Result<(Vec<conversation::Model>, u64), DbErr> {
    let mut query = conversation::Entity::find()
        .filter(conversation::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(reachable);
    if let Some(term) = search {
        let pattern = format!("%{}%", escape_like_pattern(&term.to_lowercase()));
        query = query
            .filter(Expr::expr(Func::lower(Expr::col(conversation::Column::Name))).like(pattern));
    }
    let paginator = query
        .order_by(conversation::Column::UpdatedAt, sea_orm::Order::Desc)
        .paginate(db, page_size);
    let total = paginator.num_items().await?;
    let rows = paginator.fetch_page(page).await?;
    Ok((rows, total))
}

pub async fn update_conversation(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    id: Uuid,
    payload: dto::ConversationDto,
) -> Result<Option<conversation::Model>, DbErr> {
    let Some(existing) = get_conversation(db, tenant_id, id).await? else {
        return Ok(None);
    };
    let mut active: conversation::ActiveModel = existing.into();
    active.name = Set(payload.name);
    active.status = Set(payload.status.to_string());
    active.updated_at = Set(time_utils::now_utc_into());
    let result = active.update(db).await?;
    Ok(Some(result))
}

pub async fn delete_conversation_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    conversation_id: Uuid,
) -> Result<(), DbErr> {
    conversation::Entity::delete_many()
        .filter(conversation::Column::Id.eq(conversation_id))
        .filter(conversation::Column::TenantId.eq(tenant_id.get_uuid()))
        .exec(txn)
        .await?;
    Ok(())
}

pub async fn delete_conversations_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    conversation_ids: &[Uuid],
) -> Result<(), DbErr> {
    conversation::Entity::delete_many()
        .filter(conversation::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(conversation::Column::Id.is_in(conversation_ids.to_vec()))
        .exec(txn)
        .await?;
    Ok(())
}

pub async fn count_conversations_by_ids(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    conversation_ids: &[Uuid],
) -> Result<u64, DbErr> {
    conversation::Entity::find()
        .filter(conversation::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(conversation::Column::Id.is_in(conversation_ids.to_vec()))
        .count(db)
        .await
}

pub async fn update_conversation_visibility(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    id: Uuid,
    visibility: &str,
) -> Result<u64, DbErr> {
    let result = conversation::Entity::update_many()
        .col_expr(
            conversation::Column::Visibility,
            sea_orm::sea_query::Expr::value(visibility.to_string()),
        )
        .col_expr(
            conversation::Column::UpdatedAt,
            sea_orm::sea_query::Expr::value(time_utils::now_utc_into::<
                sea_orm::prelude::DateTimeWithTimeZone,
            >()),
        )
        .filter(conversation::Column::Id.eq(id))
        .filter(conversation::Column::TenantId.eq(tenant_id.get_uuid()))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}
