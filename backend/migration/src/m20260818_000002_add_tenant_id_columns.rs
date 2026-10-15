use crate::db::{Attribution, GeneralIds, SYSTEM_ACTOR};
use crate::m20250221_000001_create_configuration_table::Configuration;
use crate::m20250315_000001_create_dashboard_table::Dashboard;
use crate::m20250317_000001_create_conversation_table::Conversation;
use crate::m20250317_000002_create_conversation_message_content_table::ConversationMessageContent;
use crate::m20250317_000002_create_conversation_message_table::ConversationMessage;
use crate::m20250320_000001_create_connection_table::Connection;
use crate::m20250504_000001_create_notebook_table::Notebook;
use crate::m20250510_000001_create_cell_table::Cell;
use crate::m20251210_000001_create_canvas_table::Canvas;
use crate::m20260520_000001_create_suggested_question_table::SuggestedQuestion;
use crate::m20260810_000001_create_ai_summary_table::AiSummary;
use crate::m20260818_000001_create_tenancy_tables::Tenant;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const DEFAULT_TENANT_NAME: &str = "Default";
const DEFAULT_TENANT_STATUS: &str = "ACTIVE";
const DEFAULT_TENANT_CONFIGURATION_KEY: &str = "default_tenant_id";
const CANVAS_TENANT_STATUS_UPDATED_INDEX: &str = "idx_canvas_tenant_status_updated";

struct TenantIdIndex {
    name: &'static str,
    table: DynIden,
    tenant_id_column: DynIden,
}

impl TenantIdIndex {
    fn new(name: &'static str, table: impl IntoIden, tenant_id_column: impl IntoIden) -> Self {
        Self {
            name,
            table: table.into_iden(),
            tenant_id_column: tenant_id_column.into_iden(),
        }
    }
}

fn tenant_id_indexes() -> Vec<TenantIdIndex> {
    vec![
        TenantIdIndex::new("idx_canvas_tenant_id", Canvas::Table, Canvas::TenantId),
        TenantIdIndex::new(
            "idx_dashboard_tenant_id",
            Dashboard::Table,
            Dashboard::TenantId,
        ),
        TenantIdIndex::new(
            "idx_notebook_tenant_id",
            Notebook::Table,
            Notebook::TenantId,
        ),
        TenantIdIndex::new("idx_cell_tenant_id", Cell::Table, Cell::TenantId),
        TenantIdIndex::new(
            "idx_conversation_tenant_id",
            Conversation::Table,
            Conversation::TenantId,
        ),
        TenantIdIndex::new(
            "idx_conversation_message_tenant_id",
            ConversationMessage::Table,
            ConversationMessage::TenantId,
        ),
        TenantIdIndex::new(
            "idx_conversation_message_content_tenant_id",
            ConversationMessageContent::Table,
            ConversationMessageContent::TenantId,
        ),
        TenantIdIndex::new(
            "idx_connection_tenant_id",
            Connection::Table,
            Connection::TenantId,
        ),
        TenantIdIndex::new(
            "idx_suggested_question_tenant_id",
            SuggestedQuestion::Table,
            SuggestedQuestion::TenantId,
        ),
        TenantIdIndex::new(
            "idx_ai_summary_tenant_id",
            AiSummary::Table,
            AiSummary::TenantId,
        ),
    ]
}

async fn read_default_tenant_id(manager: &SchemaManager<'_>) -> Result<uuid::Uuid, DbErr> {
    let select_default_tenant_id = Query::select()
        .column(Configuration::Value)
        .from(Configuration::Table)
        .and_where(Expr::col(Configuration::Key).eq(DEFAULT_TENANT_CONFIGURATION_KEY))
        .to_owned();

    let configuration_row = manager
        .get_connection()
        .query_one(
            manager
                .get_database_backend()
                .build(&select_default_tenant_id),
        )
        .await?
        .ok_or_else(|| {
            DbErr::Custom(format!(
                "configuration `{DEFAULT_TENANT_CONFIGURATION_KEY}` is missing"
            ))
        })?;

    let stored_tenant_id: String = configuration_row.try_get("", "value")?;
    uuid::Uuid::parse_str(&stored_tenant_id).map_err(|parse_error| {
        DbErr::Custom(format!(
            "configuration `{DEFAULT_TENANT_CONFIGURATION_KEY}` is not a uuid: {parse_error}"
        ))
    })
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let default_tenant_id = uuid::Uuid::new_v4();

        manager
            .exec_stmt(
                Query::insert()
                    .into_table(Tenant::Table)
                    .columns([
                        Tenant::Id.into_iden(),
                        Tenant::Name.into_iden(),
                        Tenant::Status.into_iden(),
                        Attribution::CreatedByActor.into_iden(),
                        Attribution::ModifiedByActor.into_iden(),
                    ])
                    .values_panic([
                        default_tenant_id.into(),
                        DEFAULT_TENANT_NAME.into(),
                        DEFAULT_TENANT_STATUS.into(),
                        SYSTEM_ACTOR.into(),
                        SYSTEM_ACTOR.into(),
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::insert()
                    .into_table(Configuration::Table)
                    .columns([Configuration::Key, Configuration::Value])
                    .values_panic([
                        DEFAULT_TENANT_CONFIGURATION_KEY.into(),
                        default_tenant_id.to_string().into(),
                    ])
                    .to_owned(),
            )
            .await?;

        for tenant_id_index in tenant_id_indexes() {
            manager
                .create_index(
                    Index::create()
                        .name(tenant_id_index.name)
                        .table(tenant_id_index.table)
                        .col(tenant_id_index.tenant_id_column)
                        .to_owned(),
                )
                .await?;
        }

        manager
            .create_index(
                Index::create()
                    .name(CANVAS_TENANT_STATUS_UPDATED_INDEX)
                    .table(Canvas::Table)
                    .col(Canvas::TenantId)
                    .col(Canvas::Status)
                    .col(GeneralIds::UpdatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(CANVAS_TENANT_STATUS_UPDATED_INDEX)
                    .table(Canvas::Table)
                    .to_owned(),
            )
            .await?;

        for tenant_id_index in tenant_id_indexes() {
            manager
                .drop_index(
                    Index::drop()
                        .name(tenant_id_index.name)
                        .table(tenant_id_index.table)
                        .to_owned(),
                )
                .await?;
        }

        let default_tenant_id = read_default_tenant_id(manager).await?;

        manager
            .exec_stmt(
                Query::delete()
                    .from_table(Tenant::Table)
                    .and_where(Expr::col(Tenant::Id).eq(default_tenant_id))
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::delete()
                    .from_table(Configuration::Table)
                    .and_where(Expr::col(Configuration::Key).eq(DEFAULT_TENANT_CONFIGURATION_KEY))
                    .to_owned(),
            )
            .await
    }
}
