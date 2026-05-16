use crate::entity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Anthropic,
    Openai,
}

impl Provider {
    pub fn as_db_str(self) -> &'static str {
        match self {
            Self::Anthropic => "ANTHROPIC",
            Self::Openai => "OPENAI",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "ANTHROPIC" => Some(Self::Anthropic),
            "OPENAI" => Some(Self::Openai),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiConfigurationStatus {
    NotConfigured,
    Untested,
    Live,
    Failed,
}

impl AiConfigurationStatus {
    pub fn from_db_liveness_when_row_exists(db: &str) -> Self {
        match db {
            "LIVE" => Self::Live,
            "FAILED" => Self::Failed,
            _ => Self::Untested,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpsertAiConfigurationRequest {
    pub provider: Provider,
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct TestAiConfigurationRequest {
    pub provider: Provider,
    pub api_key: String,
}

#[derive(Debug, Serialize)]
pub struct TestAiConfigurationResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AiConfigurationDto {
    pub status: AiConfigurationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<Provider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_checked_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

impl AiConfigurationDto {
    pub fn unconfigured() -> Self {
        Self {
            status: AiConfigurationStatus::NotConfigured,
            provider: None,
            last_checked_at: None,
        }
    }

    pub fn from_model(model: &entity::ai_configuration::Model) -> Self {
        Self {
            status: AiConfigurationStatus::from_db_liveness_when_row_exists(
                model.liveness_status.as_str(),
            ),
            provider: Provider::from_db_str(&model.provider),
            last_checked_at: model.last_checked_at,
        }
    }
}
