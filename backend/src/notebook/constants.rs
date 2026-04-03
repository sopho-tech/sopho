use crate::cell::constants::CellType;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub enum NotebookStatus {
    Active,
    Inactive,
    Failed,
}

impl NotebookStatus {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ACTIVE" => Ok(NotebookStatus::Active),
            "INACTIVE" => Ok(NotebookStatus::Inactive),
            "FAILED" => Ok(NotebookStatus::Failed),
            _ => Err(format!("Invalid connection status: {}", s)),
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
    #[serde(
        deserialize_with = "CellType::deserialize_option_from_str",
        serialize_with = "CellType::serialize_option_to_str"
    )]
    cell_type: Option<CellType>,
}

impl QueryFilters {
    pub fn cell_type(&self) -> Option<CellType> {
        self.cell_type.clone()
    }
}
