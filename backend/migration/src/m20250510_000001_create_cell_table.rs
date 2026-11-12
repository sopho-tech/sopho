use crate::db::{add_attribution_columns, add_created_at_and_updated_at_timestamps};
use sea_orm_migration::{
    prelude::*,
    schema::{integer, pk_uuid, string, string_null, uuid, uuid_null},
};
const TENANT_ID_INDEX: &str = "idx_cell_tenant_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                add_attribution_columns(
                    Table::create()
                        .table(Cell::Table)
                        .col(pk_uuid(Cell::Id))
                        .col(uuid(Cell::TenantId))
                        .col(uuid(Cell::NotebookId))
                        .col(uuid_null(Cell::ConnectionId))
                        .col(integer(Cell::DisplayOrder))
                        .col(string_null(Cell::Name))
                        .col(string_null(Cell::Content))
                        .col(string(Cell::CellType))
                        .col(string(Cell::Status))
                        .to_owned(),
                ),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(TENANT_ID_INDEX)
                    .table(Cell::Table)
                    .col(Cell::TenantId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Cell::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
#[expect(
    clippy::enum_variant_names,
    reason = "variant names map to existing column names"
)]
pub enum Cell {
    Table,
    Id,
    TenantId,
    NotebookId,
    ConnectionId,
    DisplayOrder,
    Name,
    Content,
    CellType,
    Status,
}
