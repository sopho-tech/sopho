use serde::{Deserialize, Serialize};
use std::fmt;

pub const GRID_COLUMN_COUNT: u16 = 12;
pub const MIN_CHART_WIDTH: u16 = 4;
pub const MIN_CHART_HEIGHT: u16 = 3;
pub const MAX_CHART_HEIGHT: u16 = 6;
pub const DEFAULT_CHART_WIDTH: u16 = 4;
pub const DEFAULT_CHART_HEIGHT: u16 = 3;

#[derive(Debug, Serialize, Deserialize)]
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

    pub fn deserialize_from_str<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }

    pub fn serialize_to_str<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = value.to_string();
        serializer.serialize_str(&s)
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
