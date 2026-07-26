use super::dto;
use crate::common::time_utils;
use crate::entity::conversation;
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

pub async fn save_conversation_transaction(
    txn: &DatabaseTransaction,
    conversation: conversation::Model,
) -> Result<conversation::Model, DbErr> {
    let conversation_active_model: conversation::ActiveModel = conversation.into();
    let conversation_active_model = conversation_active_model.insert(txn).await?;
    Ok(conversation_active_model)
}

pub async fn get_conversation(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<conversation::Model, DbErr> {
    match conversation::Entity::find_by_id(id).one(db).await? {
        Some(m) => Ok(m),
        None => Err(DbErr::RecordNotFound("Conversation not found".into())),
    }
}

fn escape_like_pattern(term: &str) -> String {
    term.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

pub async fn list_conversations(
    db: &DatabaseConnection,
    search: Option<&str>,
    page: u64,
    page_size: u64,
) -> Result<(Vec<conversation::Model>, u64), DbErr> {
    let mut query = conversation::Entity::find();
    if let Some(term) = search {
        let pattern = format!("%{}%", escape_like_pattern(&term.to_lowercase()));
        query = query.filter(
            Expr::expr(Func::lower(Expr::col(conversation::Column::Name))).like(pattern),
        );
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
    id: Uuid,
    payload: dto::ConversationDto,
) -> Result<conversation::Model, DbErr> {
    let existing = get_conversation(db, id).await?;
    let mut active: conversation::ActiveModel = existing.into();
    active.name = Set(payload.name);
    active.status = Set(payload.status.to_string());
    active.updated_at = Set(time_utils::now_utc_into());
    let result = active.update(db).await?;
    Ok(result)
}

pub async fn delete_conversation_transaction(
    txn: &DatabaseTransaction,
    conversation_id: Uuid,
) -> Result<(), DbErr> {
    conversation::Entity::delete_by_id(conversation_id)
        .exec(txn)
        .await?;
    Ok(())
}

pub async fn delete_conversations_transaction(
    txn: &DatabaseTransaction,
    conversation_ids: &[Uuid],
) -> Result<(), DbErr> {
    conversation::Entity::delete_many()
        .filter(conversation::Column::Id.is_in(conversation_ids.to_vec()))
        .exec(txn)
        .await?;
    Ok(())
}

pub async fn count_conversations_by_ids(
    db: &DatabaseConnection,
    conversation_ids: &[Uuid],
) -> Result<u64, DbErr> {
    conversation::Entity::find()
        .filter(conversation::Column::Id.is_in(conversation_ids.to_vec()))
        .count(db)
        .await
}
