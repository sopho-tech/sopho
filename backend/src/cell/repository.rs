use sea_orm::{ActiveModelTrait, EntityTrait, DatabaseConnection, DbErr};
use crate::entity::cell;
use uuid::Uuid;
use sea_orm::Set;
use chrono::Utc;
use crate::cell::dto;
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;


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

pub async fn get_cells_by_notebook_id(db: &DatabaseConnection, notebook_id: Uuid) -> Result<Vec<cell::Model>, DbErr> {
    let cells = cell::Entity::find().filter(cell::Column::NotebookId.eq(notebook_id)).all(db).await?;
    Ok(cells)
}

pub async fn update_cell(db: &DatabaseConnection, cell_id: Uuid, payload: dto::CellDto) -> Result<cell::Model, DbErr> {
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
            cell_entity.updated_at = Set(Utc::now().into());
            cell_entity.created_at = Set(payload.created_at);

            let cell_entity = cell_entity.update(db).await;
            return cell_entity;
        }
        Err(e) => Err(e),
    }
}
