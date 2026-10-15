use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::common::time_utils;
use crate::entity::notebook;
use crate::notebook::dto;
use sea_orm::sea_query::{Expr, Query, SelectStatement};
use sea_orm::ActiveValue::Set;
use sea_orm::Condition;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait};
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseTransaction, QueryFilter, QueryOrder};
use uuid::Uuid;

pub async fn save_notebook_connection(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    notebook: notebook::Model,
) -> Result<notebook::Model, DbErr> {
    let mut notebook_active_model: notebook::ActiveModel = notebook.into();
    notebook_active_model.tenant_id = Set(tenant_id.get_uuid());
    notebook_active_model.insert(db).await
}

pub async fn save_notebook_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    notebook: notebook::Model,
) -> Result<notebook::Model, DbErr> {
    let mut notebook_active_model: notebook::ActiveModel = notebook.into();
    notebook_active_model.tenant_id = Set(tenant_id.get_uuid());
    notebook_active_model.insert(txn).await
}

pub async fn get_notebook(
    db: &impl ConnectionTrait,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<Option<notebook::Model>, DbErr> {
    notebook::Entity::find()
        .filter(notebook::Column::Id.eq(id))
        .filter(notebook::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(db)
        .await
}

pub async fn get_notebooks_by_ids(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    ids: Vec<Uuid>,
) -> Result<Vec<notebook::Model>, DbErr> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let notebooks = notebook::Entity::find()
        .filter(notebook::Column::Id.is_in(ids))
        .filter(notebook::Column::TenantId.eq(tenant_id.get_uuid()))
        .all(db)
        .await?;
    Ok(notebooks)
}

pub async fn get_all_notebooks(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
) -> Result<Vec<notebook::Model>, DbErr> {
    let notebooks = notebook::Entity::find()
        .filter(notebook::Column::TenantId.eq(tenant_id.get_uuid()))
        .all(db)
        .await?;
    Ok(notebooks)
}

pub fn visible_notebook_ids_subquery(
    tenant_id: &TenantId,
    visible_canvas_ids: SelectStatement,
) -> SelectStatement {
    Query::select()
        .column(notebook::Column::Id)
        .from(notebook::Entity)
        .cond_where(
            Condition::all()
                .add(Expr::col(notebook::Column::TenantId).eq(tenant_id.get_uuid()))
                .add(Expr::col(notebook::Column::CanvasId).in_subquery(visible_canvas_ids)),
        )
        .to_owned()
}

pub async fn get_paginated_visible_notebooks(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    visible_canvas_ids: SelectStatement,
    page: u64,
    page_size: u64,
) -> Result<(Vec<notebook::Model>, u64, u64), DbErr> {
    let paginator = notebook::Entity::find()
        .filter(notebook::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(notebook::Column::CanvasId.in_subquery(visible_canvas_ids))
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
    tenant_id: &TenantId,
    notebook_id: Uuid,
    payload: &dto::NotebookDto,
    actor: &Actor,
) -> Result<Option<notebook::Model>, DbErr> {
    let Some(notebook) = get_notebook(db, tenant_id, notebook_id).await? else {
        return Ok(None);
    };
    let mut notebook_entity: notebook::ActiveModel = notebook.into();
    notebook_entity.name = Set(payload.name.clone());
    notebook_entity.description = Set(payload.description.clone());
    notebook_entity.status = Set(payload.status.to_string());
    notebook_entity.updated_at = Set(time_utils::now_utc_into());
    notebook_entity.modified_by_user_id = Set(actor.user_id());
    notebook_entity.modified_by_actor = Set(actor.kind_name());
    notebook_entity.created_at = Set(payload.created_at);

    notebook_entity.update(txn).await.map(Some)
}

pub async fn get_notebook_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<Option<notebook::Model>, DbErr> {
    notebook::Entity::find()
        .filter(notebook::Column::Id.eq(id))
        .filter(notebook::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(txn)
        .await
}

pub async fn get_notebook_by_canvas_id(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    canvas_id: Uuid,
) -> Result<Option<notebook::Model>, DbErr> {
    notebook::Entity::find()
        .filter(notebook::Column::CanvasId.eq(canvas_id))
        .filter(notebook::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(db)
        .await
}

pub async fn get_notebooks_by_canvas_id_connection(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    canvas_id: Uuid,
) -> Result<Vec<notebook::Model>, DbErr> {
    let notebooks = notebook::Entity::find()
        .filter(notebook::Column::CanvasId.eq(canvas_id))
        .filter(notebook::Column::TenantId.eq(tenant_id.get_uuid()))
        .all(db)
        .await?;
    Ok(notebooks)
}

pub async fn get_notebook_by_canvas_id_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    canvas_id: Uuid,
) -> Result<Option<notebook::Model>, DbErr> {
    notebook::Entity::find()
        .filter(notebook::Column::CanvasId.eq(canvas_id))
        .filter(notebook::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(txn)
        .await
}

pub async fn delete_notebook_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<(), DbErr> {
    notebook::Entity::delete_many()
        .filter(notebook::Column::Id.eq(id))
        .filter(notebook::Column::TenantId.eq(tenant_id.get_uuid()))
        .exec(txn)
        .await?;
    Ok(())
}
