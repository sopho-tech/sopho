use crate::ai::dto::{
    ConversationHistory, ConversationHistoryTerminalStatus, ConversationHistoryTurn,
};
use crate::common::AppState;
use crate::conversational_analytics::conversation::constants::{
    MessageStatus, CONVERSATION_HISTORY_SCAN_MESSAGE_LIMIT, CONVERSATION_HISTORY_TURN_LIMIT,
};
use crate::conversational_analytics::message::constants::Sender;
use crate::conversational_analytics::message::dto::ConversationMessageDto;
use crate::conversational_analytics::message::service as message_service;
use crate::conversational_analytics::message_content::dto::ConversationMessageContentDto;
use crate::conversational_analytics::message_content::service as message_content_service;
use crate::conversational_analytics::segment;
use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use uuid::Uuid;

struct TurnCandidate {
    human_message_id: Uuid,
    assistant_message_id: Uuid,
    terminal_status: ConversationHistoryTerminalStatus,
}

pub async fn load(
    app_state: &AppState,
    conversation_id: Uuid,
    current_assistant_message_id: Uuid,
) -> anyhow::Result<ConversationHistory> {
    let messages = message_service::list_messages_for_conversation(
        &app_state.database_connection,
        conversation_id,
        Some(CONVERSATION_HISTORY_SCAN_MESSAGE_LIMIT as u64),
        true,
    )
    .await
    .map_err(|e| {
        tracing::error!("conversation history: failed to list messages: {e}");
        anyhow::anyhow!("failed to load conversation history: {e}")
    })?;

    let candidates = pair_turns(messages, current_assistant_message_id);
    let selected = ConversationHistory::select_relevant(
        candidates,
        CONVERSATION_HISTORY_TURN_LIMIT,
        |candidate| candidate.terminal_status,
    );
    let turns = hydrate(&app_state.database_connection, selected).await;

    Ok(ConversationHistory::from_turns(
        turns,
        CONVERSATION_HISTORY_TURN_LIMIT,
    ))
}

fn pair_turns(
    messages: Vec<ConversationMessageDto>,
    current_assistant_message_id: Uuid,
) -> Vec<TurnCandidate> {
    let mut candidates: Vec<TurnCandidate> = Vec::new();
    let mut pending_human: Option<ConversationMessageDto> = None;

    for message in messages.into_iter().rev() {
        if message.id == current_assistant_message_id {
            continue;
        }
        match message.sender.parse::<Sender>() {
            Ok(Sender::Human) => pending_human = Some(message),
            Ok(Sender::Assistant) => {
                let Some(human) = pending_human.take() else {
                    continue;
                };
                let Some(terminal_status) = terminal_status_of(&message.status) else {
                    continue;
                };
                candidates.push(TurnCandidate {
                    human_message_id: human.id,
                    assistant_message_id: message.id,
                    terminal_status,
                });
            }
            Err(_) => continue,
        }
    }

    candidates
}

fn terminal_status_of(status: &str) -> Option<ConversationHistoryTerminalStatus> {
    match status.parse::<MessageStatus>().ok()? {
        MessageStatus::Processed => Some(ConversationHistoryTerminalStatus::Completed),
        MessageStatus::AwaitingClarification => {
            Some(ConversationHistoryTerminalStatus::AwaitingClarification)
        }
        MessageStatus::Rejected => Some(ConversationHistoryTerminalStatus::Rejected),
        MessageStatus::Failed => Some(ConversationHistoryTerminalStatus::Failed),
        MessageStatus::Processing => None,
    }
}

async fn hydrate(
    db: &DatabaseConnection,
    candidates: Vec<TurnCandidate>,
) -> Vec<ConversationHistoryTurn> {
    let message_ids: Vec<Uuid> = candidates
        .iter()
        .flat_map(|candidate| [candidate.human_message_id, candidate.assistant_message_id])
        .collect();

    let contents =
        match message_content_service::list_message_content_for_messages(db, &message_ids).await {
            Ok(contents) => contents,
            Err(e) => {
                tracing::error!("conversation history: failed to list message content: {e}");
                Vec::new()
            }
        };

    let mut by_message: HashMap<Uuid, Vec<ConversationMessageContentDto>> = HashMap::new();
    for content in contents {
        by_message
            .entry(content.conversation_message_id)
            .or_default()
            .push(content);
    }

    candidates
        .into_iter()
        .map(|candidate| {
            let human = by_message
                .get(&candidate.human_message_id)
                .map(Vec::as_slice)
                .unwrap_or_default();
            let assistant = by_message
                .get(&candidate.assistant_message_id)
                .map(Vec::as_slice)
                .unwrap_or_default();
            ConversationHistoryTurn {
                user_question: plain_text(human),
                terminal_status: candidate.terminal_status,
                assistant_message: assistant_message(assistant, candidate.terminal_status),
                generated_sql: generated_sql(assistant, candidate.terminal_status),
            }
        })
        .collect()
}

fn plain_text(contents: &[ConversationMessageContentDto]) -> String {
    contents
        .first()
        .map(|content| segment::extract_plain_text(&content.content))
        .unwrap_or_default()
}

fn assistant_message(
    contents: &[ConversationMessageContentDto],
    terminal_status: ConversationHistoryTerminalStatus,
) -> Option<String> {
    match terminal_status {
        ConversationHistoryTerminalStatus::AwaitingClarification
        | ConversationHistoryTerminalStatus::Rejected => {
            for content in contents.iter() {
                if let Some(crate::ai::dto::Event::Routed { decision }) =
                    parse_persisted_event(&content.content)
                {
                    return Some(decision.message);
                }
            }
            None
        }
        ConversationHistoryTerminalStatus::Completed => {
            Some(narration(contents).unwrap_or_else(|| "<query executed>".to_string()))
        }
        ConversationHistoryTerminalStatus::Failed => None,
    }
}

fn generated_sql(
    contents: &[ConversationMessageContentDto],
    terminal_status: ConversationHistoryTerminalStatus,
) -> Option<String> {
    match terminal_status {
        ConversationHistoryTerminalStatus::AwaitingClarification
        | ConversationHistoryTerminalStatus::Rejected => return None,
        ConversationHistoryTerminalStatus::Completed
        | ConversationHistoryTerminalStatus::Failed => {}
    }

    let mut sql_of_turn: Option<String> = None;
    for content in contents.iter() {
        if let Some(crate::ai::dto::Event::GeneratedSql { sql }) =
            parse_persisted_event(&content.content)
        {
            sql_of_turn = Some(sql);
        }
    }
    sql_of_turn
}

fn narration(contents: &[ConversationMessageContentDto]) -> Option<String> {
    for content in contents.iter() {
        if let Some(crate::ai::dto::Event::Narrated { narration }) =
            parse_persisted_event(&content.content)
        {
            return Some(narration);
        }
    }
    None
}

fn parse_persisted_event(content: &str) -> Option<crate::ai::dto::Event> {
    serde_json::from_str(content).ok()
}
