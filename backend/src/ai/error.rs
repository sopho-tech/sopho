use crate::ai_usage::error::AiUsageError;
use rig_core::ProviderError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentCallError {
    #[error("PromptError: {0}")]
    Prompt(#[from] Box<ProviderError>),
    #[error("DeserializationError: {0}")]
    UnparsableAnswer(#[from] serde_json::Error),
    #[error("EmptyResponse: model returned no content")]
    EmptyAnswer,
    #[error("{0}")]
    UsageRecording(#[from] AiUsageError),
}

pub fn is_usage_recording_failure(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        matches!(
            cause.downcast_ref::<AgentCallError>(),
            Some(AgentCallError::UsageRecording(_))
        )
    })
}
