use sea_orm_migration::{prelude::*, schema::*};
use crate::db::add_created_at_and_updated_at_timestamps;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                add_created_at_and_updated_at_timestamps(
                    Table::create()
                        .table(DashboardChartMapping::Table)
                        .if_not_exists()
                        .col(pk_uuid(DashboardChartMapping::Id))
                        .col(uuid(DashboardChartMapping::DashboardId))
                        .col(uuid(DashboardChartMapping::ChartId))
                        .col(string(DashboardChartMapping::Status))
                        .to_owned()
                ),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DashboardChartMapping::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum DashboardChartMapping {
    Table,
    Id,
    DashboardId,
    ChartId,
    Status,
}
