use crate::ai::agent_utils::ModelProvider;
use crate::ai_usage::constants::KeySource;
use crate::ai_usage::dto::AllowanceUsageDto;
use serde::{Deserialize, Serialize};

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
    pub provider: ModelProvider,
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct TestAiConfigurationRequest {
    pub provider: ModelProvider,
    pub api_key: String,
}

#[derive(Debug, Serialize)]
pub struct TestAiConfigurationResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PlatformAiAllowanceDto {
    pub allowance_usage: AllowanceUsageDto,
    pub tokens_left_label: String,
    pub is_allowance_used: bool,
}

#[derive(Debug, Serialize)]
pub struct AiConfigurationDto {
    pub status: AiConfigurationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ModelProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_checked_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub is_ai_available: bool,
    pub key_source: Option<KeySource>,
    pub unavailable_message: Option<&'static str>,
    pub platform_ai: Option<PlatformAiAllowanceDto>,
}
