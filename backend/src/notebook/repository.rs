use sea_orm::{ActiveModelTrait, EntityTrait, DatabaseConnection, DbErr};
use crate::entity::notebook;
use uuid::Uuid;
use sea_orm::ActiveValue::Set;
use chrono::Utc;
use crate::notebook::dto;
use sea_orm::DatabaseTransaction;


pub async fn save_notebook(db: &DatabaseConnection, notebook: notebook::Model) -> Result<notebook::Model, DbErr> {
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

pub async fn update_notebook(db: &DatabaseConnection, txn: &DatabaseTransaction, notebook_id: Uuid, payload: &dto::NotebookDto) -> Result<notebook::Model, DbErr> {
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
