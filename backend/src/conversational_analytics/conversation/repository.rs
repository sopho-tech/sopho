use super::dto;
use crate::common::time_utils;
use crate::entity::conversation;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, QueryOrder, Set,
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

pub async fn get_all_conversations(
    db: &DatabaseConnection,
) -> Result<Vec<conversation::Model>, DbErr> {
    let rows = conversation::Entity::find()
        .order_by(conversation::Column::UpdatedAt, sea_orm::Order::Desc)
        .all(db)
        .await?;
    Ok(rows)
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
