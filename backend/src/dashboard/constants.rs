use serde::{Deserialize, Serialize};
use std::fmt;

pub const MAX_CHARTS_PER_DASHBOARD: usize = 30;
pub const GRID_COLUMN_COUNT: u16 = 12;
pub const MIN_CHART_WIDTH: u16 = 4;
pub const MIN_CHART_HEIGHT: u16 = 3;
pub const MAX_CHART_HEIGHT: u16 = 6;
pub const DEFAULT_CHART_WIDTH: u16 = 4;
pub const DEFAULT_CHART_HEIGHT: u16 = 3;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DashboardStatus {
    Active,
    Inactive,
}

impl DashboardStatus {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ACTIVE" => Ok(DashboardStatus::Active),
            "INACTIVE" => Ok(DashboardStatus::Inactive),
            _ => Err(format!("Invalid dashboard status: {}", s)),
        }
    }
}

impl fmt::Display for DashboardStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            DashboardStatus::Active => "ACTIVE",
            DashboardStatus::Inactive => "INACTIVE",
        })
    }
}
