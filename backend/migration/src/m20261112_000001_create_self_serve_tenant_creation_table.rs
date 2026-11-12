use sea_orm_migration::{
    prelude::*,
    schema::{pk_uuid, timestamp_with_time_zone, uuid},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SelfServeTenantCreation::Table)
                    .col(pk_uuid(SelfServeTenantCreation::Id))
                    .col(uuid(SelfServeTenantCreation::CreatorUserId))
                    .col(uuid(SelfServeTenantCreation::CreatedTenantId))
                    .col(timestamp_with_time_zone(SelfServeTenantCreation::CreatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_self_serve_tenant_creation_creator_user")
                    .table(SelfServeTenantCreation::Table)
                    .col(SelfServeTenantCreation::CreatorUserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_self_serve_tenant_creation_created_tenant_unique")
                    .table(SelfServeTenantCreation::Table)
                    .col(SelfServeTenantCreation::CreatedTenantId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(SelfServeTenantCreation::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
pub enum SelfServeTenantCreation {
    Table,
    Id,
    CreatorUserId,
    CreatedTenantId,
    CreatedAt,
}
