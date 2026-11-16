mod system_prompt;
mod user_prompt;

use crate::ai::agent_utils::{AgentName, AgentSettings, ModelRole};
use crate::ai::dto::DashboardSummaryInput;
use crate::ai_usage::datatypes::AiCallContext;
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
    call_context: &AiCallContext,
    dashboard: &DashboardSummaryInput,
) -> Result<String> {
    let model_client = call_context.model_client();
    let agent = model_client.build_agent(
        app_state,
        call_context,
        AgentSettings {
            role: ModelRole::Default,
            agent_name: AgentName::DashboardSummaryAgent,
            system_prompt: system_prompt::SystemPrompt::SummarizeDashboard.as_str(),
            temperature: TEMPERATURE,
            max_tokens: MAX_TOKENS,
        },
    );

    let prompt = user_prompt::UserPrompt::SummarizeDashboard { dashboard }.render();
    let response: DashboardSummaryResponse = agent.prompt_typed(prompt).await?;

    let summary = response.summary.trim().to_string();
    if summary.is_empty() {
        anyhow::bail!("dashboard summary agent returned an empty summary");
    }
    Ok(summary)
}
