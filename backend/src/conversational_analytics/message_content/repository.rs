use crate::entity::conversation_message;
use crate::entity::conversation_message_content;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, QueryTrait,
};
use uuid::Uuid;

pub async fn save_message_content_transaction(
    txn: &DatabaseTransaction,
    row: conversation_message_content::Model,
) -> Result<conversation_message_content::Model, DbErr> {
    let active: conversation_message_content::ActiveModel = row.into();
    let inserted = active.insert(txn).await?;
    Ok(inserted)
}

pub async fn save_message_content_connection(
    db: &DatabaseConnection,
    row: conversation_message_content::Model,
) -> Result<conversation_message_content::Model, DbErr> {
    let active: conversation_message_content::ActiveModel = row.into();
    let inserted = active.insert(db).await?;
    Ok(inserted)
}

pub async fn get_message_content(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<conversation_message_content::Model, DbErr> {
    match conversation_message_content::Entity::find_by_id(id)
        .one(db)
        .await?
    {
        Some(m) => Ok(m),
        None => Err(DbErr::RecordNotFound(
            "Conversation message content not found".into(),
        )),
    }
}

pub async fn get_first_message_content_by_conversation_message_id(
    db: &DatabaseConnection,
    conversation_message_id: Uuid,
) -> Result<Option<conversation_message_content::Model>, DbErr> {
    conversation_message_content::Entity::find()
        .filter(
            conversation_message_content::Column::ConversationMessageId.eq(conversation_message_id),
        )
        .order_by_asc(conversation_message_content::Column::SequenceNumber)
        .one(db)
        .await
}

pub async fn list_message_content_by_conversation_message_id(
    db: &DatabaseConnection,
    conversation_message_id: Uuid,
) -> Result<Vec<conversation_message_content::Model>, DbErr> {
    conversation_message_content::Entity::find()
        .filter(
            conversation_message_content::Column::ConversationMessageId.eq(conversation_message_id),
        )
        .order_by_asc(conversation_message_content::Column::SequenceNumber)
        .all(db)
        .await
}

pub async fn list_message_content_by_conversation_message_ids(
    db: &DatabaseConnection,
    conversation_message_ids: &[Uuid],
) -> Result<Vec<conversation_message_content::Model>, DbErr> {
    conversation_message_content::Entity::find()
        .filter(
            conversation_message_content::Column::ConversationMessageId
                .is_in(conversation_message_ids.to_vec()),
        )
        .order_by_asc(conversation_message_content::Column::ConversationMessageId)
        .order_by_asc(conversation_message_content::Column::SequenceNumber)
        .all(db)
        .await
}

pub async fn delete_message_contents_for_conversation_transaction(
    txn: &DatabaseTransaction,
    conversation_id: Uuid,
) -> Result<(), DbErr> {
    let message_id_subquery = conversation_message::Entity::find()
        .filter(conversation_message::Column::ConversationId.eq(conversation_id))
        .select_only()
        .column(conversation_message::Column::Id)
        .into_query();
    conversation_message_content::Entity::delete_many()
        .filter(
            conversation_message_content::Column::ConversationMessageId
                .in_subquery(message_id_subquery),
        )
        .exec(txn)
        .await?;
    Ok(())
}
