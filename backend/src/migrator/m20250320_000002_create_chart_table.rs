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
                        .table(Chart::Table)
                        .if_not_exists()
                        .col(pk_uuid(Chart::Id))
                        .col(string(Chart::Name))
                        .col(string(Chart::Title))
                        .col(string(Chart::Description))
                        .col(string(Chart::ChartType))
                        .col(string(Chart::Status))
                        .col(string(Chart::Query))
                        .col(uuid(Chart::ConnectionId))
                        .col(ColumnDef::new(Chart::PlotDetails).json_binary())
                        .to_owned()
                ),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Chart::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Chart {
    Table,
    Id,
    Name,
    Title,
    Description,
    ChartType,
    Status,
    Query,
    ConnectionId,
    PlotDetails,
}
