use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::entity;
use crate::connection::constants::SourceType;
use crate::connection::constants::ConnectionStatus;
use chrono::{DateTime, FixedOffset};
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConnectionDto {
    pub name: String,
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: i32,
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
    pub port: i32,
    pub schema: Option<String>,
    pub username: String,
    pub password: String,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    #[serde(deserialize_with = "ConnectionStatus::deserialize_from_str")]
    #[serde(serialize_with = "ConnectionStatus::serialize_to_str")]
    pub status: ConnectionStatus,
}

impl From<entity::connection::Model> for ConnectionDto {
    fn from(model: entity::connection::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            source_type: SourceType::from_str(&model.source_type).unwrap(),
            database: model.database,
            host: model.host,
            port: model.port,
            schema: model.schema,
            username: model.username,
            password: model.password,
            created_at: model.created_at,
            updated_at: model.updated_at,
            status: ConnectionStatus::from_str(&model.status).unwrap(),
        }
    }
}
