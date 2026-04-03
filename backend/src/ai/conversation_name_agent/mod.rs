mod system_prompt;
mod user_prompt;

use crate::common::AppState;
use anyhow::Result;

const MAX_NAME_LENGTH: usize = 60;

pub async fn suggest_name(app_state: &AppState, question: &str) -> Result<String> {
    let agent = app_state.model_client.build_agent(
        "claude-haiku-4-5",
        "conversation_name_agent",
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
