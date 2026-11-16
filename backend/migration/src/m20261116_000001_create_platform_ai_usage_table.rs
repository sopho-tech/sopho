use sea_orm_migration::{
    prelude::*,
    schema::{big_integer, timestamp_with_time_zone, uuid},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PlatformAiUsage::Table)
                    .col(uuid(PlatformAiUsage::TenantId).primary_key())
                    .col(big_integer(PlatformAiUsage::UsedTokens))
                    .col(timestamp_with_time_zone(PlatformAiUsage::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PlatformAiUsage::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PlatformAiUsage {
    Table,
    TenantId,
    UsedTokens,
    UpdatedAt,
}
