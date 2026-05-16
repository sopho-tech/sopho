use crate::db::add_created_at_and_updated_at_timestamps;
use sea_orm_migration::{
    prelude::*,
    schema::{string, timestamp_with_time_zone_null},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(AiConfiguration::Table)
                    .col(
                        ColumnDef::new(AiConfiguration::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(string(AiConfiguration::Provider))
                    .col(string(AiConfiguration::ApiKeyEncrypted))
                    .col(string(AiConfiguration::LivenessStatus))
                    .col(timestamp_with_time_zone_null(AiConfiguration::LastCheckedAt))
                    .to_owned(),
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
    Id,
    Provider,
    ApiKeyEncrypted,
    LivenessStatus,
    LastCheckedAt,
}
