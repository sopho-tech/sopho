use crate::ai_summary::constants::{SummaryEntityType, SummaryStatus};
use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::common::time_utils;
use crate::entity::ai_summary;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

pub async fn find(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    entity_type: SummaryEntityType,
    entity_id: Uuid,
) -> Result<Option<ai_summary::Model>, DbErr> {
    ai_summary::Entity::find()
        .filter(ai_summary::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(ai_summary::Column::EntityType.eq(entity_type.to_string()))
        .filter(ai_summary::Column::EntityId.eq(entity_id))
        .one(db)
        .await
}

pub async fn find_for_dashboard(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    dashboard_id: Uuid,
    chart_cell_ids: &[Uuid],
) -> Result<Vec<ai_summary::Model>, DbErr> {
    let mut condition = Condition::any().add(
        Condition::all()
            .add(ai_summary::Column::EntityType.eq(SummaryEntityType::Dashboard.to_string()))
            .add(ai_summary::Column::EntityId.eq(dashboard_id)),
    );

    if !chart_cell_ids.is_empty() {
        condition = condition.add(
            Condition::all()
                .add(ai_summary::Column::EntityType.eq(SummaryEntityType::ChartCell.to_string()))
                .add(ai_summary::Column::EntityId.is_in(chart_cell_ids.iter().copied())),
        );
    }

    ai_summary::Entity::find()
        .filter(ai_summary::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(condition)
        .all(db)
        .await
}

pub async fn try_mark_generating(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    entity_type: SummaryEntityType,
    entity_id: Uuid,
    stale_before: DateTimeWithTimeZone,
    actor: &Actor,
) -> Result<u64, DbErr> {
    let now: DateTimeWithTimeZone = time_utils::now_utc_into();
    let result = ai_summary::Entity::update_many()
        .filter(ai_summary::Column::TenantId.eq(tenant_id.get_uuid()))
        .col_expr(
            ai_summary::Column::Status,
            Expr::value(SummaryStatus::Generating.to_string()),
        )
        .col_expr(ai_summary::Column::RequestedAt, Expr::value(now))
        .col_expr(ai_summary::Column::UpdatedAt, Expr::value(now))
        .col_expr(
            ai_summary::Column::ModifiedByUserId,
            Expr::value(actor.user_id()),
        )
        .col_expr(
            ai_summary::Column::ModifiedByActor,
            Expr::value(actor.kind_name()),
        )
        .filter(ai_summary::Column::EntityType.eq(entity_type.to_string()))
        .filter(ai_summary::Column::EntityId.eq(entity_id))
        .filter(
            Condition::any()
                .add(ai_summary::Column::Status.ne(SummaryStatus::Generating.to_string()))
                .add(ai_summary::Column::RequestedAt.lt(stale_before)),
        )
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}

pub async fn insert_generating(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    entity_type: SummaryEntityType,
    entity_id: Uuid,
    actor: &Actor,
) -> Result<ai_summary::Model, DbErr> {
    let now: DateTimeWithTimeZone = time_utils::now_utc_into();
    ai_summary::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id.get_uuid()),
        entity_type: Set(entity_type.to_string()),
        entity_id: Set(entity_id),
        status: Set(SummaryStatus::Generating.to_string()),
        summary_text: Set(None),
        error_message: Set(None),
        summarized_entity_count: Set(None),
        user_prompt: Set(None),
        user_prompt_used: Set(None),
        generated_at: Set(None),
        requested_at: Set(now),
        created_by_user_id: Set(actor.user_id()),
        created_by_actor: Set(actor.kind_name()),
        modified_by_user_id: Set(actor.user_id()),
        modified_by_actor: Set(actor.kind_name()),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await
}

pub async fn set_user_prompt(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    entity_type: SummaryEntityType,
    entity_id: Uuid,
    user_prompt: Option<String>,
    actor: &Actor,
) -> Result<ai_summary::Model, DbErr> {
    let now: DateTimeWithTimeZone = time_utils::now_utc_into();

    if let Some(row) = find(db, tenant_id, entity_type, entity_id).await? {
        let mut active: ai_summary::ActiveModel = row.into();
        active.user_prompt = Set(user_prompt);
        active.updated_at = Set(now);
        active.modified_by_user_id = Set(actor.user_id());
        active.modified_by_actor = Set(actor.kind_name());
        return active.update(db).await;
    }

    ai_summary::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id.get_uuid()),
        entity_type: Set(entity_type.to_string()),
        entity_id: Set(entity_id),
        status: Set(SummaryStatus::Idle.to_string()),
        summary_text: Set(None),
        error_message: Set(None),
        summarized_entity_count: Set(None),
        user_prompt: Set(user_prompt),
        user_prompt_used: Set(None),
        generated_at: Set(None),
        requested_at: Set(now),
        created_by_user_id: Set(actor.user_id()),
        created_by_actor: Set(actor.kind_name()),
        modified_by_user_id: Set(actor.user_id()),
        modified_by_actor: Set(actor.kind_name()),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await
}

