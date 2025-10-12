use sea_orm_migration::{prelude::*, schema::{integer, pk_uuid, string, string_null, uuid, uuid_null}};
use crate::db::add_created_at_and_updated_at_timestamps;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(Cell::Table)
                    .col(pk_uuid(Cell::Id))
                    .col(uuid(Cell::NotebookId))
                    .col(uuid_null(Cell::ConnectionId))
                    .col(integer(Cell::DisplayOrder))
                    .col(string_null(Cell::Name))
                    .col(string_null(Cell::Content))
                    .col(string(Cell::CellType))
                    .col(string(Cell::Status))
                    .to_owned(),
            ))
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Cell::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Cell {
    Table,
    Id,
    NotebookId,
    ConnectionId,
    DisplayOrder,
    Name,
    Content,
    CellType,
    Status,
}
