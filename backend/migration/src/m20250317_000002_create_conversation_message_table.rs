use crate::db::{add_attribution_columns, add_created_at_and_updated_at_timestamps};
use sea_orm_migration::{
    prelude::*,
    schema::{integer, pk_uuid, string, uuid},
};
const TENANT_ID_INDEX: &str = "idx_conversation_message_tenant_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                add_attribution_columns(
                    Table::create()
                        .table(ConversationMessage::Table)
                        .col(pk_uuid(ConversationMessage::Id))
                        .col(uuid(ConversationMessage::TenantId))
                        .col(uuid(ConversationMessage::ConversationId))
                        .col(integer(ConversationMessage::SequenceNumber))
                        .col(string(ConversationMessage::Sender))
                        .col(string(ConversationMessage::Status))
                        .to_owned(),
                ),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(TENANT_ID_INDEX)
                    .table(ConversationMessage::Table)
                    .col(ConversationMessage::TenantId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ConversationMessage::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum ConversationMessage {
    Table,
    Id,
    TenantId,
    ConversationId,
    SequenceNumber,
    Sender,
    Status,
}
