use crate::db::add_created_at_and_updated_at_timestamps;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(Dashboard::Table)
                    .if_not_exists()
                    .col(pk_uuid(Dashboard::Id))
                    .col(uuid(Dashboard::CanvasId))
                    .col(string(Dashboard::Name))
                    .col(string(Dashboard::Title))
                    .col(string(Dashboard::Description))
                    .col(string(Dashboard::Status))
                    .to_owned(),
            ))
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Dashboard::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Dashboard {
    Table,
    Id,
    CanvasId,
    Name,
    Title,
    Description,
    Status,
}
