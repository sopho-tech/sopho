use crate::m20250315_000001_create_dashboard_table::Dashboard;
use crate::m20250320_000005_create_user_table::User;
use crate::m20250404_000001_create_session_table::Session;
use crate::m20250504_000001_create_notebook_table::Notebook;
use crate::m20260810_000001_create_ai_summary_table::{AiSummary, UNIQUE_ENTITY_INDEX};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

struct UniqueIndex {
    name: &'static str,
    table: DynIden,
    columns: Vec<DynIden>,
}

impl UniqueIndex {
    fn new(name: &'static str, table: impl IntoIden, columns: Vec<DynIden>) -> Self {
        Self {
            name,
            table: table.into_iden(),
            columns,
        }
    }
}

fn unique_indexes() -> Vec<UniqueIndex> {
    vec![
        UniqueIndex::new(
            "idx_user_email_unique",
            User::Table,
            vec![User::Email.into_iden()],
        ),
        UniqueIndex::new(
            "idx_user_username_unique",
            User::Table,
            vec![User::Username.into_iden()],
        ),
        UniqueIndex::new(
            "idx_session_access_token_hash_unique",
            Session::Table,
            vec![Session::AccessTokenHash.into_iden()],
        ),
        UniqueIndex::new(
            "idx_session_refresh_token_hash_unique",
            Session::Table,
            vec![Session::RefreshTokenHash.into_iden()],
        ),
        UniqueIndex::new(
            "idx_notebook_canvas_id_unique",
            Notebook::Table,
            vec![Notebook::CanvasId.into_iden()],
        ),
        UniqueIndex::new(
            "idx_dashboard_canvas_id_unique",
            Dashboard::Table,
            vec![Dashboard::CanvasId.into_iden()],
        ),
        UniqueIndex::new(
            "idx_ai_summary_tenant_entity_unique",
            AiSummary::Table,
            vec![
                AiSummary::TenantId.into_iden(),
                AiSummary::EntityType.into_iden(),
                AiSummary::EntityId.into_iden(),
            ],
        ),
    ]
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(UNIQUE_ENTITY_INDEX)
                    .table(AiSummary::Table)
                    .to_owned(),
            )
            .await?;

        for unique_index in unique_indexes() {
            let mut statement = Index::create();
            statement
                .name(unique_index.name)
                .table(unique_index.table)
                .unique();
            for column in unique_index.columns {
                statement.col(column);
            }
            manager.create_index(statement.to_owned()).await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for unique_index in unique_indexes() {
            manager
                .drop_index(
                    Index::drop()
                        .name(unique_index.name)
                        .table(unique_index.table)
                        .to_owned(),
                )
                .await?;
        }

        manager
            .create_index(
                Index::create()
                    .name(UNIQUE_ENTITY_INDEX)
                    .table(AiSummary::Table)
                    .col(AiSummary::EntityType)
                    .col(AiSummary::EntityId)
                    .unique()
                    .to_owned(),
            )
            .await
    }
}
