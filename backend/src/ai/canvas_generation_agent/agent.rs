use super::system_prompt::SystemPrompt;
use super::user_prompt::UserPrompt;
use crate::ai::agent_utils::{AgentName, ModelRole};
use crate::ai::dto::{CanvasPlan, ConversationHistory};
use crate::common::AppState;
use anyhow::Result;
use tracing::info;

pub async fn execute(
    app_state: &AppState,
    conversation_history: &ConversationHistory,
) -> Result<CanvasPlan> {
    info!(
        "canvas_generation_agent: planning canvas (history_turns={})",
        conversation_history.len()
    );

    let model_client = app_state.require_model_client().await?;
    let agent = model_client.build_agent(
        ModelRole::Default,
        AgentName::CanvasGenerationAgent,
        SystemPrompt::GenerateCanvas.as_str(),
        0.2,
        4096,
    );

    let prompt = UserPrompt::Generate {
        conversation_history,
    }
    .render();
    let plan: CanvasPlan = agent.prompt_typed::<CanvasPlan>(prompt).await?;

    info!("canvas_generation_agent: planned {} cells", plan.cells.len());
    Ok(plan)
}
