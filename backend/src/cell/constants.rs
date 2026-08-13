use serde::{Deserialize, Serialize};
use std::fmt;

pub const MAX_CELLS_PER_NOTEBOOK: usize = 60;
pub const MAX_CHART_SERIES: usize = 6;
pub const MAX_IDENTIFIER_LEN: usize = 128;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CellType {
    Text,
    Code,
    Markdown,
    Sql,
    Chart,
}

impl fmt::Display for CellType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CellType::Text => "TEXT",
            CellType::Code => "CODE",
            CellType::Markdown => "MARKDOWN",
            CellType::Sql => "SQL",
            CellType::Chart => "CHART",
        })
    }
}

impl CellType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "TEXT" => Ok(CellType::Text),
            "CODE" => Ok(CellType::Code),
            "MARKDOWN" => Ok(CellType::Markdown),
            "SQL" => Ok(CellType::Sql),
            "CHART" => Ok(CellType::Chart),
            _ => Err(format!("Invalid cell type: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CellStatus {
    Active,
    Inactive,
}

impl CellStatus {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ACTIVE" => Ok(CellStatus::Active),
            "INACTIVE" => Ok(CellStatus::Inactive),
            _ => Err(format!("Invalid cell status: {}", s)),
        }
    }
}

impl std::fmt::Display for CellStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CellStatus::Active => "ACTIVE",
            CellStatus::Inactive => "INACTIVE",
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChartOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BarLayout {
    Grouped,
    Stacked,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SortOrder {
    None,
    Asc,
    Desc,
}

impl SortOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortOrder::None => "NONE",
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AxisTickShow {
    Show,
    Hide,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AxisMinorTickShow {
    Show,
    Hide,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricFormat {
    Default,
    Percentage,
    Currency,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Metric,
}

impl ChartType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChartType::Bar => "BAR",
            ChartType::Line => "LINE",
            ChartType::Pie => "PIE",
            ChartType::Metric => "METRIC",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AggregateFunction {
    Max,
    Min,
    Sum,
    Count,
    Avg,
}

impl AggregateFunction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AggregateFunction::Max => "MAX",
            AggregateFunction::Min => "MIN",
            AggregateFunction::Sum => "SUM",
            AggregateFunction::Count => "COUNT",
            AggregateFunction::Avg => "AVG",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "MAX" => Ok(AggregateFunction::Max),
            "MIN" => Ok(AggregateFunction::Min),
            "SUM" => Ok(AggregateFunction::Sum),
            "COUNT" => Ok(AggregateFunction::Count),
            "AVG" => Ok(AggregateFunction::Avg),
            _ => Err(format!("Invalid aggregate function: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CellDisplayOrderMovement {
    Up,
    Down,
    Top,
    Bottom,
}

