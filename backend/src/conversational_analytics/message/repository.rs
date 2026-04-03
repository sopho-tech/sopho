use crate::entity::conversation_message;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

pub async fn save_message_transaction(
    txn: &DatabaseTransaction,
    message: conversation_message::Model,
) -> Result<conversation_message::Model, DbErr> {
    let active: conversation_message::ActiveModel = message.into();
    let inserted = active.insert(txn).await?;
    Ok(inserted)
}

pub async fn save_message_connection(
    db: &DatabaseConnection,
    message: conversation_message::Model,
) -> Result<conversation_message::Model, DbErr> {
    let active: conversation_message::ActiveModel = message.into();
    let inserted = active.insert(db).await?;
    Ok(inserted)
}

pub async fn get_message(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<conversation_message::Model, DbErr> {
    match conversation_message::Entity::find_by_id(id).one(db).await? {
        Some(m) => Ok(m),
        None => Err(DbErr::RecordNotFound(
            "Conversation message not found".into(),
        )),
    }
}

pub async fn get_last_message_by_conversation_id(
    db: &DatabaseConnection,
    conversation_id: Uuid,
) -> Result<Option<conversation_message::Model>, DbErr> {
    conversation_message::Entity::find()
        .filter(conversation_message::Column::ConversationId.eq(conversation_id))
        .order_by_desc(conversation_message::Column::SequenceNumber)
        .one(db)
        .await
}

pub async fn get_first_message_by_conversation_id(
    db: &DatabaseConnection,
    conversation_id: Uuid,
) -> Result<Option<conversation_message::Model>, DbErr> {
    conversation_message::Entity::find()
        .filter(conversation_message::Column::ConversationId.eq(conversation_id))
        .order_by_asc(conversation_message::Column::SequenceNumber)
        .one(db)
        .await
}

pub async fn list_messages_by_conversation_id(
    db: &DatabaseConnection,
    conversation_id: Uuid,
) -> Result<Vec<conversation_message::Model>, DbErr> {
    conversation_message::Entity::find()
        .filter(conversation_message::Column::ConversationId.eq(conversation_id))
        .order_by_asc(conversation_message::Column::SequenceNumber)
        .all(db)
        .await
}

pub async fn update_message_status(
    db: &DatabaseConnection,
    message_id: Uuid,
    status: String,
) -> Result<conversation_message::Model, DbErr> {
    let message = get_message(db, message_id).await?;
    let mut active: conversation_message::ActiveModel = message.into();
    active.status = Set(status);
    active.updated_at = Set(crate::common::time_utils::now_utc_into());
    active.update(db).await
}

pub async fn delete_messages_for_conversation_transaction(
    txn: &DatabaseTransaction,
    conversation_id: Uuid,
) -> Result<(), DbErr> {
    conversation_message::Entity::delete_many()
        .filter(conversation_message::Column::ConversationId.eq(conversation_id))
        .exec(txn)
        .await?;
    Ok(())
}
