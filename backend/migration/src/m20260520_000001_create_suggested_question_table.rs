use crate::db::add_created_at_and_updated_at_timestamps;
use sea_orm_migration::{
    prelude::*,
    schema::{pk_uuid, string, timestamp_with_time_zone, uuid},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(SuggestedQuestion::Table)
                    .col(pk_uuid(SuggestedQuestion::Id))
                    .col(uuid(SuggestedQuestion::ConnectionId))
                    .col(string(SuggestedQuestion::QuestionText))
                    .col(timestamp_with_time_zone(SuggestedQuestion::GeneratedAt))
                    .to_owned(),
            ))
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SuggestedQuestion::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum SuggestedQuestion {
    Table,
    Id,
    ConnectionId,
    QuestionText,
    GeneratedAt,
}
