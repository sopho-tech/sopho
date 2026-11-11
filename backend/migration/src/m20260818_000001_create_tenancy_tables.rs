use crate::db::{add_attribution_columns, add_created_at_and_updated_at_timestamps};
use sea_orm_migration::{
    prelude::*,
    schema::{boolean, pk_uuid, string, uuid},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                add_attribution_columns(
                    Table::create()
                        .table(Tenant::Table)
                        .col(pk_uuid(Tenant::Id))
                        .col(string(Tenant::Name))
                        .col(string(Tenant::Status))
                        .to_owned(),
                ),
            ))
            .await?;

        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                add_attribution_columns(
                    Table::create()
                        .table(Role::Table)
                        .col(pk_uuid(Role::Id))
                        .col(uuid(Role::TenantId))
                        .col(string(Role::Name))
                        .col(boolean(Role::IsSystem))
                        .to_owned(),
                ),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_role_tenant_name_unique")
                    .table(Role::Table)
                    .col(Role::TenantId)
                    .col(Role::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(RolePermission::Table)
                    .col(pk_uuid(RolePermission::Id))
                    .col(uuid(RolePermission::RoleId))
                    .col(string(RolePermission::Permission))
                    .to_owned(),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_role_permission_unique")
                    .table(RolePermission::Table)
                    .col(RolePermission::RoleId)
                    .col(RolePermission::Permission)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                add_attribution_columns(
                    Table::create()
                        .table(TenantMembership::Table)
                        .col(pk_uuid(TenantMembership::Id))
                        .col(uuid(TenantMembership::TenantId))
                        .col(uuid(TenantMembership::UserId))
                        .col(uuid(TenantMembership::RoleId))
                        .col(string(TenantMembership::Status))
                        .to_owned(),
                ),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tenant_membership_tenant_user_unique")
                    .table(TenantMembership::Table)
                    .col(TenantMembership::TenantId)
                    .col(TenantMembership::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tenant_membership_user")
                    .table(TenantMembership::Table)
                    .col(TenantMembership::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TenantMembership::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RolePermission::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Role::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tenant::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Tenant {
    Table,
    Id,
    Name,
    Status,
}

#[derive(DeriveIden)]
pub enum Role {
    Table,
    Id,
    TenantId,
    Name,
    IsSystem,
}

#[derive(DeriveIden)]
pub enum RolePermission {
    Table,
    Id,
    RoleId,
    Permission,
}

#[derive(DeriveIden)]
pub enum TenantMembership {
    Table,
    Id,
    TenantId,
    UserId,
    RoleId,
    Status,
}
