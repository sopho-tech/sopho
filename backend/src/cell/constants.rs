use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CellType {
    Text,
    Code,
    Markdown,
    Sql,
    Chart,
}

impl fmt::Display for CellType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl CellType {
    pub fn to_string(&self) -> String {
        match self {
            CellType::Text => "TEXT".to_string(),
            CellType::Code => "CODE".to_string(),
            CellType::Markdown => "MARKDOWN".to_string(),
            CellType::Sql => "SQL".to_string(),
            CellType::Chart => "CHART".to_string(),
        }
    }

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

#[derive(Debug, Serialize, Deserialize)]
pub enum CellStatus {
    Active,
    Inactive,
}

impl CellStatus {
    pub fn to_string(&self) -> String {
        match self {
            CellStatus::Active => "ACTIVE".to_string(),
            CellStatus::Inactive => "INACTIVE".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ACTIVE" => Ok(CellStatus::Active),
            "INACTIVE" => Ok(CellStatus::Inactive),
            _ => Err(format!("Invalid cell status: {}", s)),
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

impl std::fmt::Display for CellStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
