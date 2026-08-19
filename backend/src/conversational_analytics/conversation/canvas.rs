use crate::ai::canvas_generation_agent;
use crate::ai::dto::{CanvasCandidate, ConversationHistory, Event, EventChannels};
use crate::canvas::dto::CanvasChangeSummary;
use crate::canvas::service as canvas_service;
use crate::common::AppState;
use crate::conversational_analytics::conversation::constants::CONVERSATION_HISTORY_SCAN_MESSAGE_LIMIT;
use crate::conversational_analytics::message::service as message_service;
use crate::conversational_analytics::message_content::service as message_content_service;
use std::collections::HashMap;
use uuid::Uuid;

pub async fn generate(
    app_state: &AppState,
    conversation_id: Uuid,
    connection_id: Uuid,
    conversation_history: &ConversationHistory,
    channels: &EventChannels,
) -> anyhow::Result<()> {
    channels.send(Event::GeneratingCanvas).await?;

    let candidates = load(app_state, conversation_id).await;
    let plan =
        canvas_generation_agent::execute(app_state, conversation_history, &candidates).await?;

    let target_canvas_id = plan.resolve_target_canvas(&candidates);
    let target =
        target_canvas_id.and_then(|id| candidates.iter().find(|candidate| candidate.id == id));
    let ops = plan.resolve_ops(target)?;
    if ops.is_empty() && target_canvas_id.is_none() {
        anyhow::bail!("the canvas plan contained no usable cells");
    }

    let summary =
        canvas_service::apply_canvas_plan(app_state, connection_id, plan, target_canvas_id, ops)
            .await?;
    channels.send(generated_event(summary)).await?;
    Ok(())
}

fn generated_event(summary: CanvasChangeSummary) -> Event {
    Event::CanvasGenerated {
        canvas_id: summary.canvas_id,
        name: summary.name,
        description: summary.description,
        reused: summary.reused,
        reasoning: summary.reasoning,
        cells_added: summary.cells_added,
        cells_updated: summary.cells_updated,
        cells_removed: summary.cells_removed,
        sql_cell_count: summary.sql_cell_count,
        chart_cell_count: summary.chart_cell_count,
        dashboard_charts_count: summary.dashboard_charts_count,
    }
}

async fn load(app_state: &AppState, conversation_id: Uuid) -> Vec<CanvasCandidate> {
    let canvas_ids = match collect_canvas_ids(app_state, conversation_id).await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!("conversation canvases: failed to collect canvas ids: {e}");
            return Vec::new();
        }
    };
    if canvas_ids.is_empty() {
        return Vec::new();
    }
    canvas_service::list_canvas_candidates(app_state, &canvas_ids).await
}

async fn collect_canvas_ids(
    app_state: &AppState,
    conversation_id: Uuid,
) -> anyhow::Result<Vec<Uuid>> {
    let mut messages = message_service::list_messages_for_conversation(
        &app_state.database_connection,
        conversation_id,
        Some(CONVERSATION_HISTORY_SCAN_MESSAGE_LIMIT as u64),
        true,
    )
    .await?;
    messages.sort_by_key(|message| message.sequence_number);

    let message_ids: Vec<Uuid> = messages.iter().map(|message| message.id).collect();
    let contents = message_content_service::list_message_content_for_messages(
        &app_state.database_connection,
        &message_ids,
    )
    .await?;

    let mut by_message: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for content in contents.iter() {
        if let Ok(Event::CanvasGenerated { canvas_id, .. }) =
            serde_json::from_str::<Event>(&content.content)
        {
            by_message
                .entry(content.conversation_message_id)
                .or_default()
                .push(canvas_id);
        }
    }

    let mut seen = Vec::new();
    for message_id in message_ids.iter() {
        for canvas_id in by_message.remove(message_id).unwrap_or_default() {
            if !seen.contains(&canvas_id) {
                seen.push(canvas_id);
            }
        }
    }
    Ok(seen)
}
