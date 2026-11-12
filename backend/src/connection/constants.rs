use serde::{Deserialize, Serialize};
use std::fmt;
use strum::{Display, EnumString, VariantArray};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SqlDialect {
    Postgresql,
    Sqlite,
}

impl fmt::Display for SqlDialect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlDialect::Postgresql => write!(f, "PostgreSQL"),
            SqlDialect::Sqlite => write!(f, "SQLite"),
        }
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, VariantArray,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceType {
    Postgresql,
    Supabase,
    Sqlite,
}

impl SourceType {
    pub fn from_str(source_type_name: &str) -> Result<Self, String> {
        <Self as std::str::FromStr>::from_str(source_type_name)
            .map_err(|_| format!("Invalid source type: {source_type_name}"))
    }

    pub fn display_name(self) -> &'static str {
        match self {
            SourceType::Postgresql => "PostgreSQL",
            SourceType::Supabase => "Supabase",
            SourceType::Sqlite => "SQLite",
        }
    }

    pub fn to_sql_dialect(self) -> SqlDialect {
        match self {
            SourceType::Postgresql | SourceType::Supabase => SqlDialect::Postgresql,
            SourceType::Sqlite => SqlDialect::Sqlite,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConnectionStatus {
    Active,
    Inactive,
    Failed,
}

impl ConnectionStatus {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ACTIVE" => Ok(ConnectionStatus::Active),
            "INACTIVE" => Ok(ConnectionStatus::Inactive),
            "FAILED" => Ok(ConnectionStatus::Failed),
            _ => Err(format!("Invalid connection status: {}", s)),
        }
    }
}

impl fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ConnectionStatus::Active => "ACTIVE",
            ConnectionStatus::Inactive => "INACTIVE",
            ConnectionStatus::Failed => "FAILED",
        })
    }
}

pub const CONNECTION_NOT_FOUND_MESSAGE: &str = "Connection not found";
