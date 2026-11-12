use crate::db::{GeneralIds, add_attribution_columns, add_created_at_and_updated_at_timestamps};
use sea_orm_migration::{
    prelude::*,
    schema::{pk_uuid, string, string_null, uuid},
};
const TENANT_ID_INDEX: &str = "idx_canvas_tenant_id";
const TENANT_STATUS_UPDATED_INDEX: &str = "idx_canvas_tenant_status_updated";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                add_attribution_columns(
                    Table::create()
                        .table(Canvas::Table)
                        .col(pk_uuid(Canvas::Id))
                        .col(uuid(Canvas::TenantId))
                        .col(uuid(Canvas::OwnerUserId))
                        .col(string(Canvas::Visibility))
                        .col(string(Canvas::Name))
                        .col(string_null(Canvas::Description))
                        .col(string(Canvas::Status))
                        .to_owned(),
                ),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(TENANT_ID_INDEX)
                    .table(Canvas::Table)
                    .col(Canvas::TenantId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(TENANT_STATUS_UPDATED_INDEX)
                    .table(Canvas::Table)
                    .col(Canvas::TenantId)
                    .col(Canvas::Status)
                    .col(GeneralIds::UpdatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Canvas::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Canvas {
    Table,
    Id,
    TenantId,
    OwnerUserId,
    Visibility,
    Name,
    Description,
    Status,
}
