use crate::ai_summary::constants::{SummaryEntityType, SummaryStatus};
use crate::entity;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct AiSummaryDto {
    #[serde(serialize_with = "SummaryEntityType::serialize_to_str")]
    pub entity_type: SummaryEntityType,
    pub entity_id: Uuid,
    #[serde(serialize_with = "SummaryStatus::serialize_to_str")]
    pub status: SummaryStatus,
    pub summary_text: Option<String>,
    pub error_message: Option<String>,
    pub summarized_entity_count: Option<i32>,
    pub user_prompt: Option<String>,
    pub user_prompt_used: Option<String>,
    pub is_prompt_stale: bool,
    pub generated_at: Option<String>,
    pub requested_at: Option<String>,
}

impl From<entity::ai_summary::Model> for AiSummaryDto {
    fn from(model: entity::ai_summary::Model) -> Self {
        let status = SummaryStatus::from_str(&model.status).unwrap_or(SummaryStatus::Failed);
        let is_prompt_stale =
            status == SummaryStatus::Ready && model.user_prompt != model.user_prompt_used;

        Self {
            entity_type: SummaryEntityType::from_str(&model.entity_type)
                .unwrap_or(SummaryEntityType::ChartCell),
            entity_id: model.entity_id,
            status,
            summary_text: model.summary_text,
            error_message: model.error_message,
            summarized_entity_count: model.summarized_entity_count,
            user_prompt: model.user_prompt,
            user_prompt_used: model.user_prompt_used,
            is_prompt_stale,
            generated_at: model.generated_at.map(|at| at.to_rfc3339()),
            requested_at: (status != SummaryStatus::Idle)
                .then(|| model.requested_at.to_rfc3339()),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateUserPromptDto {
    pub user_prompt: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DashboardSummariesDto {
    pub dashboard: Option<AiSummaryDto>,
    pub charts: Vec<AiSummaryDto>,
}
