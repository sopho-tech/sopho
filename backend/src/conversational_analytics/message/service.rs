use crate::common::time_utils;
use crate::common::AppState;
use crate::conversational_analytics::conversation::error::ConversationError;
use crate::conversational_analytics::message::dto;
use crate::conversational_analytics::message::repository;
use crate::entity;
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use uuid::Uuid;

pub async fn create_message(
    txn: &DatabaseTransaction,
    payload: dto::CreateConversationMessageDto,
) -> Result<dto::ConversationMessageDto, ConversationError> {
    let now = time_utils::now_utc_into();
    let message = entity::conversation_message::Model {
        id: Uuid::new_v4(),
        conversation_id: payload.conversation_id,
        sequence_number: payload.sequence_number,
        sender: payload.sender,
        status: payload.status,
        created_at: now,
        updated_at: now,
    };
    let saved = repository::save_message_transaction(txn, message).await?;
    Ok(dto::ConversationMessageDto::from(saved))
}

pub async fn get_message(
    app_state: AppState,
    id: Uuid,
) -> Result<dto::ConversationMessageDto, ConversationError> {
    let model = repository::get_message(&app_state.database_connection, id).await?;
    Ok(dto::ConversationMessageDto::from(model))
}

pub async fn get_last_message_for_conversation(
    db: &DatabaseConnection,
    conversation_id: Uuid,
) -> Result<Option<dto::ConversationMessageDto>, ConversationError> {
    let model = repository::get_last_message_by_conversation_id(db, conversation_id).await?;
    Ok(model.map(dto::ConversationMessageDto::from))
}

pub async fn get_first_message_for_conversation(
    db: &DatabaseConnection,
    conversation_id: Uuid,
) -> Result<Option<dto::ConversationMessageDto>, ConversationError> {
    let model = repository::get_first_message_by_conversation_id(db, conversation_id).await?;
    Ok(model.map(dto::ConversationMessageDto::from))
}

pub async fn list_messages_for_conversation(
    db: &DatabaseConnection,
    conversation_id: Uuid,
    limit: Option<u64>,
    descending: bool,
) -> Result<Vec<dto::ConversationMessageDto>, ConversationError> {
    let models =
        repository::list_messages_by_conversation_id(db, conversation_id, limit, descending)
            .await?;
    Ok(models
        .into_iter()
        .map(dto::ConversationMessageDto::from)
        .collect())
}

pub async fn delete_messages_for_conversation_transaction(
    txn: &DatabaseTransaction,
    conversation_id: Uuid,
) -> Result<(), ConversationError> {
    repository::delete_messages_for_conversation_transaction(txn, conversation_id)
        .await
        .map_err(ConversationError::Database)
}
