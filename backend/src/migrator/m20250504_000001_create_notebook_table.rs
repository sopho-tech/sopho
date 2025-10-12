use sea_orm_migration::{prelude::*, schema::{pk_uuid, string, string_null}};
use crate::db::add_created_at_and_updated_at_timestamps;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(Notebook::Table)
                    .col(pk_uuid(Notebook::Id))
                    .col(string(Notebook::Name))
                    .col(string_null(Notebook::Description))
                    .col(string(Notebook::Status))
                    .to_owned(),
            ))
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
    Name,
    Description,
    Status,
}
