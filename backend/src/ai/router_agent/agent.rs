use super::system_prompt::SystemPrompt;
use super::user_prompt::UserPrompt;
use crate::ai::agent_utils::{AgentName, AgentSettings, ModelRole};
use crate::ai::dto::{ConversationHistory, RouterDecision};
use crate::ai_usage::dto::AiCallContext;
use crate::common::AppState;
use anyhow::Result;
use tracing::info;

pub async fn execute(
    app_state: &AppState,
    call_context: &AiCallContext,
    question: &str,
    conversation_history: &ConversationHistory,
) -> Result<RouterDecision> {
    info!(
        "router_agent: routing question (len={}, history_turns={})",
        question.len(),
        conversation_history.len()
    );

    let model_client = app_state
        .require_model_client(call_context.tenant_id())
        .await?;
    let agent = model_client.build_agent(
        app_state,
        call_context,
        AgentSettings {
            role: ModelRole::Small,
            agent_name: AgentName::RouterAgent,
            system_prompt: SystemPrompt::Routing.as_str(),
            temperature: 0.0,
            max_tokens: 256,
        },
    );

    let prompt = UserPrompt::Route {
        question,
        conversation_history,
    }
    .render();
    let decision: RouterDecision = agent.prompt_typed::<RouterDecision>(prompt).await?;

    info!(
        "router_agent: decided code={:?} message_len={}",
        decision.code,
        decision.message.len()
    );

    Ok(decision)
}
