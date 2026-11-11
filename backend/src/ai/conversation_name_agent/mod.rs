mod system_prompt;
mod user_prompt;

use crate::ai::agent_utils::{AgentName, ModelRole};
use crate::authorization::dto::TenantId;
use crate::common::AppState;
use anyhow::Result;

const MAX_NAME_LENGTH: usize = 60;

pub async fn suggest_name(
    app_state: &AppState,
    tenant_id: &TenantId,
    question: &str,
) -> Result<String> {
    let model_client = app_state.require_model_client(tenant_id).await?;
    let agent = model_client.build_agent(
        ModelRole::Default,
        AgentName::ConversationNameAgent,
        system_prompt::SystemPrompt::SuggestName.as_str(),
        0.3,
        100,
    );

    let prompt = user_prompt::UserPrompt::SuggestName {
        question: question.to_string(),
    }
    .render();

    let name: String = agent.prompt(&prompt).await?;
    let name = name.trim().trim_matches('"').to_string();
    let name: String = name.chars().take(MAX_NAME_LENGTH).collect();
    Ok(name)
}
