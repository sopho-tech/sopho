use sea_orm_migration::{
    prelude::*,
    schema::{
        big_integer, json_binary, pk_uuid, string, timestamp_with_time_zone, uuid, uuid_null,
    },
};

const CREATE_CONVERSATION_INDEX_SQL: &str = "CREATE INDEX idx_ai_usage_tenant_conversation ON ai_usage (tenant_id, (metadata ->> 'conversation_id'))";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AiUsage::Table)
                    .col(pk_uuid(AiUsage::Id))
                    .col(uuid(AiUsage::TenantId))
                    .col(uuid_null(AiUsage::UserId))
                    .col(string(AiUsage::AgentName))
                    .col(string(AiUsage::Provider))
                    .col(string(AiUsage::Model))
                    .col(string(AiUsage::KeySource))
                    .col(big_integer(AiUsage::InputTokens))
                    .col(big_integer(AiUsage::OutputTokens))
                    .col(big_integer(AiUsage::CachedInputTokens))
                    .col(json_binary(AiUsage::Metadata))
                    .col(timestamp_with_time_zone(AiUsage::CreatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ai_usage_tenant_created_at")
                    .table(AiUsage::Table)
                    .col(AiUsage::TenantId)
                    .col(AiUsage::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(CREATE_CONVERSATION_INDEX_SQL)
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AiUsage::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum AiUsage {
    Table,
    Id,
    TenantId,
    UserId,
    AgentName,
    Provider,
    Model,
    KeySource,
    InputTokens,
    OutputTokens,
    CachedInputTokens,
    Metadata,
    CreatedAt,
}
