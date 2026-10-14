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
                    .table(User::Table)
                    .col(pk_uuid(User::Id))
                    .col(string(User::Username))
                    .col(string_null(User::PasswordHash))
                    .col(string(User::Email))
                    .col(string(User::FullName))
                    .col(string(User::PlatformRole))
                    .to_owned(),
            ))
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum User {
    Table,
    Id,
    Username,
    PasswordHash,
    Email,
    FullName,
    PlatformRole,
}
