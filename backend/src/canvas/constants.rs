use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CanvasStatus {
    Active,
    Inactive,
}

impl CanvasStatus {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ACTIVE" => Ok(CanvasStatus::Active),
            "INACTIVE" => Ok(CanvasStatus::Inactive),
            _ => Err(format!("Invalid canvas status: {}", s)),
        }
    }
}

impl fmt::Display for CanvasStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CanvasStatus::Active => "ACTIVE",
            CanvasStatus::Inactive => "INACTIVE",
        })
    }
}

pub const CANVAS_NOT_FOUND_MESSAGE: &str = "Canvas not found";
pub const CANVAS_DELETED_MESSAGE: &str = "Canvas deleted successfully";
pub const DASHBOARD_NOT_FOUND_FOR_CANVAS_MESSAGE: &str = "Dashboard not found for canvas";
