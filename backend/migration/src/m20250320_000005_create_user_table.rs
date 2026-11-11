use crate::db::{add_created_at_and_updated_at_timestamps, user_status_names};
use sea_orm_migration::{
    prelude::*,
    schema::{integer, pk_uuid, string, string_null, timestamp_with_time_zone_null},
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
                    .col(timestamp_with_time_zone_null(User::EmailVerifiedAt))
                    .col(string(User::Status))
                    .col(integer(User::FailedSignInCount).default(0))
                    .col(timestamp_with_time_zone_null(User::SignInBlockedUntil))
                    .check(Expr::col(User::Status).is_in(user_status_names()))
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
    EmailVerifiedAt,
    Status,
    FailedSignInCount,
    SignInBlockedUntil,
}
