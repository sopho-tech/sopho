use crate::cell::constants::CellType;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotebookStatus {
    Active,
    Inactive,
    Failed,
}

impl fmt::Display for NotebookStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            NotebookStatus::Active => "ACTIVE",
            NotebookStatus::Inactive => "INACTIVE",
            NotebookStatus::Failed => "FAILED",
        })
    }
}

#[derive(Deserialize, Serialize)]
pub struct QueryFilters {
    cell_type: Option<CellType>,
}

impl QueryFilters {
    pub fn cell_type(&self) -> Option<CellType> {
        self.cell_type.clone()
    }
}
