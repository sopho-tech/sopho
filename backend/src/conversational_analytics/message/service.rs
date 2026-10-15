use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::common::time_utils;
use crate::common::AppState;
use crate::conversational_analytics::conversation::error::ConversationError;
use crate::conversational_analytics::conversation::service as conversation_service;
use crate::conversational_analytics::message::constants::Sender;
use crate::conversational_analytics::message::dto;
use crate::conversational_analytics::message::repository;
use crate::entity;
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use std::collections::HashMap;
use uuid::Uuid;

pub async fn create_message(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    payload: dto::CreateConversationMessageDto,
    actor: &Actor,
) -> Result<dto::ConversationMessageDto, ConversationError> {
    let now = time_utils::now_utc_into();
    let message = entity::conversation_message::Model {
        id: Uuid::new_v4(),
        tenant_id: tenant_id.get_uuid(),
        conversation_id: payload.conversation_id,
        sequence_number: payload.sequence_number,
        sender: payload.sender,
        status: payload.status,
        created_by_user_id: actor.user_id(),
        created_by_actor: actor.kind_name(),
        modified_by_user_id: actor.user_id(),
        modified_by_actor: actor.kind_name(),
        created_at: now,
        updated_at: now,
    };
    let saved = repository::save_message_transaction(txn, tenant_id, message).await?;
    conversation_service::stamp_modified(txn, tenant_id, saved.conversation_id, actor).await?;
    Ok(dto::ConversationMessageDto::from(saved))
}

pub async fn update_message_status(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    message_id: Uuid,
    status: String,
    actor: &Actor,
) -> Result<(), ConversationError> {
    let message = repository::update_message_status(
        database_connection,
        tenant_id,
        message_id,
        status,
        actor,
    )
    .await?
    .ok_or(ConversationError::NotFound)?;
    conversation_service::stamp_modified(
        database_connection,
        tenant_id,
        message.conversation_id,
        actor,
    )
    .await?;
    Ok(())
}

pub async fn get_message(
    app_state: AppState,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<dto::ConversationMessageDto, ConversationError> {
    let model = repository::get_message(&app_state.database_connection, tenant_id, id)
        .await?
        .ok_or(ConversationError::NotFound)?;
    Ok(dto::ConversationMessageDto::from(model))
}

pub async fn get_last_message_for_conversation(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    conversation_id: Uuid,
) -> Result<Option<dto::ConversationMessageDto>, ConversationError> {
    let model =
        repository::get_last_message_by_conversation_id(db, tenant_id, conversation_id).await?;
    Ok(model.map(dto::ConversationMessageDto::from))
}

pub async fn get_first_message_for_conversation(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    conversation_id: Uuid,
) -> Result<Option<dto::ConversationMessageDto>, ConversationError> {
    let model =
        repository::get_first_message_by_conversation_id(db, tenant_id, conversation_id).await?;
    Ok(model.map(dto::ConversationMessageDto::from))
}

pub async fn list_messages_for_conversation(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    conversation_id: Uuid,
    limit: Option<u64>,
    descending: bool,
) -> Result<Vec<dto::ConversationMessageDto>, ConversationError> {
    let models = repository::list_messages_by_conversation_id(
        db,
        tenant_id,
        conversation_id,
        limit,
        descending,
    )
    .await?;
    Ok(models
        .into_iter()
        .map(dto::ConversationMessageDto::from)
        .collect())
}

pub async fn count_user_messages_for_conversation(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    conversation_id: Uuid,
) -> Result<i64, ConversationError> {
    let counts = count_messages_by_sender_for_conversations(
        db,
        tenant_id,
        &[conversation_id],
        &Sender::Human.to_string(),
    )
    .await?;
    Ok(counts.get(&conversation_id).copied().unwrap_or(0))
}

pub async fn count_messages_by_sender_for_conversations(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    conversation_ids: &[Uuid],
    sender: &str,
) -> Result<HashMap<Uuid, i64>, ConversationError> {
    if conversation_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = repository::count_messages_by_sender_for_conversations(
        db,
        tenant_id,
        conversation_ids,
        sender,
    )
    .await?;
    Ok(rows.into_iter().collect())
}

pub async fn delete_messages_for_conversation_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    conversation_id: Uuid,
) -> Result<(), ConversationError> {
    repository::delete_messages_for_conversation_transaction(txn, tenant_id, conversation_id)
        .await
        .map_err(ConversationError::Database)
}

pub async fn delete_messages_for_conversations_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    conversation_ids: &[Uuid],
) -> Result<(), ConversationError> {
    repository::delete_messages_for_conversations_transaction(txn, tenant_id, conversation_ids)
        .await
        .map_err(ConversationError::Database)
}
