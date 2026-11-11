use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::canvas::dto;
use crate::common::time_utils;
use crate::entity::canvas;
use sea_orm::Condition;
use sea_orm::sea_query::{Expr, Func, Query, SelectStatement};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, PaginatorTrait,
};
use sea_orm::{Order, QueryFilter, QueryOrder, Set};
use uuid::Uuid;

pub async fn get_canvas(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<Option<canvas::Model>, DbErr> {
    canvas::Entity::find()
        .filter(canvas::Column::Id.eq(id))
        .filter(canvas::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(db)
        .await
}

pub async fn get_canvas_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<Option<canvas::Model>, DbErr> {
    canvas::Entity::find()
        .filter(canvas::Column::Id.eq(id))
        .filter(canvas::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(txn)
        .await
}

pub async fn update_canvas_transaction(
    txn: &DatabaseTransaction,
    canvas: canvas::Model,
    name: String,
    description: Option<String>,
    actor: &Actor,
) -> Result<canvas::Model, DbErr> {
    let mut canvas_active: canvas::ActiveModel = canvas.into();
    canvas_active.name = Set(name);
    canvas_active.description = Set(description);
    canvas_active.updated_at = Set(time_utils::now_utc_into());
    canvas_active.modified_by_user_id = Set(actor.user_id());
    canvas_active.modified_by_actor = Set(actor.kind_name());
    canvas_active.update(txn).await
}

pub async fn save_canvas_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    canvas: canvas::Model,
) -> Result<canvas::Model, DbErr> {
    let mut canvas_active_model: canvas::ActiveModel = canvas.into();
    canvas_active_model.tenant_id = Set(tenant_id.get_uuid());
    canvas_active_model.insert(txn).await
}

pub async fn delete_canvas_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<(), DbErr> {
    canvas::Entity::delete_many()
        .filter(canvas::Column::Id.eq(id))
        .filter(canvas::Column::TenantId.eq(tenant_id.get_uuid()))
        .exec(txn)
        .await?;
    Ok(())
}

pub async fn update_canvas(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    id: Uuid,
    payload: dto::CreateCanvasDto,
    actor: &Actor,
) -> Result<Option<canvas::Model>, DbErr> {
    let Some(canvas) = get_canvas(db, tenant_id, id).await? else {
        return Ok(None);
    };
    let mut canvas_active: canvas::ActiveModel = canvas.into();
    canvas_active.name = Set(payload.name);
    canvas_active.description = Set(payload.description);
    canvas_active.updated_at = Set(time_utils::now_utc_into());
    canvas_active.modified_by_user_id = Set(actor.user_id());
    canvas_active.modified_by_actor = Set(actor.kind_name());
    let result = canvas_active.update(db).await?;
    Ok(Some(result))
}

pub async fn search_canvases_by_name(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    reachable: Condition,
    search_query: &str,
    limit: u64,
) -> Result<Vec<canvas::Model>, DbErr> {
    let search_pattern = format!("%{}%", search_query.to_lowercase());
    let query = canvas::Entity::find()
        .filter(canvas::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(reachable)
        .filter(Expr::expr(Func::lower(Expr::col(canvas::Column::Name))).like(&search_pattern))
        .order_by(canvas::Column::UpdatedAt, Order::Desc);
    let paginator = query.paginate(db, limit);
    let results = paginator.fetch_page(0).await?;
    Ok(results)
}

pub async fn get_paginated_visible_canvases(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    reachable: Condition,
    created_by_user_id: Option<Uuid>,
    page: u64,
    page_size: u64,
) -> Result<(Vec<canvas::Model>, u64, u64), DbErr> {
    let mut query = canvas::Entity::find()
        .filter(canvas::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(reachable);
    if let Some(created_by_user_id) = created_by_user_id {
        query = query.filter(canvas::Column::CreatedByUserId.eq(created_by_user_id));
    }
    let paginator = query
        .order_by(canvas::Column::UpdatedAt, Order::Desc)
        .paginate(db, page_size);
    let total_items = paginator.num_items().await?;
    let canvases = paginator.fetch_page(page).await?;
    let total_pages = paginator.num_pages().await?;
    Ok((canvases, total_items, total_pages))
}

pub fn visible_canvas_ids_subquery(tenant_id: &TenantId, reachable: Condition) -> SelectStatement {
    Query::select()
        .column(canvas::Column::Id)
        .from(canvas::Entity)
        .cond_where(
            Condition::all()
                .add(Expr::col(canvas::Column::TenantId).eq(tenant_id.get_uuid()))
                .add(reachable),
        )
        .to_owned()
}

pub async fn update_canvas_visibility(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    id: Uuid,
    visibility: &str,
    actor: &Actor,
) -> Result<u64, DbErr> {
    let result = canvas::Entity::update_many()
        .col_expr(
            canvas::Column::Visibility,
            sea_orm::sea_query::Expr::value(visibility.to_string()),
        )
        .col_expr(
            canvas::Column::UpdatedAt,
            sea_orm::sea_query::Expr::value(time_utils::now_utc_into::<
                sea_orm::prelude::DateTimeWithTimeZone,
            >()),
        )
        .col_expr(
            canvas::Column::ModifiedByUserId,
            sea_orm::sea_query::Expr::value(actor.user_id()),
        )
        .col_expr(
            canvas::Column::ModifiedByActor,
            sea_orm::sea_query::Expr::value(actor.kind_name()),
        )
        .filter(canvas::Column::Id.eq(id))
        .filter(canvas::Column::TenantId.eq(tenant_id.get_uuid()))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}

pub async fn stamp_modified(
    connection: &impl ConnectionTrait,
    tenant_id: &TenantId,
    id: Uuid,
    actor: &Actor,
) -> Result<u64, DbErr> {
    let result = canvas::Entity::update_many()
        .col_expr(
            canvas::Column::UpdatedAt,
            Expr::value(time_utils::now_utc_into::<
                sea_orm::prelude::DateTimeWithTimeZone,
            >()),
        )
        .col_expr(
            canvas::Column::ModifiedByUserId,
            Expr::value(actor.user_id()),
        )
        .col_expr(
            canvas::Column::ModifiedByActor,
            Expr::value(actor.kind_name()),
        )
        .filter(canvas::Column::Id.eq(id))
        .filter(canvas::Column::TenantId.eq(tenant_id.get_uuid()))
        .exec(connection)
        .await?;
    Ok(result.rows_affected)
}
