use crate::db::add_created_at_and_updated_at_timestamps;
use sea_orm_migration::{
    prelude::*,
    schema::{
        integer_null, pk_uuid, string, text_null, timestamp_with_time_zone,
        timestamp_with_time_zone_null, uuid,
    },
};

#[derive(DeriveMigrationName)]
pub struct Migration;

const UNIQUE_ENTITY_INDEX: &str = "idx_ai_summary_entity_type_entity_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(AiSummary::Table)
                    .col(pk_uuid(AiSummary::Id))
                    .col(string(AiSummary::EntityType))
                    .col(uuid(AiSummary::EntityId))
                    .col(string(AiSummary::Status))
                    .col(text_null(AiSummary::SummaryText))
                    .col(text_null(AiSummary::ErrorMessage))
                    .col(integer_null(AiSummary::SummarizedEntityCount))
                    .col(text_null(AiSummary::UserPrompt))
                    .col(text_null(AiSummary::UserPromptUsed))
                    .col(timestamp_with_time_zone_null(AiSummary::GeneratedAt))
                    .col(timestamp_with_time_zone(AiSummary::RequestedAt))
                    .to_owned(),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(UNIQUE_ENTITY_INDEX)
                    .table(AiSummary::Table)
                    .col(AiSummary::EntityType)
                    .col(AiSummary::EntityId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AiSummary::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum AiSummary {
    Table,
    Id,
    EntityType,
    EntityId,
    Status,
    SummaryText,
    ErrorMessage,
    SummarizedEntityCount,
    UserPrompt,
    UserPromptUsed,
    GeneratedAt,
    RequestedAt,
}
