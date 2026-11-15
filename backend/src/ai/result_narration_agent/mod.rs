mod system_prompt;
mod user_prompt;

use crate::ai::agent_utils::{AgentName, AgentSettings, ModelRole};
use crate::ai::dto::{Event, EventChannels};
use crate::ai::error::is_usage_recording_failure;
use crate::ai_usage::dto::AiCallContext;
use crate::common::AppState;
use crate::database::constants::QueryResult;
use anyhow::Result;

const NARRATION_SAMPLE_ROWS: usize = 50;

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct NarrationResponse {
    narration: String,
}

pub async fn execute(
    app_state: &AppState,
    call_context: &AiCallContext,
    question: &str,
    sql: &str,
    query_result: &QueryResult,
    channels: &EventChannels,
) -> Result<()> {
    channels.send_sse_only(Event::Narrating).await;
    match narrate(app_state, call_context, question, sql, query_result).await {
        Ok(narration) if !narration.trim().is_empty() => {
            let _ = channels.send(Event::Narrated { narration }).await;
        }
        Ok(_) => tracing::error!("result narration: agent returned empty narration"),
        Err(error) if is_usage_recording_failure(&error) => return Err(error),
        Err(e) => tracing::error!("result narration: generation failed: {e}"),
    }
    Ok(())
}

async fn narrate(
    app_state: &AppState,
    call_context: &AiCallContext,
    question: &str,
    sql: &str,
    query_result: &QueryResult,
) -> Result<String> {
    let model_client = call_context.model_client();
    let agent = model_client.build_agent(
        app_state,
        call_context,
        AgentSettings {
            role: ModelRole::Default,
            agent_name: AgentName::ResultNarrationAgent,
            system_prompt: system_prompt::SystemPrompt::NarrateResult.as_str(),
            temperature: 0.3,
            max_tokens: 500,
        },
    );

    let rows = &query_result.data[..query_result.data.len().min(NARRATION_SAMPLE_ROWS)];
    let prompt = user_prompt::UserPrompt::NarrateResult {
        question,
        sql,
        columns: &query_result.columns,
        rows,
        total_row_count: query_result.data.len(),
    }
    .render();

    let response: NarrationResponse = agent.prompt_typed(prompt).await?;
    Ok(response.narration)
}
