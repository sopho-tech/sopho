use crate::db::add_created_at_and_updated_at_timestamps;
use sea_orm_migration::{
    prelude::*,
    schema::{pk_uuid, string, uuid},
};
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(Conversation::Table)
                    .col(pk_uuid(Conversation::Id))
                    .col(uuid(Conversation::ConnectionId))
                    .col(string(Conversation::Name))
                    .col(string(Conversation::Status))
                    .to_owned(),
            ))
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Conversation::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Conversation {
    Table,
    Id,
    ConnectionId,
    Name,
    Status,
}
