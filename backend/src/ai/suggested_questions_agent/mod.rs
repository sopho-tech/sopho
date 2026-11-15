mod system_prompt;
mod user_prompt;

use crate::ai::agent_utils::{AgentName, AgentSettings, ModelRole};
use crate::ai_usage::dto::AiCallContext;
use crate::common::AppState;
use crate::data_catalog::dto::Database;
use crate::entity;
use anyhow::Result;

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct SuggestedQuestionsResponse {
    questions: Vec<String>,
}

pub async fn suggest_questions(
    app_state: &AppState,
    call_context: &AiCallContext,
    connection: &entity::connection::Model,
    catalog: &Database,
) -> Result<Vec<String>> {
    let model_client = call_context.model_client();
    let agent = model_client.build_agent(
        app_state,
        call_context,
        AgentSettings {
            role: ModelRole::Default,
            agent_name: AgentName::SuggestedQuestionsAgent,
            system_prompt: system_prompt::SystemPrompt::SuggestQuestions.as_str(),
            temperature: 0.4,
            max_tokens: 600,
        },
    );

    let schema = catalog.to_display_string().map_err(anyhow::Error::from)?;
    let prompt = user_prompt::UserPrompt::SuggestQuestions {
        name: &connection.name,
        source_type: &connection.source_type,
        schema: &schema,
    }
    .render();

    let response: SuggestedQuestionsResponse = agent
        .prompt_typed::<SuggestedQuestionsResponse>(prompt)
        .await?;

    Ok(response.questions)
}
