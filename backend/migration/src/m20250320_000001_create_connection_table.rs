use crate::db::{add_attribution_columns, add_created_at_and_updated_at_timestamps};
use sea_orm_migration::{prelude::*, schema::*};

const TENANT_ID_INDEX: &str = "idx_connection_tenant_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                add_attribution_columns(
                    Table::create()
                        .table(Connection::Table)
                        .if_not_exists()
                        .col(pk_uuid(Connection::Id))
                        .col(uuid(Connection::TenantId))
                        .col(uuid(Connection::OwnerUserId))
                        .col(string(Connection::Visibility))
                        .col(string(Connection::Name))
                        .col(string_null(Connection::Description))
                        .col(string_null(Connection::Username))
                        .col(string_null(Connection::Password))
                        .col(string_null(Connection::Host))
                        .col(string_null(Connection::Port))
                        .col(string(Connection::Database))
                        .col(string_null(Connection::Schema))
                        .col(string(Connection::SourceType))
                        .col(string(Connection::Status))
                        .to_owned(),
                ),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(TENANT_ID_INDEX)
                    .table(Connection::Table)
                    .col(Connection::TenantId)
                    .to_owned(),
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
pub enum Connection {
    Table,
    Id,
    TenantId,
    OwnerUserId,
    Visibility,
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
