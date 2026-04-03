use crate::canvas::dto;
use crate::common::time_utils;
use crate::entity::canvas;
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait,
};
use sea_orm::{Order, QueryFilter, QueryOrder, Set};
use uuid::Uuid;

pub async fn get_canvas(db: &DatabaseConnection, id: Uuid) -> Result<canvas::Model, DbErr> {
    let canvas = canvas::Entity::find_by_id(id).one(db).await?;
    match canvas {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Canvas not found".into())),
    }
}

pub async fn get_paginated_canvases(
    db: &DatabaseConnection,
    page: u64,
    page_size: u64,
) -> Result<(Vec<canvas::Model>, u64, u64), DbErr> {
    let paginator = canvas::Entity::find()
        .order_by(canvas::Column::UpdatedAt, Order::Desc)
        .paginate(db, page_size);
    let total_items = paginator.num_items().await?;
    let canvases = paginator.fetch_page(page).await?;
    let total_pages = paginator.num_pages().await?;
    Ok((canvases, total_items, total_pages))
}

pub async fn save_canvas_transaction(
    txn: &DatabaseTransaction,
    canvas: canvas::Model,
) -> Result<canvas::Model, DbErr> {
    let canvas_active_model: canvas::ActiveModel = canvas.into();
    let canvas_active_model = canvas_active_model.insert(txn).await?;
    Ok(canvas_active_model)
}

pub async fn save_canvas_connection(
    db: &DatabaseConnection,
    canvas: canvas::Model,
) -> Result<canvas::Model, DbErr> {
    let canvas_active_model: canvas::ActiveModel = canvas.into();
    let canvas_active_model = canvas_active_model.insert(db).await?;
    Ok(canvas_active_model)
}

pub async fn delete_canvas_transaction(txn: &DatabaseTransaction, id: Uuid) -> Result<(), DbErr> {
    canvas::Entity::delete_by_id(id).exec(txn).await?;
    Ok(())
}

pub async fn update_canvas(
    db: &DatabaseConnection,
    id: Uuid,
    payload: dto::CreateCanvasDto,
) -> Result<canvas::Model, DbErr> {
    let canvas = get_canvas(db, id).await?;
    let mut canvas_active: canvas::ActiveModel = canvas.into();
    canvas_active.name = Set(payload.name);
    canvas_active.description = Set(payload.description);
    canvas_active.updated_at = Set(time_utils::now_utc_into());
    let result = canvas_active.update(db).await?;
    Ok(result)
}

pub async fn search_canvases_by_name(
    db: &DatabaseConnection,
    search_query: &str,
    limit: u64,
) -> Result<Vec<canvas::Model>, DbErr> {
    let search_pattern = format!("%{}%", search_query.to_lowercase());
    let query = canvas::Entity::find()
        .filter(Expr::expr(Func::lower(Expr::col(canvas::Column::Name))).like(&search_pattern))
        .order_by(canvas::Column::UpdatedAt, Order::Desc);
    let paginator = query.paginate(db, limit);
    let results = paginator.fetch_page(0).await?;
    Ok(results)
}
