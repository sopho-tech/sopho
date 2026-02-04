use crate::canvas::constants::CanvasStatus;
use crate::entity;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCanvasDto {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CanvasDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(serialize_with = "CanvasStatus::serialize_to_str")]
    pub status: CanvasStatus,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub sql_cell_count: i32,
    pub chart_cell_count: i32,
    pub dashboard_charts_count: i32,
}

impl From<entity::canvas::Model> for CanvasDto {
    fn from(model: entity::canvas::Model) -> Self {
        CanvasDto {
            id: model.id,
            name: model.name,
            description: model.description,
            status: CanvasStatus::from_str(&model.status).unwrap(),
            created_at: model.created_at,
            updated_at: model.updated_at,
            sql_cell_count: 0,
            chart_cell_count: 0,
            dashboard_charts_count: 0,
        }
    }
}
