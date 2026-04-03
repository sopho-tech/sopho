use crate::db::add_created_at_and_updated_at_timestamps;
use sea_orm_migration::{
    prelude::*,
    schema::{integer, pk_uuid, string, uuid},
};
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(ConversationMessageContent::Table)
                    .col(pk_uuid(ConversationMessageContent::Id))
                    .col(uuid(ConversationMessageContent::ConversationMessageId))
                    .col(integer(ConversationMessageContent::SequenceNumber))
                    .col(string(ConversationMessageContent::ContentType))
                    .col(string(ConversationMessageContent::Content))
                    .col(string(ConversationMessageContent::Status))
                    .to_owned(),
            ))
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
    ConversationMessageId,
    SequenceNumber,
    ContentType,
    Content,
    Status,
}
