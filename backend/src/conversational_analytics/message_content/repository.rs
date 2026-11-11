use crate::authorization::dto::TenantId;
use crate::entity::conversation_message;
use crate::entity::conversation_message_content;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, QueryTrait, Set,
};
use uuid::Uuid;

pub async fn save_message_content_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    row: conversation_message_content::Model,
) -> Result<conversation_message_content::Model, DbErr> {
    let mut active: conversation_message_content::ActiveModel = row.into();
    active.tenant_id = Set(tenant_id.get_uuid());
    let inserted = active.insert(txn).await?;
    Ok(inserted)
}

pub async fn save_message_content_connection(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    row: conversation_message_content::Model,
) -> Result<conversation_message_content::Model, DbErr> {
    let mut active: conversation_message_content::ActiveModel = row.into();
    active.tenant_id = Set(tenant_id.get_uuid());
    let inserted = active.insert(db).await?;
    Ok(inserted)
}

pub async fn get_first_message_content_by_conversation_message_id(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    conversation_message_id: Uuid,
) -> Result<Option<conversation_message_content::Model>, DbErr> {
    conversation_message_content::Entity::find()
        .filter(conversation_message_content::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(
            conversation_message_content::Column::ConversationMessageId.eq(conversation_message_id),
        )
        .order_by_asc(conversation_message_content::Column::SequenceNumber)
        .one(db)
        .await
}

pub async fn list_message_content_by_conversation_message_id(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    conversation_message_id: Uuid,
) -> Result<Vec<conversation_message_content::Model>, DbErr> {
    conversation_message_content::Entity::find()
        .filter(conversation_message_content::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(
            conversation_message_content::Column::ConversationMessageId.eq(conversation_message_id),
        )
        .order_by_asc(conversation_message_content::Column::SequenceNumber)
        .all(db)
        .await
}

pub async fn list_message_content_by_conversation_message_ids(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    conversation_message_ids: &[Uuid],
) -> Result<Vec<conversation_message_content::Model>, DbErr> {
    conversation_message_content::Entity::find()
        .filter(conversation_message_content::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(
            conversation_message_content::Column::ConversationMessageId
                .is_in(conversation_message_ids.to_vec()),
        )
        .order_by_asc(conversation_message_content::Column::ConversationMessageId)
        .order_by_asc(conversation_message_content::Column::SequenceNumber)
        .all(db)
        .await
}

pub async fn delete_message_contents_for_conversations_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    conversation_ids: &[Uuid],
) -> Result<(), DbErr> {
    let message_id_subquery = conversation_message::Entity::find()
        .filter(conversation_message::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(conversation_message::Column::ConversationId.is_in(conversation_ids.to_vec()))
        .select_only()
        .column(conversation_message::Column::Id)
        .into_query();
    conversation_message_content::Entity::delete_many()
        .filter(conversation_message_content::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(
            conversation_message_content::Column::ConversationMessageId
                .in_subquery(message_id_subquery),
        )
        .exec(txn)
        .await?;
    Ok(())
}

pub async fn delete_message_contents_for_conversation_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    conversation_id: Uuid,
) -> Result<(), DbErr> {
    let message_id_subquery = conversation_message::Entity::find()
        .filter(conversation_message::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(conversation_message::Column::ConversationId.eq(conversation_id))
        .select_only()
        .column(conversation_message::Column::Id)
        .into_query();
    conversation_message_content::Entity::delete_many()
        .filter(conversation_message_content::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(
            conversation_message_content::Column::ConversationMessageId
                .in_subquery(message_id_subquery),
        )
        .exec(txn)
        .await?;
    Ok(())
}
