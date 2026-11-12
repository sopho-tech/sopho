use crate::db::{add_attribution_columns, add_created_at_and_updated_at_timestamps};
use sea_orm_migration::{
    prelude::*,
    schema::{integer, pk_uuid, string, uuid},
};
const TENANT_ID_INDEX: &str = "idx_conversation_message_content_tenant_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                add_attribution_columns(
                    Table::create()
                        .table(ConversationMessageContent::Table)
                        .col(pk_uuid(ConversationMessageContent::Id))
                        .col(uuid(ConversationMessageContent::TenantId))
                        .col(uuid(ConversationMessageContent::ConversationMessageId))
                        .col(integer(ConversationMessageContent::SequenceNumber))
                        .col(string(ConversationMessageContent::ContentType))
                        .col(string(ConversationMessageContent::Content))
                        .col(string(ConversationMessageContent::Status))
                        .to_owned(),
                ),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(TENANT_ID_INDEX)
                    .table(ConversationMessageContent::Table)
                    .col(ConversationMessageContent::TenantId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ConversationMessageContent::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
pub enum ConversationMessageContent {
    Table,
    Id,
    TenantId,
    ConversationMessageId,
    SequenceNumber,
    ContentType,
    Content,
    Status,
}
