use crate::db::add_created_at_and_updated_at_timestamps;
use crate::m20250317_000001_create_conversation_table::Conversation;
use crate::m20250320_000001_create_connection_table::Connection;
use crate::m20251210_000001_create_canvas_table::Canvas;
use sea_orm_migration::{
    prelude::*,
    schema::{pk_uuid, string, uuid},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

struct TenantOwnerIndex {
    name: &'static str,
    table: DynIden,
    tenant_id_column: DynIden,
    owner_user_id_column: DynIden,
}

impl TenantOwnerIndex {
    fn new(
        name: &'static str,
        table: impl IntoIden,
        tenant_id_column: impl IntoIden,
        owner_user_id_column: impl IntoIden,
    ) -> Self {
        Self {
            name,
            table: table.into_iden(),
            tenant_id_column: tenant_id_column.into_iden(),
            owner_user_id_column: owner_user_id_column.into_iden(),
        }
    }
}

fn tenant_owner_indexes() -> Vec<TenantOwnerIndex> {
    vec![
        TenantOwnerIndex::new(
            "idx_canvas_tenant_owner",
            Canvas::Table,
            Canvas::TenantId,
            Canvas::OwnerUserId,
        ),
        TenantOwnerIndex::new(
            "idx_conversation_tenant_owner",
            Conversation::Table,
            Conversation::TenantId,
            Conversation::OwnerUserId,
        ),
        TenantOwnerIndex::new(
            "idx_connection_tenant_owner",
            Connection::Table,
            Connection::TenantId,
            Connection::OwnerUserId,
        ),
    ]
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for tenant_owner_index in tenant_owner_indexes() {
            manager
                .create_index(
                    Index::create()
                        .name(tenant_owner_index.name)
                        .table(tenant_owner_index.table)
                        .col(tenant_owner_index.tenant_id_column)
                        .col(tenant_owner_index.owner_user_id_column)
                        .to_owned(),
                )
                .await?;
        }

        manager
            .create_table(add_created_at_and_updated_at_timestamps(
                Table::create()
                    .table(ResourceGrant::Table)
                    .col(pk_uuid(ResourceGrant::Id))
                    .col(uuid(ResourceGrant::TenantId))
                    .col(string(ResourceGrant::ResourceType))
                    .col(uuid(ResourceGrant::ResourceId))
                    .col(string(ResourceGrant::PrincipalType))
                    .col(uuid(ResourceGrant::PrincipalId))
                    .col(string(ResourceGrant::AccessLevel))
                    .col(uuid(ResourceGrant::GrantedByUserId))
                    .to_owned(),
            ))
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_resource_grant_unique")
                    .table(ResourceGrant::Table)
                    .col(ResourceGrant::ResourceType)
                    .col(ResourceGrant::ResourceId)
                    .col(ResourceGrant::PrincipalType)
                    .col(ResourceGrant::PrincipalId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_resource_grant_principal")
                    .table(ResourceGrant::Table)
                    .col(ResourceGrant::PrincipalId)
                    .col(ResourceGrant::ResourceType)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ResourceGrant::Table).to_owned())
            .await?;

        for tenant_owner_index in tenant_owner_indexes() {
            manager
                .drop_index(
                    Index::drop()
                        .name(tenant_owner_index.name)
                        .table(tenant_owner_index.table)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum ResourceGrant {
    Table,
    Id,
    TenantId,
    ResourceType,
    ResourceId,
    PrincipalType,
    PrincipalId,
    AccessLevel,
    GrantedByUserId,
}
