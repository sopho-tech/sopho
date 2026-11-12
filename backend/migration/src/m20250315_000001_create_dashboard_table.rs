use crate::db::{add_attribution_columns, add_created_at_and_updated_at_timestamps};
use sea_orm_migration::{prelude::*, schema::*};

const TENANT_ID_INDEX: &str = "idx_dashboard_tenant_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                add_attribution_columns(
                    Table::create()
                        .table(Dashboard::Table)
                        .if_not_exists()
                        .col(pk_uuid(Dashboard::Id))
                        .col(uuid(Dashboard::TenantId))
                        .col(uuid(Dashboard::CanvasId))
                        .col(string(Dashboard::Name))
                        .col(string(Dashboard::Description))
                        .col(json_binary_null(Dashboard::Layout))
                        .col(string(Dashboard::Status))
                        .to_owned(),
                ),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(TENANT_ID_INDEX)
                    .table(Dashboard::Table)
                    .col(Dashboard::TenantId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Dashboard::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Dashboard {
    Table,
    Id,
    TenantId,
    CanvasId,
    Name,
    Description,
    Layout,
    Status,
}
