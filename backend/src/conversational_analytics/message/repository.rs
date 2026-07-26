use crate::entity::conversation_message;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    FromQueryResult, QueryFilter, QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

#[derive(Debug, FromQueryResult)]
struct ConversationMessageCountRow {
    conversation_id: Uuid,
    message_count: i64,
}

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
    limit: Option<u64>,
    descending: bool,
) -> Result<Vec<conversation_message::Model>, DbErr> {
    let mut query = conversation_message::Entity::find()
        .filter(conversation_message::Column::ConversationId.eq(conversation_id));
    query = if descending {
        query.order_by_desc(conversation_message::Column::SequenceNumber)
    } else {
        query.order_by_asc(conversation_message::Column::SequenceNumber)
    };
    if let Some(limit) = limit {
        query = query.limit(limit);
    }
    query.all(db).await
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

pub async fn count_messages_by_sender_for_conversations(
    db: &DatabaseConnection,
    conversation_ids: &[Uuid],
    sender: &str,
) -> Result<Vec<(Uuid, i64)>, DbErr> {
    let rows = conversation_message::Entity::find()
        .select_only()
        .column(conversation_message::Column::ConversationId)
        .column_as(conversation_message::Column::Id.count(), "message_count")
        .filter(conversation_message::Column::ConversationId.is_in(conversation_ids.to_vec()))
        .filter(conversation_message::Column::Sender.eq(sender))
        .group_by(conversation_message::Column::ConversationId)
        .into_model::<ConversationMessageCountRow>()
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| (row.conversation_id, row.message_count))
        .collect())
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

pub async fn delete_messages_for_conversations_transaction(
    txn: &DatabaseTransaction,
    conversation_ids: &[Uuid],
) -> Result<(), DbErr> {
    conversation_message::Entity::delete_many()
        .filter(conversation_message::Column::ConversationId.is_in(conversation_ids.to_vec()))
        .exec(txn)
        .await?;
    Ok(())
}
