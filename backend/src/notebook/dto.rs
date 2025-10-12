use uuid::Uuid;
use chrono::{DateTime, FixedOffset};
use serde::{Serialize, Deserialize};
use crate::entity;
use crate::cell::dto::CellDto;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNotebookDto {
    pub name: String,
    pub description: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct NotebookDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub cells: Vec<CellDto>,
}

impl From<entity::notebook::Model> for NotebookDto {
    fn from(model: entity::notebook::Model) -> Self {
        NotebookDto {
            id: model.id,
            name: model.name,
            description: model.description,
            status: model.status,
            created_at: model.created_at,
            updated_at: model.updated_at,
            cells: vec![],
        }
    }
}
