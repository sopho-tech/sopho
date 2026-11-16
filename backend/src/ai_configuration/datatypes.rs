use crate::ai_configuration::dto::AiConfigurationStatus;
use crate::entity;

pub(super) enum AiKeyDecision<'saved_configuration> {
    Tenant {
        saved_configuration: &'saved_configuration entity::ai_configuration::Model,
        status: AiConfigurationStatus,
    },
    Platform {
        token_allowance: u64,
        used_tokens: u64,
        is_allowance_used: bool,
    },
    Unconfigured,
}
