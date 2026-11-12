use crate::db::{add_attribution_columns, add_created_at_and_updated_at_timestamps};
use sea_orm_migration::{
    prelude::*,
    schema::{pk_uuid, string, string_null, uuid},
};
const TENANT_ID_INDEX: &str = "idx_notebook_tenant_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                add_attribution_columns(
                    Table::create()
                        .table(Notebook::Table)
                        .col(pk_uuid(Notebook::Id))
                        .col(uuid(Notebook::TenantId))
                        .col(uuid(Notebook::CanvasId))
                        .col(string(Notebook::Name))
                        .col(string_null(Notebook::Description))
                        .col(string(Notebook::Status))
                        .to_owned(),
                ),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(TENANT_ID_INDEX)
                    .table(Notebook::Table)
                    .col(Notebook::TenantId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Notebook::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Notebook {
    Table,
    Id,
    TenantId,
    CanvasId,
    Name,
    Description,
    Status,
}
