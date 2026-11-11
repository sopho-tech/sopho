use crate::db::{add_attribution_columns, add_created_at_and_updated_at_timestamps};
use sea_orm_migration::{
    prelude::*,
    schema::{string, timestamp_with_time_zone_null, uuid},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                add_attribution_columns(
                    Table::create()
                        .table(AiConfiguration::Table)
                        .col(uuid(AiConfiguration::TenantId))
                        .primary_key(
                            Index::create()
                                .name("ai_configuration_tenant_pkey")
                                .col(AiConfiguration::TenantId),
                        )
                        .col(string(AiConfiguration::Provider))
                        .col(string(AiConfiguration::ApiKeyEncrypted))
                        .col(string(AiConfiguration::LivenessStatus))
                        .col(timestamp_with_time_zone_null(
                            AiConfiguration::LastCheckedAt,
                        ))
                        .to_owned(),
                ),
            ))
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AiConfiguration::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum AiConfiguration {
    Table,
    TenantId,
    Provider,
    ApiKeyEncrypted,
    LivenessStatus,
    LastCheckedAt,
}
