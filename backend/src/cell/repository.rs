use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::cell::constants::CellType;
use crate::cell::dto;
use crate::common::time_utils;
use crate::entity::cell;
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::Set;
use sea_orm::sea_query::{Expr, Func, SelectStatement};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    Order, PaginatorTrait,
};
use uuid::Uuid;

pub async fn save_cell_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    cell: cell::Model,
) -> Result<cell::Model, DbErr> {
    let mut cell_active_model: cell::ActiveModel = cell.into();
    cell_active_model.tenant_id = Set(tenant_id.get_uuid());
    cell_active_model.insert(txn).await
}

pub async fn get_cell(
    db: &impl ConnectionTrait,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<Option<cell::Model>, DbErr> {
    cell::Entity::find()
        .filter(cell::Column::Id.eq(id))
        .filter(cell::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(db)
        .await
}

pub async fn count_cells_by_notebook_id(
    db: &impl ConnectionTrait,
    tenant_id: &TenantId,
    notebook_id: Uuid,
) -> Result<u64, DbErr> {
    cell::Entity::find()
        .filter(cell::Column::NotebookId.eq(notebook_id))
        .filter(cell::Column::TenantId.eq(tenant_id.get_uuid()))
        .count(db)
        .await
}

pub async fn get_cells_by_ids(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    ids: &[Uuid],
) -> Result<Vec<cell::Model>, DbErr> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    cell::Entity::find()
        .filter(cell::Column::Id.is_in(ids.iter().copied()))
        .filter(cell::Column::TenantId.eq(tenant_id.get_uuid()))
        .all(db)
        .await
}

pub async fn get_cells_by_notebook_id(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    notebook_id: Uuid,
) -> Result<Vec<cell::Model>, DbErr> {
    let cells = cell::Entity::find()
        .filter(cell::Column::NotebookId.eq(notebook_id))
        .filter(cell::Column::TenantId.eq(tenant_id.get_uuid()))
        .order_by_asc(cell::Column::DisplayOrder)
        .all(db)
        .await?;
    Ok(cells)
}

pub async fn get_cells_by_notebook_id_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    notebook_id: Uuid,
) -> Result<Vec<cell::Model>, DbErr> {
    let cells = cell::Entity::find()
        .filter(cell::Column::NotebookId.eq(notebook_id))
        .filter(cell::Column::TenantId.eq(tenant_id.get_uuid()))
        .order_by_asc(cell::Column::DisplayOrder)
        .all(txn)
        .await?;
    Ok(cells)
}

pub async fn update_cell_content_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    cell_id: Uuid,
    name: Option<String>,
    content: Option<String>,
    actor: &Actor,
) -> Result<Option<cell::Model>, DbErr> {
    let Some(cell) = get_cell_transaction(txn, tenant_id, cell_id).await? else {
        return Ok(None);
    };
    let mut cell_entity: cell::ActiveModel = cell.into();
    if let Some(name) = name {
        cell_entity.name = Set(Some(name));
    }
    if let Some(content) = content {
        cell_entity.content = Set(Some(content));
    }
    cell_entity.updated_at = Set(time_utils::now_utc_into());
    cell_entity.modified_by_user_id = Set(actor.user_id());
    cell_entity.modified_by_actor = Set(actor.kind_name());
    cell_entity.update(txn).await.map(Some)
}

pub async fn update_cell_display_order_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    cell_id: Uuid,
    display_order: i32,
) -> Result<Option<cell::Model>, DbErr> {
    let Some(cell) = get_cell_transaction(txn, tenant_id, cell_id).await? else {
        return Ok(None);
    };
    let mut cell_entity: cell::ActiveModel = cell.into();
    cell_entity.display_order = Set(display_order);
    cell_entity.updated_at = Set(time_utils::now_utc_into());
    cell_entity.update(txn).await.map(Some)
}

pub async fn get_cells_by_notebook_id_and_cell_type(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    notebook_id: Uuid,
    cell_type: CellType,
) -> Result<Vec<cell::Model>, DbErr> {
    let condition = Condition::all()
        .add(cell::Column::TenantId.eq(tenant_id.get_uuid()))
        .add(cell::Column::NotebookId.eq(notebook_id))
        .add(cell::Column::CellType.eq(cell_type.to_string()));
    let cells = cell::Entity::find().filter(condition).all(db).await?;
    Ok(cells)
}

