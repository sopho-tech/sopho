use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::common::time_utils;
use crate::conversational_analytics::conversation::error::ConversationError;
use crate::conversational_analytics::message_content::dto;
use crate::conversational_analytics::message_content::repository;
use crate::entity;
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use uuid::Uuid;

pub async fn create_message_content(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    payload: dto::CreateConversationMessageContentDto,
    actor: &Actor,
) -> Result<(), ConversationError> {
    let now = time_utils::now_utc_into();
    let row = entity::conversation_message_content::Model {
        id: Uuid::new_v4(),
        tenant_id: tenant_id.get_uuid(),
        conversation_message_id: payload.conversation_message_id,
        sequence_number: payload.sequence_number,
        content_type: payload.content_type,
        content: payload.content,
        status: payload.status,
        created_by_user_id: actor.user_id(),
        created_by_actor: actor.kind_name(),
        modified_by_user_id: actor.user_id(),
        modified_by_actor: actor.kind_name(),
        created_at: now,
        updated_at: now,
    };
    repository::save_message_content_transaction(txn, tenant_id, row).await?;
    Ok(())
}

pub async fn create_message_content_connection(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    payload: dto::CreateConversationMessageContentDto,
    actor: &Actor,
) -> Result<dto::ConversationMessageContentDto, ConversationError> {
    let now = time_utils::now_utc_into();
    let row = entity::conversation_message_content::Model {
        id: Uuid::new_v4(),
        tenant_id: tenant_id.get_uuid(),
        conversation_message_id: payload.conversation_message_id,
        sequence_number: payload.sequence_number,
        content_type: payload.content_type,
        content: payload.content,
        status: payload.status,
        created_by_user_id: actor.user_id(),
        created_by_actor: actor.kind_name(),
        modified_by_user_id: actor.user_id(),
        modified_by_actor: actor.kind_name(),
        created_at: now,
        updated_at: now,
    };
    let saved = repository::save_message_content_connection(db, tenant_id, row).await?;
    Ok(dto::ConversationMessageContentDto::from(saved))
}

pub async fn get_first_message_content(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    conversation_message_id: Uuid,
) -> Result<Option<dto::ConversationMessageContentDto>, ConversationError> {
    let model = repository::get_first_message_content_by_conversation_message_id(
        db,
        tenant_id,
        conversation_message_id,
    )
    .await?;
    Ok(model.map(dto::ConversationMessageContentDto::from))
}

pub async fn list_message_content_for_message(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    conversation_message_id: Uuid,
) -> Result<Vec<dto::ConversationMessageContentDto>, ConversationError> {
    let models = repository::list_message_content_by_conversation_message_id(
        db,
        tenant_id,
        conversation_message_id,
    )
    .await?;
    Ok(models
        .into_iter()
        .map(dto::ConversationMessageContentDto::from)
        .collect())
}

pub async fn list_message_content_for_messages(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    conversation_message_ids: &[Uuid],
) -> Result<Vec<dto::ConversationMessageContentDto>, ConversationError> {
    if conversation_message_ids.is_empty() {
        return Ok(vec![]);
    }
    let models = repository::list_message_content_by_conversation_message_ids(
        db,
        tenant_id,
        conversation_message_ids,
    )
    .await?;
    Ok(models
        .into_iter()
        .map(dto::ConversationMessageContentDto::from)
        .collect())
}

pub async fn delete_message_contents_for_conversation_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    conversation_id: Uuid,
) -> Result<(), ConversationError> {
    repository::delete_message_contents_for_conversation_transaction(
        txn,
        tenant_id,
        conversation_id,
    )
    .await
    .map_err(ConversationError::Database)
}

pub async fn delete_message_contents_for_conversations_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    conversation_ids: &[Uuid],
) -> Result<(), ConversationError> {
    repository::delete_message_contents_for_conversations_transaction(
        txn,
        tenant_id,
        conversation_ids,
    )
    .await
    .map_err(ConversationError::Database)
}
