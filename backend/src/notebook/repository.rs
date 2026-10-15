use crate::entity::notebook;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait};
use sea_orm::{ColumnTrait, DatabaseTransaction, QueryFilter, QueryOrder};
use uuid::Uuid;

pub async fn save_notebook_connection(
    db: &DatabaseConnection,
    notebook: notebook::Model,
) -> Result<notebook::Model, DbErr> {
    let notebook_active_model: notebook::ActiveModel = notebook.into();
    let notebook_active_model = notebook_active_model.insert(db).await?;
    Ok(notebook_active_model)
}

pub async fn save_notebook_transaction(
    txn: &DatabaseTransaction,
    notebook: notebook::Model,
) -> Result<notebook::Model, DbErr> {
    let notebook_active_model: notebook::ActiveModel = notebook.into();
    let notebook_active_model = notebook_active_model.insert(txn).await?;
    Ok(notebook_active_model)
}

pub async fn get_notebook(db: &DatabaseConnection, id: Uuid) -> Result<notebook::Model, DbErr> {
    let notebook = notebook::Entity::find_by_id(id).one(db).await?;
    match notebook {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Notebook not found".into())),
    }
}

pub async fn get_notebooks_by_ids(
    db: &DatabaseConnection,
    ids: Vec<Uuid>,
) -> Result<Vec<notebook::Model>, DbErr> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let notebooks = notebook::Entity::find()
        .filter(notebook::Column::Id.is_in(ids))
        .all(db)
        .await?;
    Ok(notebooks)
}

pub async fn get_paginated_notebooks(
    db: &DatabaseConnection,
    page: u64,
    page_size: u64,
) -> Result<(Vec<notebook::Model>, u64, u64), DbErr> {
    let paginator = notebook::Entity::find()
        .order_by(notebook::Column::CreatedAt, sea_orm::Order::Desc)
        .paginate(db, page_size);
    let total_items = paginator.num_items().await?;
    let notebooks = paginator.fetch_page(page).await?;
    let total_pages = paginator.num_pages().await?;
    Ok((notebooks, total_items, total_pages))
}

pub async fn get_notebook_transaction(
    txn: &DatabaseTransaction,
    id: Uuid,
) -> Result<notebook::Model, DbErr> {
    let notebook = notebook::Entity::find_by_id(id).one(txn).await?;
    match notebook {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Notebook not found".into())),
    }
}

pub async fn get_notebook_by_canvas_id(
    db: &DatabaseConnection,
    canvas_id: Uuid,
) -> Result<notebook::Model, DbErr> {
    let notebook = notebook::Entity::find()
        .filter(notebook::Column::CanvasId.eq(canvas_id))
        .one(db)
        .await?;
    match notebook {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Notebook not found".into())),
    }
}

pub async fn get_notebooks_by_canvas_id_connection(
    db: &DatabaseConnection,
    canvas_id: Uuid,
) -> Result<Vec<notebook::Model>, DbErr> {
    let notebooks = notebook::Entity::find()
        .filter(notebook::Column::CanvasId.eq(canvas_id))
        .all(db)
        .await?;
    Ok(notebooks)
}

pub async fn get_notebook_by_canvas_id_transaction(
    txn: &DatabaseTransaction,
    canvas_id: Uuid,
) -> Result<notebook::Model, DbErr> {
    let notebook = notebook::Entity::find()
        .filter(notebook::Column::CanvasId.eq(canvas_id))
        .one(txn)
        .await?;
    match notebook {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Notebook not found".into())),
    }
}

pub async fn delete_notebook_transaction(txn: &DatabaseTransaction, id: Uuid) -> Result<(), DbErr> {
    notebook::Entity::delete_by_id(id).exec(txn).await?;
    Ok(())
}