pub async fn count_cells_by_notebook_id_and_cell_type(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    notebook_id: Uuid,
    cell_type: CellType,
) -> Result<u64, DbErr> {
    let condition = Condition::all()
        .add(cell::Column::TenantId.eq(tenant_id.get_uuid()))
        .add(cell::Column::NotebookId.eq(notebook_id))
        .add(cell::Column::CellType.eq(cell_type.to_string()));
    let count = cell::Entity::find().filter(condition).count(db).await?;
    Ok(count)
}

pub async fn update_cell(
    db: &impl ConnectionTrait,
    tenant_id: &TenantId,
    cell_id: Uuid,
    payload: dto::CellDto,
    actor: &Actor,
) -> Result<Option<cell::Model>, DbErr> {
    let Some(cell) = get_cell(db, tenant_id, cell_id).await? else {
        return Ok(None);
    };
    let mut cell_entity: cell::ActiveModel = cell.into();
    cell_entity.cell_type = Set(payload.cell_type.to_string());
    cell_entity.notebook_id = Set(payload.notebook_id);
    cell_entity.connection_id = Set(payload.connection_id);
    cell_entity.name = Set(payload.name.clone());
    cell_entity.content = Set(payload.content.clone());
    cell_entity.status = Set(payload.status.to_string());
    cell_entity.updated_at = Set(time_utils::now_utc_into());
    cell_entity.modified_by_user_id = Set(actor.user_id());
    cell_entity.modified_by_actor = Set(actor.kind_name());
    cell_entity.created_at = Set(payload.created_at);
    cell_entity.update(db).await.map(Some)
}

pub async fn get_cell_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<Option<cell::Model>, DbErr> {
    cell::Entity::find()
        .filter(cell::Column::Id.eq(id))
        .filter(cell::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(txn)
        .await
}

pub async fn delete_cell_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<sea_orm::DeleteResult, DbErr> {
    cell::Entity::delete_many()
        .filter(cell::Column::Id.eq(id))
        .filter(cell::Column::TenantId.eq(tenant_id.get_uuid()))
        .exec(txn)
        .await
}

pub async fn delete_cells_by_notebook_id_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    notebook_id: Uuid,
) -> Result<(), DbErr> {
    cell::Entity::delete_many()
        .filter(cell::Column::NotebookId.eq(notebook_id))
        .filter(cell::Column::TenantId.eq(tenant_id.get_uuid()))
        .exec(txn)
        .await?;
    Ok(())
}

pub async fn search_cells_by_name_and_type(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    visible_notebook_ids: SelectStatement,
    search_query: &str,
    cell_type: CellType,
    limit: u64,
) -> Result<Vec<cell::Model>, DbErr> {
    let search_pattern = format!("%{}%", search_query.to_lowercase());
    let condition = Condition::all()
        .add(cell::Column::TenantId.eq(tenant_id.get_uuid()))
        .add(cell::Column::NotebookId.in_subquery(visible_notebook_ids))
        .add(cell::Column::Name.is_not_null())
        .add(Expr::expr(Func::lower(Expr::col(cell::Column::Name))).like(&search_pattern))
        .add(cell::Column::CellType.eq(cell_type.to_string()));
    let query = cell::Entity::find()
        .filter(condition)
        .order_by(cell::Column::UpdatedAt, Order::Desc);
    let paginator = query.paginate(db, limit);
    let cells = paginator.fetch_page(0).await?;
    Ok(cells)
}

pub async fn get_latest_cells_by_type(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    visible_notebook_ids: SelectStatement,
    cell_type: CellType,
    limit: u64,
) -> Result<Vec<cell::Model>, DbErr> {
    let condition = Condition::all()
        .add(cell::Column::TenantId.eq(tenant_id.get_uuid()))
        .add(cell::Column::NotebookId.in_subquery(visible_notebook_ids))
        .add(cell::Column::Name.is_not_null())
        .add(cell::Column::CellType.eq(cell_type.to_string()));
    let query = cell::Entity::find()
        .filter(condition)
        .order_by(cell::Column::UpdatedAt, Order::Desc);
    let paginator = query.paginate(db, limit);
    let cells = paginator.fetch_page(0).await?;
    Ok(cells)
}
