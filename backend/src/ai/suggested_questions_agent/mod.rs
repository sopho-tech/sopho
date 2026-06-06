mod system_prompt;
mod user_prompt;

use crate::ai::agent_utils::{AgentName, ModelRole};
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
    connection: &entity::connection::Model,
    catalog: &Database,
) -> Result<Vec<String>> {
    let model_client = app_state.require_model_client().await?;
    let agent = model_client.build_agent(
        ModelRole::Default,
        AgentName::SuggestedQuestionsAgent,
        system_prompt::SystemPrompt::SuggestQuestions.as_str(),
        0.4,
        600,
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
