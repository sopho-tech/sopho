use sea_orm_migration::{prelude::*, schema::{pk_uuid, string, timestamp_with_time_zone, uuid}};
use crate::db::add_created_at_and_updated_at_timestamps;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(Session::Table)
                    .col(pk_uuid(Session::Id))
                    .col(uuid(Session::UserId))
                    .col(string(Session::RefreshToken))
                    .col(string(Session::AccessToken))
                    .col(timestamp_with_time_zone(Session::RefreshTokenExpiresAt))
                    .col(timestamp_with_time_zone(Session::AccessTokenExpiresAt))
                    .col(string(Session::Status))
                    .to_owned(),
            ))
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Session::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Session {
    Table,
    Id,
    UserId,
    RefreshToken,
    AccessToken,
    RefreshTokenExpiresAt,
    AccessTokenExpiresAt,
    Status,
}
