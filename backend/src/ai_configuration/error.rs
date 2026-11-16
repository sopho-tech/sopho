use crate::ai_configuration::constants::{
    AI_NOT_CONFIGURED_MESSAGE, AI_NOT_LIVE_MESSAGE, PLATFORM_AI_ALLOWANCE_USED_MESSAGE,
};
use crate::ai_usage::error::AiUsageError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiAccessError {
    #[error("{AI_NOT_CONFIGURED_MESSAGE}")]
    NotConfigured,
    #[error("{AI_NOT_LIVE_MESSAGE}")]
    NotLive,
    #[error("{PLATFORM_AI_ALLOWANCE_USED_MESSAGE}")]
    PlatformAiAllowanceUsed,
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Usage(#[from] AiUsageError),
    #[error("the saved AI key of tenant {tenant_id} has no model client")]
    MissingModelClient { tenant_id: uuid::Uuid },
}

#[derive(Debug, Error)]
pub enum AiConfigurationError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("AI usage could not be recorded: {0}")]
    UsageRecording(String),
    #[error("{0}")]
    Usage(#[from] AiUsageError),
    #[error("API key must not be empty")]
    EmptyApiKey,
    #[error("{0}")]
    Unauthorized(#[from] crate::authorization::errors::AuthorizationError),
}
