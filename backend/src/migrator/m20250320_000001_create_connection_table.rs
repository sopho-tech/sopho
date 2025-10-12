use sea_orm_migration::{prelude::*, schema::*};
use crate::db::add_created_at_and_updated_at_timestamps;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                add_created_at_and_updated_at_timestamps(
                    Table::create()
                        .table(Connection::Table)
                        .if_not_exists()
                        .col(pk_uuid(Connection::Id))
                        .col(string(Connection::Name))
                        .col(string_null(Connection::Description))
                        .col(string(Connection::Username))
                        .col(string(Connection::Password))
                        .col(string(Connection::Host))
                        .col(unsigned(Connection::Port))
                        .col(string(Connection::Database))
                        .col(string_null(Connection::Schema))
                        .col(string(Connection::SourceType))
                        .col(string(Connection::Status))
                        .to_owned()
                ),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Connection::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Connection {
    Table,
    Id,
    Name,
    Description,
    Username,
    Password,
    Host,
    Port,
    Database,
    Schema,
    SourceType,
    Status,
}
