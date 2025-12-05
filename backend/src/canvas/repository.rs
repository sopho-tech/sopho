use crate::entity::canvas;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait,
};
use sea_orm::{Order, QueryOrder};
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
        .order_by(canvas::Column::CreatedAt, Order::Desc)
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
    Ok(canvas_active_model.into())
}

pub async fn save_canvas_connection(
    db: &DatabaseConnection,
    canvas: canvas::Model,
) -> Result<canvas::Model, DbErr> {
    let canvas_active_model: canvas::ActiveModel = canvas.into();
    let canvas_active_model = canvas_active_model.insert(db).await?;
    Ok(canvas_active_model.into())
}

pub async fn delete_canvas_transaction(txn: &DatabaseTransaction, id: Uuid) -> Result<(), DbErr> {
    canvas::Entity::delete_by_id(id).exec(txn).await?;
    Ok(())
}
