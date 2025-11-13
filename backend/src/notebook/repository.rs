use crate::entity::notebook;
use crate::notebook::dto;
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait};
use sea_orm::{DatabaseTransaction, Order, QueryOrder};
use uuid::Uuid;

pub async fn save_notebook(
    db: &DatabaseConnection,
    notebook: notebook::Model,
) -> Result<notebook::Model, DbErr> {
    let notebook_active_model: notebook::ActiveModel = notebook.into();
    let notebook_active_model = notebook_active_model.insert(db).await?;
    Ok(notebook_active_model.into())
}

pub async fn get_notebook(db: &DatabaseConnection, id: Uuid) -> Result<notebook::Model, DbErr> {
    let notebook = notebook::Entity::find_by_id(id).one(db).await?;
    match notebook {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Notebook not found".into())),
    }
}

pub async fn get_all_notebooks(db: &DatabaseConnection) -> Result<Vec<notebook::Model>, DbErr> {
    let notebooks = notebook::Entity::find().all(db).await?;
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

pub async fn update_notebook(
    db: &DatabaseConnection,
    txn: &DatabaseTransaction,
    notebook_id: Uuid,
    payload: &dto::NotebookDto,
) -> Result<notebook::Model, DbErr> {
    let notebook = get_notebook(&db, notebook_id).await;
    match notebook {
        Ok(notebook) => {
            let mut notebook_entity: notebook::ActiveModel = notebook.into();
            notebook_entity.name = Set(payload.name.clone());
            notebook_entity.description = Set(payload.description.clone());
            notebook_entity.status = Set(payload.status.to_string());
            notebook_entity.updated_at = Set(Utc::now().into());
            notebook_entity.created_at = Set(payload.created_at);

            let notebook_entity = notebook_entity.update(txn).await;
            return notebook_entity;
        }
        Err(e) => Err(e),
    }
}
