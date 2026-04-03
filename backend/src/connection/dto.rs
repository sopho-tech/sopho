use crate::connection::constants::ConnectionStatus;
use crate::connection::constants::SourceType;
use crate::entity;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

fn deserialize_port<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PortInput {
        Number(i64),
        String(String),
    }
    match PortInput::deserialize(deserializer)? {
        PortInput::Number(n) => Ok(n.to_string()),
        PortInput::String(s) => Ok(s),
    }
}

fn deserialize_optional_port<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PortInput {
        Number(i64),
        String(String),
    }
    let opt = Option::<PortInput>::deserialize(deserializer)?;
    Ok(opt.map(|p| match p {
        PortInput::Number(n) => n.to_string(),
        PortInput::String(s) => s,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConnectionDto {
    pub name: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub host: Option<String>,
    #[serde(deserialize_with = "deserialize_optional_port")]
    pub port: Option<String>,
    pub database: String,
    pub schema: Option<String>,
    pub description: Option<String>,
    #[serde(deserialize_with = "SourceType::deserialize_from_str")]
    pub source_type: SourceType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(deserialize_with = "SourceType::deserialize_from_str")]
    #[serde(serialize_with = "SourceType::serialize_to_str")]
    pub source_type: SourceType,
    pub database: String,
    pub host: String,
    #[serde(deserialize_with = "deserialize_port")]
    pub port: String,
    pub schema: Option<String>,
    pub username: String,
    pub password: String,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    #[serde(deserialize_with = "ConnectionStatus::deserialize_from_str")]
    #[serde(serialize_with = "ConnectionStatus::serialize_to_str")]
    pub status: ConnectionStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteQueryDto {
    pub query: String,
}

impl From<entity::connection::Model> for ConnectionDto {
    fn from(model: entity::connection::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            source_type: SourceType::from_str(&model.source_type).unwrap(),
            database: model.database,
            host: model.host.unwrap_or_default(),
            port: model.port.unwrap_or_default(),
            schema: model.schema,
            username: model.username.unwrap_or_default(),
            password: model.password.unwrap_or_default(),
            created_at: model.created_at,
            updated_at: model.updated_at,
            status: ConnectionStatus::from_str(&model.status).unwrap(),
        }
    }
}
