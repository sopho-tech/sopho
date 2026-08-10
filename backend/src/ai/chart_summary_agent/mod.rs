mod system_prompt;
mod user_prompt;

use crate::ai::agent_utils::{AgentName, ModelRole};
use crate::ai::dto::ChartSummaryInput;
use crate::common::AppState;
use anyhow::Result;

const TEMPERATURE: f64 = 0.3;
const MAX_TOKENS: u64 = 300;

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct ChartSummaryResponse {
    summary: String,
}

pub async fn summarize_chart(
    app_state: &AppState,
    chart: &ChartSummaryInput,
    user_prompt: &Option<String>,
) -> Result<String> {
    let model_client = app_state.require_model_client().await?;
    let agent = model_client.build_agent(
        ModelRole::Default,
        AgentName::ChartSummaryAgent,
        system_prompt::SystemPrompt::SummarizeChart.as_str(),
        TEMPERATURE,
        MAX_TOKENS,
    );

    let prompt = user_prompt::UserPrompt::SummarizeChart { chart, user_prompt }.render();
    let response: ChartSummaryResponse = agent.prompt_typed(prompt).await?;

    let summary = response.summary.trim().to_string();
    if summary.is_empty() {
        anyhow::bail!("chart summary agent returned an empty summary");
    }
    Ok(summary)
}
