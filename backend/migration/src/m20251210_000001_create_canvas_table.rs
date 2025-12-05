use crate::db::add_created_at_and_updated_at_timestamps;
use sea_orm_migration::{
    prelude::*,
    schema::{pk_uuid, string, string_null},
};
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(Canvas::Table)
                    .col(pk_uuid(Canvas::Id))
                    .col(string(Canvas::Name))
                    .col(string_null(Canvas::Description))
                    .col(string(Canvas::Status))
                    .to_owned(),
            ))
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
    Name,
    Description,
    Status,
}
