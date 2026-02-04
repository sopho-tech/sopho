use crate::cell::constants::CellType;
use crate::cell::dto;
use crate::common::time_utils;
use crate::entity::cell;
use sea_orm::sea_query::{Expr, Func};
use sea_orm::ColumnTrait;
use sea_orm::Condition;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::Set;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, Order,
    PaginatorTrait,
};
use uuid::Uuid;

pub async fn save_cell(db: &DatabaseConnection, cell: cell::Model) -> Result<cell::Model, DbErr> {
    let cell_active_model: cell::ActiveModel = cell.into();
    let cell_active_model = cell_active_model.insert(db).await?;
    Ok(cell_active_model.into())
}

pub async fn get_cell(db: &DatabaseConnection, id: Uuid) -> Result<cell::Model, DbErr> {
    let cell = cell::Entity::find_by_id(id).one(db).await?;
    match cell {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Cell not found".into())),
    }
}

pub async fn get_cells_by_notebook_id(
    db: &DatabaseConnection,
    notebook_id: Uuid,
) -> Result<Vec<cell::Model>, DbErr> {
    let cells = cell::Entity::find()
        .filter(cell::Column::NotebookId.eq(notebook_id))
        .all(db)
        .await?;
    Ok(cells)
}

pub async fn get_cells_by_notebook_id_and_cell_type(
    db: &DatabaseConnection,
    notebook_id: Uuid,
    cell_type: CellType,
) -> Result<Vec<cell::Model>, DbErr> {
    let condition = Condition::all()
        .add(cell::Column::NotebookId.eq(notebook_id))
        .add(cell::Column::CellType.eq(cell_type.to_string()));
    let cells = cell::Entity::find().filter(condition).all(db).await?;
    Ok(cells)
}

pub async fn count_cells_by_notebook_id_and_cell_type(
    db: &DatabaseConnection,
    notebook_id: Uuid,
    cell_type: CellType,
) -> Result<u64, DbErr> {
    let condition = Condition::all()
        .add(cell::Column::NotebookId.eq(notebook_id))
        .add(cell::Column::CellType.eq(cell_type.to_string()));
    let count = cell::Entity::find().filter(condition).count(db).await?;
    Ok(count)
}

pub async fn update_cell(
    db: &DatabaseConnection,
    cell_id: Uuid,
    payload: dto::CellDto,
) -> Result<cell::Model, DbErr> {
    let cell = get_cell(&db, cell_id).await;
    match cell {
        Ok(cell) => {
            let mut cell_entity: cell::ActiveModel = cell.into();
            cell_entity.cell_type = Set(payload.cell_type.to_string());
            cell_entity.notebook_id = Set(payload.notebook_id);
            cell_entity.connection_id = Set(payload.connection_id);
            cell_entity.name = Set(payload.name.clone());
            cell_entity.content = Set(payload.content.clone());
            cell_entity.status = Set(payload.status.to_string());
            cell_entity.updated_at = Set(time_utils::now_utc_into());
            cell_entity.created_at = Set(payload.created_at);

            let cell_entity = cell_entity.update(db).await;
            return cell_entity;
        }
        Err(e) => Err(e),
    }
}

pub async fn delete_cells_by_notebook_id_transaction(
    txn: &DatabaseTransaction,
    notebook_id: Uuid,
) -> Result<(), DbErr> {
    cell::Entity::delete_many()
        .filter(cell::Column::NotebookId.eq(notebook_id))
        .exec(txn)
        .await?;
    Ok(())
}

pub async fn search_cells_by_name_and_type(
    db: &DatabaseConnection,
    search_query: &str,
    cell_type: CellType,
    limit: u64,
) -> Result<Vec<cell::Model>, DbErr> {
    let search_pattern = format!("%{}%", search_query.to_lowercase());
    let condition = Condition::all()
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
    cell_type: CellType,
    limit: u64,
) -> Result<Vec<cell::Model>, DbErr> {
    let condition = Condition::all()
        .add(cell::Column::Name.is_not_null())
        .add(cell::Column::CellType.eq(cell_type.to_string()));
    let query = cell::Entity::find()
        .filter(condition)
        .order_by(cell::Column::UpdatedAt, Order::Desc);
    let paginator = query.paginate(db, limit);
    let cells = paginator.fetch_page(0).await?;
    Ok(cells)
}
