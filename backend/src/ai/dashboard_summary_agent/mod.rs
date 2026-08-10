mod system_prompt;
mod user_prompt;

use crate::ai::agent_utils::{AgentName, ModelRole};
use crate::ai::dto::DashboardSummaryInput;
use crate::common::AppState;
use anyhow::Result;

const TEMPERATURE: f64 = 0.3;
const MAX_TOKENS: u64 = 700;

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct DashboardSummaryResponse {
    summary: String,
}

pub async fn summarize_dashboard(
    app_state: &AppState,
    dashboard: &DashboardSummaryInput,
) -> Result<String> {
    let model_client = app_state.require_model_client().await?;
    let agent = model_client.build_agent(
        ModelRole::Default,
        AgentName::DashboardSummaryAgent,
        system_prompt::SystemPrompt::SummarizeDashboard.as_str(),
        TEMPERATURE,
        MAX_TOKENS,
    );

    let prompt = user_prompt::UserPrompt::SummarizeDashboard { dashboard }.render();
    let response: DashboardSummaryResponse = agent.prompt_typed(prompt).await?;

    let summary = response.summary.trim().to_string();
    if summary.is_empty() {
        anyhow::bail!("dashboard summary agent returned an empty summary");
    }
    Ok(summary)
}
