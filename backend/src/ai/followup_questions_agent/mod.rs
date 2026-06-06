mod system_prompt;
mod user_prompt;

use crate::ai::agent_utils::{AgentName, ModelRole};
use crate::ai::dto::{ConversationHistoryTerminalStatus, ConversationHistoryTurn};
use crate::common::AppState;
use crate::data_catalog::dto::Database;
use crate::entity;
use anyhow::Result;

pub const FOLLOWUP_QUESTION_COUNT: usize = 2;

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct FollowupQuestionsResponse {
    questions: Vec<String>,
}

fn format_history(history: &[ConversationHistoryTurn]) -> String {
    if history.is_empty() {
        return "(none)".to_string();
    }
    history
        .iter()
        .map(|turn| {
            let status = match turn.terminal_status {
                ConversationHistoryTerminalStatus::Completed => "answered",
                ConversationHistoryTerminalStatus::AwaitingClarification => "clarification requested",
                ConversationHistoryTerminalStatus::Rejected => "rejected",
                ConversationHistoryTerminalStatus::Failed => "failed",
            };
            format!("Q: {} ({status})", turn.user_question)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn suggest_followups(
    app_state: &AppState,
    _connection: &entity::connection::Model,
    question: &str,
    sql: &str,
    history: &[ConversationHistoryTurn],
    catalog: &Database,
) -> Result<Vec<String>> {
    let model_client = app_state.require_model_client().await?;
    let agent = model_client.build_agent(
        ModelRole::Default,
        AgentName::FollowupQuestionsAgent,
        &system_prompt::SystemPrompt::SuggestFollowups.render(FOLLOWUP_QUESTION_COUNT),
        0.4,
        600,
    );

    let schema = catalog.to_display_string().map_err(anyhow::Error::from)?;
    let prompt = user_prompt::UserPrompt::SuggestFollowups {
        count: FOLLOWUP_QUESTION_COUNT,
        question,
        sql,
        history: &format_history(history),
        schema: &schema,
    }
    .render();

    let response: FollowupQuestionsResponse = agent
        .prompt_typed::<FollowupQuestionsResponse>(prompt)
        .await?;

    Ok(response.questions)
}
