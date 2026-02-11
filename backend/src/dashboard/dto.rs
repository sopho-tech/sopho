use crate::dashboard::constants::DashboardStatus;
use crate::entity;
use sea_orm::entity::prelude::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDashboardDto {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Layout {
    cell_id: Uuid,
    notebook_id: Uuid,
    x_position: u16,
    y_position: u16,
    x_size: u16,
    y_size: u16,
}

impl Layout {
    pub fn cell_id(&self) -> Uuid {
        self.cell_id
    }

    pub fn to_json(layout: Option<Vec<Layout>>) -> Option<Json> {
        layout.map(|layouts| {
            let value = serde_json::to_value(layouts).unwrap_or(serde_json::Value::Null);
            Json::from(value)
        })
    }

    pub fn from_json(json: Option<Json>) -> Option<Vec<Layout>> {
        json.and_then(|j| {
            let value: serde_json::Value = j.into();
            serde_json::from_value(value).ok()
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardDto {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub layout: Option<Vec<Layout>>,
    #[serde(deserialize_with = "DashboardStatus::deserialize_from_str")]
    #[serde(serialize_with = "DashboardStatus::serialize_to_str")]
    pub status: DashboardStatus,
}

impl From<entity::dashboard::Model> for DashboardDto {
    fn from(model: entity::dashboard::Model) -> Self {
        let layout = Layout::from_json(model.layout);
        Self {
            id: model.id,
            name: Some(model.name),
            description: Some(model.description),
            status: DashboardStatus::from_str(&model.status).unwrap(),
            layout,
        }
    }
}
