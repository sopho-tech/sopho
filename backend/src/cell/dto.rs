use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::entity;
use crate::cell::constants::CellType;
use crate::cell::constants::CellStatus;
use crate::common::errors::SophoError;
use chrono::{DateTime, FixedOffset};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChartContent {
    pub x_axis: String,
    pub y_axis: String,
    pub chart_type: Option<String>,
    pub cell_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SqlContent {
    pub query: String,
    pub parameters: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CellContent {
    Chart(ChartContent),
    Sql(SqlContent),
}

impl CellContent {
    pub fn parse(content: &str, cell_type: &CellType) -> Result<Self, SophoError> {
        match cell_type {
            CellType::Chart => {
                let chart_content: ChartContent = serde_json::from_str(content)?;
                Ok(CellContent::Chart(chart_content))
            }
            CellType::Sql => {
                let sql_content: SqlContent = serde_json::from_str(content)?;
                Ok(CellContent::Sql(sql_content))
            }
            _ => Err(SophoError::UnsupportedCellType),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCellDto {
    pub notebook_id: Uuid,
    pub connection_id: Option<Uuid>,
    pub name: Option<String>,
    pub content: Option<String>,
    #[serde(deserialize_with = "CellType::deserialize_from_str")]
    pub cell_type: CellType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CellDto {
    pub id: Uuid,
    pub name: Option<String>,
    pub content: Option<String>,
    #[serde(deserialize_with = "CellType::deserialize_from_str")]
    #[serde(serialize_with = "CellType::serialize_to_str")]
    pub cell_type: CellType,
    pub notebook_id: Uuid,
    pub connection_id: Option<Uuid>,
    pub display_order: i32,
    #[serde(deserialize_with = "CellStatus::deserialize_from_str")]
    #[serde(serialize_with = "CellStatus::serialize_to_str")]
    pub status: CellStatus,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<entity::cell::Model> for CellDto {
    fn from(model: entity::cell::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            content: model.content,
            cell_type: CellType::from_str(&model.cell_type).unwrap(),
            notebook_id: model.notebook_id,
            connection_id: model.connection_id,
            display_order: model.display_order,
            status: CellStatus::from_str(&model.status).unwrap(),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

impl CellDto {
    pub fn parse_content(&self) -> Result<Option<CellContent>, SophoError> {
        if let Some(content) = &self.content {
            CellContent::parse(content, &self.cell_type).map(Some)
        } else {
            Ok(None)
        }
    }
}
