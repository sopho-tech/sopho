use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SqlDialect {
    Postgresql,
    #[serde(rename = "MYSQL")]
    MySql,
    Sqlite,
}

impl fmt::Display for SqlDialect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlDialect::Postgresql => write!(f, "PostgreSQL"),
            SqlDialect::MySql => write!(f, "MySQL"),
            SqlDialect::Sqlite => write!(f, "SQLite"),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceType {
    Postgresql,
    Supabase,
    MySql,
    Sqlite,
    MsSql,
    Oracle,
    #[serde(rename = "MONGODB")]
    MongoDb,
    Redis,
    Elasticsearch,
    Kafka,
    Api,
    File,
    GoogleSheets,
    Airtable,
}

impl SourceType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "POSTGRESQL" => Ok(SourceType::Postgresql),
            "SUPABASE" => Ok(SourceType::Supabase),
            "MYSQL" => Ok(SourceType::MySql),
            "SQLITE" => Ok(SourceType::Sqlite),
            "MS_SQL" => Ok(SourceType::MsSql),
            "ORACLE" => Ok(SourceType::Oracle),
            "MONGODB" => Ok(SourceType::MongoDb),
            "REDIS" => Ok(SourceType::Redis),
            "ELASTICSEARCH" => Ok(SourceType::Elasticsearch),
            "KAFKA" => Ok(SourceType::Kafka),
            "API" => Ok(SourceType::Api),
            "FILE" => Ok(SourceType::File),
            "GOOGLE_SHEETS" => Ok(SourceType::GoogleSheets),
            "AIRTABLE" => Ok(SourceType::Airtable),
            _ => Err(format!("Invalid source type: {}", s)),
        }
    }



    pub fn to_sql_dialect(&self) -> Option<SqlDialect> {
        match self {
            SourceType::Postgresql | SourceType::Supabase => Some(SqlDialect::Postgresql),
            SourceType::MySql => Some(SqlDialect::MySql),
            SourceType::Sqlite => Some(SqlDialect::Sqlite),
            SourceType::MsSql
            | SourceType::Oracle
            | SourceType::MongoDb
            | SourceType::Redis
            | SourceType::Elasticsearch
            | SourceType::Kafka
            | SourceType::Api
            | SourceType::File
            | SourceType::GoogleSheets
            | SourceType::Airtable => None,
        }
    }
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SourceType::Postgresql => "POSTGRESQL",
            SourceType::Supabase => "SUPABASE",
            SourceType::MySql => "MYSQL",
            SourceType::Sqlite => "SQLITE",
            SourceType::MsSql => "MS_SQL",
            SourceType::Oracle => "ORACLE",
            SourceType::MongoDb => "MONGODB",
            SourceType::Redis => "REDIS",
            SourceType::Elasticsearch => "ELASTICSEARCH",
            SourceType::Kafka => "KAFKA",
            SourceType::Api => "API",
            SourceType::File => "FILE",
            SourceType::GoogleSheets => "GOOGLE_SHEETS",
            SourceType::Airtable => "AIRTABLE",
        })
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
