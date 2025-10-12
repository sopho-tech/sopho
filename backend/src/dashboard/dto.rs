use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::entity;
use crate::dashboard::constants::DashboardStatus;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDashboardDto {
    pub name: String,
    pub description: String,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct DashboardDto {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub title: String,
    #[serde(serialize_with = "DashboardStatus::serialize_to_str")]
    pub status: DashboardStatus,
}

impl From<entity::dashboard::Model> for DashboardDto {
    fn from(model: entity::dashboard::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            title: model.title,
            status: DashboardStatus::from_str(&model.status).unwrap(),
        }
    }
}