pub struct ReadySummary {
    pub summary_text: String,
    pub summarized_entity_count: Option<i32>,
    pub user_prompt_used: Option<String>,
}

pub async fn mark_ready(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    entity_type: SummaryEntityType,
    entity_id: Uuid,
    ready_summary: ReadySummary,
    actor: &Actor,
) -> Result<Option<ai_summary::Model>, DbErr> {
    let Some(row) = find(db, tenant_id, entity_type, entity_id).await? else {
        return Ok(None);
    };
    let now: DateTimeWithTimeZone = time_utils::now_utc_into();
    let mut active: ai_summary::ActiveModel = row.into();
    active.status = Set(SummaryStatus::Ready.to_string());
    active.summary_text = Set(Some(ready_summary.summary_text));
    active.error_message = Set(None);
    active.summarized_entity_count = Set(ready_summary.summarized_entity_count);
    active.user_prompt_used = Set(ready_summary.user_prompt_used);
    active.generated_at = Set(Some(now));
    active.updated_at = Set(now);
    active.modified_by_user_id = Set(actor.user_id());
    active.modified_by_actor = Set(actor.kind_name());
    active.update(db).await.map(Some)
}

pub async fn mark_failed(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    entity_type: SummaryEntityType,
    entity_id: Uuid,
    error_message: String,
    actor: &Actor,
) -> Result<Option<ai_summary::Model>, DbErr> {
    let Some(row) = find(db, tenant_id, entity_type, entity_id).await? else {
        return Ok(None);
    };
    let now: DateTimeWithTimeZone = time_utils::now_utc_into();
    let mut active: ai_summary::ActiveModel = row.into();
    active.status = Set(SummaryStatus::Failed.to_string());
    active.error_message = Set(Some(error_message));
    active.updated_at = Set(now);
    active.modified_by_user_id = Set(actor.user_id());
    active.modified_by_actor = Set(actor.kind_name());
    active.update(db).await.map(Some)
}

pub async fn delete_for_entity(
    db: &impl ConnectionTrait,
    tenant_id: &TenantId,
    entity_type: SummaryEntityType,
    entity_id: Uuid,
) -> Result<(), DbErr> {
    ai_summary::Entity::delete_many()
        .filter(ai_summary::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(ai_summary::Column::EntityType.eq(entity_type.to_string()))
        .filter(ai_summary::Column::EntityId.eq(entity_id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn delete_for_entities(
    db: &impl ConnectionTrait,
    tenant_id: &TenantId,
    entity_type: SummaryEntityType,
    entity_ids: &[Uuid],
) -> Result<(), DbErr> {
    if entity_ids.is_empty() {
        return Ok(());
    }
    ai_summary::Entity::delete_many()
        .filter(ai_summary::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(ai_summary::Column::EntityType.eq(entity_type.to_string()))
        .filter(ai_summary::Column::EntityId.is_in(entity_ids.iter().copied()))
        .exec(db)
        .await?;
    Ok(())
}
