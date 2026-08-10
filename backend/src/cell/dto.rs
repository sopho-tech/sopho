use crate::cell::constants::AxisMinorTickShow;
use crate::cell::constants::AxisTickShow;
use crate::cell::constants::CellDisplayOrderMovement;
use crate::cell::constants::CellStatus;
use crate::cell::constants::CellType;
use crate::cell::constants::ChartOrientation;
use crate::cell::constants::ChartType;
use crate::cell::constants::MetricFormat;
use crate::cell::constants::SortOrder;
use crate::common::errors::ExecuteChartError;
use crate::common::errors::SophoError;
use crate::database::constants::QueryResult;
use crate::entity;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "chart_type")]
pub enum ChartContent {
    #[serde(rename = "BAR")]
    Bar(AxisChartContent),
    #[serde(rename = "LINE")]
    Line(AxisChartContent),
    #[serde(rename = "PIE")]
    Pie(PieChartContent),
    #[serde(rename = "METRIC")]
    Metric(MetricChartContent),
}

impl ChartContent {
    pub fn cell_id(&self) -> Uuid {
        match self {
            ChartContent::Bar(c) | ChartContent::Line(c) => c.cell_id,
            ChartContent::Pie(c) => c.cell_id,
            ChartContent::Metric(c) => c.cell_id,
        }
    }

    pub fn chart_type(&self) -> ChartType {
        match self {
            ChartContent::Bar(_) => ChartType::Bar,
            ChartContent::Line(_) => ChartType::Line,
            ChartContent::Pie(_) => ChartType::Pie,
            ChartContent::Metric(_) => ChartType::Metric,
        }
    }

    pub fn field_names(&self) -> Vec<String> {
        match self {
            ChartContent::Bar(c) | ChartContent::Line(c) => {
                vec![c.x_axis.clone(), c.y_axis.clone()]
            }
            ChartContent::Pie(c) => vec![c.category.clone(), c.value.clone()],
            ChartContent::Metric(_) => Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct ChartExecution {
    pub cell_id: Uuid,
    pub chart_name: String,
    pub chart_type: String,
    pub field_names: Vec<String>,
    pub result: QueryResult,
}

#[derive(Debug)]
pub struct ChartExecutionOutcome {
    pub cell_id: Uuid,
    pub chart_name: String,
    pub result: Result<ChartExecution, ExecuteChartError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricChartContent {
    pub cell_id: Uuid,
    pub decimal_precision: Option<u32>,
    pub suffix: Option<String>,
    #[serde(deserialize_with = "MetricFormat::deserialize_option_from_str")]
    #[serde(serialize_with = "MetricFormat::serialize_option_to_str")]
    pub format: Option<MetricFormat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AxisChartContent {
    pub cell_id: Uuid,
    pub x_axis: String,
    pub y_axis: String,
    #[serde(deserialize_with = "ChartOrientation::deserialize_option_from_str")]
    #[serde(serialize_with = "ChartOrientation::serialize_option_to_str")]
    pub orientation: Option<ChartOrientation>,
    pub y_axis_aggregate_function: Option<String>,
    #[serde(deserialize_with = "SortOrder::deserialize_option_from_str")]
    #[serde(serialize_with = "SortOrder::serialize_option_to_str")]
    pub y_axis_sort_order: Option<SortOrder>,
    #[serde(deserialize_with = "AxisTickShow::deserialize_option_from_str")]
    #[serde(serialize_with = "AxisTickShow::serialize_option_to_str")]
    pub x_axis_tick_show: Option<AxisTickShow>,
    #[serde(deserialize_with = "AxisTickShow::deserialize_option_from_str")]
    #[serde(serialize_with = "AxisTickShow::serialize_option_to_str")]
    pub y_axis_tick_show: Option<AxisTickShow>,
    #[serde(deserialize_with = "AxisMinorTickShow::deserialize_option_from_str")]
    #[serde(serialize_with = "AxisMinorTickShow::serialize_option_to_str")]
    pub axis_minor_tick_show: Option<AxisMinorTickShow>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PieChartContent {
    pub cell_id: Uuid,
    pub category: String,
    pub value: String,
    pub aggregate_function: Option<String>,
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
pub struct ExecuteCellPreviewDto {
    pub content: String,
    #[serde(deserialize_with = "CellType::deserialize_from_str")]
    pub cell_type: CellType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReorderCellDto {
    #[serde(deserialize_with = "CellDisplayOrderMovement::deserialize_from_str")]
    #[serde(serialize_with = "CellDisplayOrderMovement::serialize_to_str")]
    pub movement_type: CellDisplayOrderMovement,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCellDto {
    pub notebook_id: Uuid,
    pub connection_id: Option<Uuid>,
    pub name: Option<String>,
    pub content: Option<String>,
    pub display_order: Option<i32>,
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
