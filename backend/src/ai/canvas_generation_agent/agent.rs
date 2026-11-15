use super::system_prompt::SystemPrompt;
use super::user_prompt::UserPrompt;
use crate::ai::agent_utils::{AgentName, AgentSettings, ModelRole};
use crate::ai::dto::{CanvasCandidate, CanvasPlan, ConversationHistory};
use crate::ai_usage::dto::AiCallContext;
use crate::common::AppState;
use anyhow::Result;
use tracing::info;

pub async fn execute(
    app_state: &AppState,
    call_context: &AiCallContext,
    conversation_history: &ConversationHistory,
    candidates: &[CanvasCandidate],
) -> Result<CanvasPlan> {
    info!(
        "canvas_generation_agent: planning canvas (history_turns={}, existing_canvases={})",
        conversation_history.len(),
        candidates.len()
    );

    let model_client = app_state
        .require_model_client(call_context.tenant_id())
        .await?;
    let agent = model_client.build_agent(
        app_state,
        call_context,
        AgentSettings {
            role: ModelRole::Default,
            agent_name: AgentName::CanvasGenerationAgent,
            system_prompt: SystemPrompt::GenerateCanvas.as_str(),
            temperature: 0.2,
            max_tokens: 4096,
        },
    );

    let prompt = UserPrompt::Generate {
        conversation_history,
        candidates,
    }
    .render();
    let plan: CanvasPlan = agent.prompt_typed::<CanvasPlan>(prompt).await?;

    info!(
        "canvas_generation_agent: planned {} operations (target_canvas_index={:?})",
        plan.cells.len(),
        plan.target_canvas_index
    );
    Ok(plan)
}
