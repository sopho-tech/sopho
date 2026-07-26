use super::constants;
use super::constants::{ConversationStatus, PipelineOutcome};
use super::dto;
use super::error::AppendUserMessageError;
use super::error::ConversationError;
use super::error::ExecuteCompletionError;
use super::repository;
use crate::ai::conversation_name_agent;
use crate::ai::dto::EventChannels;
use crate::ai::dto::{ConversationHistoryTerminalStatus, ConversationHistoryTurn, RouterCode};
use crate::ai::followup_questions_agent;
use crate::ai::router_agent;
use crate::ai::text_to_sql_agent;
use crate::ai::visualization_agent;
use crate::ai_configuration::dto::AiConfigurationStatus;
use crate::ai_configuration::service as ai_config_service;
use crate::common::time_utils;
use crate::common::AppState;
use crate::connection::service as connection_service;
use crate::conversational_analytics::command::constants::{self as command_constants, Command};
use crate::conversational_analytics::conversation::constants::ContentType;
use crate::conversational_analytics::conversation::constants::MessageStatus;
use crate::conversational_analytics::conversation::constants::CONVERSATION_HISTORY_MESSAGE_LIMIT;
use crate::conversational_analytics::conversation::constants::TERMINAL_MESSAGE_STATUSES;
use crate::conversational_analytics::message::constants::Sender;
use crate::conversational_analytics::message::dto::{
    ConversationMessageDto, ConversationMessageWithContentDto, CreateConversationMessageDto,
};
use crate::conversational_analytics::message::repository as message_repository;
use crate::conversational_analytics::message::service as message_service;
use crate::conversational_analytics::message_content::dto::{
    ConversationMessageContentDto, CreateConversationMessageContentDto,
};
use crate::conversational_analytics::message_content::service as message_content_service;
use crate::conversational_analytics::segment;
use crate::data_catalog;
use crate::entity;
use axum::response::sse::{Event, Sse};
use sea_orm::TransactionTrait;
use std::collections::HashMap;
use std::{convert::Infallible, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt as _;
use uuid::Uuid;

async fn ensure_ai_live(app_state: &AppState) -> Result<(), ConversationError> {
    let status = ai_config_service::get_status(app_state)
        .await
        .map_err(|e| match e {
            crate::ai_configuration::error::AiConfigurationError::Database(d) => {
                ConversationError::Database(d)
            }
            other => ConversationError::Conversion(other.to_string()),
        })?;
    if status.status == AiConfigurationStatus::Live {
        Ok(())
    } else {
        Err(ConversationError::AiNotLive)
    }
}

pub async fn get_conversation(
    app_state: AppState,
    conversation_id: Uuid,
) -> Result<dto::ConversationWithMessagesDto, ConversationError> {
    let model = repository::get_conversation(&app_state.database_connection, conversation_id)
        .await
        .map_err(|e| match &e {
            sea_orm::DbErr::RecordNotFound(_) => ConversationError::NotFound,
            _ => ConversationError::Database(e),
        })?;
    let conversation =
        dto::ConversationDto::try_from(model).map_err(ConversationError::Conversion)?;
    let messages = message_service::list_messages_for_conversation(
        &app_state.database_connection,
        conversation_id,
        None,
        false,
    )
    .await?;
    let message_ids: Vec<Uuid> = messages.iter().map(|m| m.id).collect();
    let contents = message_content_service::list_message_content_for_messages(
        &app_state.database_connection,
        &message_ids,
    )
    .await?;
    let mut by_message: HashMap<Uuid, Vec<ConversationMessageContentDto>> = HashMap::new();
    for c in contents {
        by_message
            .entry(c.conversation_message_id)
            .or_default()
            .push(c);
    }
    let messages: Vec<ConversationMessageWithContentDto> = messages
        .into_iter()
        .map(|m| {
            let content = by_message.remove(&m.id).unwrap_or_default();
            ConversationMessageWithContentDto::new(m, content)
        })
        .collect();
    let should_execute_completion = messages
        .last()
        .map(|m| m.message.sender == Sender::Human.to_string())
        .unwrap_or(false);
    Ok(dto::ConversationWithMessagesDto {
        conversation,
        messages,
        should_execute_completion,
    })
}

fn normalize_search(search: Option<String>) -> Option<String> {
    search
        .map(|term| term.trim().to_string())
        .filter(|term| !term.is_empty())
}

pub async fn list_conversations(
    app_state: AppState,
    query: dto::ListConversationsQuery,
) -> Result<dto::PaginatedConversationsDto, ConversationError> {
    let page = query.page.unwrap_or(0);
    let page_size = query
        .page_size
        .unwrap_or(constants::DEFAULT_PAGE_SIZE)
        .clamp(1, constants::MAX_PAGE_SIZE);
    let search = normalize_search(query.search);

    let (models, total) = repository::list_conversations(
        &app_state.database_connection,
        search.as_deref(),
        page,
        page_size,
    )
    .await?;

    let conversation_ids: Vec<Uuid> = models.iter().map(|model| model.id).collect();
    let user_message_counts = message_service::count_messages_by_sender_for_conversations(
        &app_state.database_connection,
        &conversation_ids,
        &Sender::Human.to_string(),
    )
    .await?;

    let items = models
        .into_iter()
        .map(|model| {
            let user_message_count = user_message_counts
                .get(&model.id)
                .copied()
                .unwrap_or_default();
            dto::ConversationDto::try_from(model).map(|conversation| dto::ConversationListItemDto {
                conversation,
                user_message_count,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(ConversationError::Conversion)?;

    Ok(dto::PaginatedConversationsDto {
        items,
        total,
        page,
        page_size,
    })
}

/// Sequence number of conversation messages start from `1` not `0`
pub async fn create_conversation(
    app_state: AppState,
    payload: dto::CreateConversationDto,
) -> Result<dto::ConversationDto, ConversationError> {
    ensure_ai_live(&app_state).await?;
    let now = time_utils::now_utc_into();
    let conversation_id = Uuid::new_v4();
    let conversation = entity::conversation::Model {
        id: conversation_id,
        connection_id: payload.connection_id,
        name: constants::DEFAULT_CONVERSATION_NAME.to_string(),
        status: ConversationStatus::Active.to_string(),
        created_at: now,
        updated_at: now,
    };
    let segments_json = segment::serialize_segments(&payload.segments);
    let plain_text = segment::plain_text_from_segments(&payload.segments);

    let txn = app_state.database_connection.begin().await?;

    let saved_conversation = repository::save_conversation_transaction(&txn, conversation).await?;

    let user_message =
        CreateConversationMessageDto::initial_user_message(saved_conversation.id, &plain_text);
    let created_message = message_service::create_message(&txn, user_message).await?;

    let user_message_content = CreateConversationMessageContentDto {
        conversation_message_id: created_message.id,
        sequence_number: 1,
        content_type: ContentType::Text.to_string(),
        content: segments_json,
        status: MessageStatus::Processed.to_string(),
    };
    message_content_service::create_message_content(&txn, user_message_content).await?;

    txn.commit().await?;
    dto::ConversationDto::try_from(saved_conversation).map_err(ConversationError::Conversion)
}

pub async fn update_conversation(
    app_state: AppState,
    conversation_id: Uuid,
    payload: dto::ConversationDto,
) -> Result<dto::ConversationDto, ConversationError> {
    let model =
        repository::update_conversation(&app_state.database_connection, conversation_id, payload)
            .await?;
    dto::ConversationDto::try_from(model).map_err(ConversationError::Conversion)
}

pub async fn delete_conversation(
    app_state: AppState,
    conversation_id: Uuid,
) -> Result<(), ConversationError> {
    let _ = repository::get_conversation(&app_state.database_connection, conversation_id)
        .await
        .map_err(|e| match &e {
            sea_orm::DbErr::RecordNotFound(_) => ConversationError::NotFound,
            _ => ConversationError::Database(e),
        })?;
    let txn = app_state.database_connection.begin().await?;
    message_content_service::delete_message_contents_for_conversation_transaction(
        &txn,
        conversation_id,
    )
    .await?;
    message_service::delete_messages_for_conversation_transaction(&txn, conversation_id).await?;
    repository::delete_conversation_transaction(&txn, conversation_id)
        .await
        .map_err(ConversationError::Database)?;
    txn.commit().await.map_err(ConversationError::Database)?;
    Ok(())
}

pub async fn bulk_delete_conversations(
    app_state: AppState,
    payload: dto::BulkDeleteConversationsDto,
) -> Result<(), ConversationError> {
    let mut conversation_ids = payload.conversation_ids;

    if conversation_ids.is_empty() {
        return Err(ConversationError::InvalidRequest(
            "conversation_ids must not be empty".to_string(),
        ));
    }

    if conversation_ids.len() > constants::MAX_BULK_DELETE_SIZE {
        return Err(ConversationError::InvalidRequest(format!(
            "cannot delete more than {} conversations at once",
            constants::MAX_BULK_DELETE_SIZE
        )));
    }

    conversation_ids.sort_unstable();
    conversation_ids.dedup();

    let existing_count =
        repository::count_conversations_by_ids(&app_state.database_connection, &conversation_ids)
            .await?;
    if existing_count != conversation_ids.len() as u64 {
        return Err(ConversationError::NotFound);
    }

    let txn = app_state.database_connection.begin().await?;
    message_content_service::delete_message_contents_for_conversations_transaction(
        &txn,
        &conversation_ids,
    )
    .await?;
    message_service::delete_messages_for_conversations_transaction(&txn, &conversation_ids).await?;
    repository::delete_conversations_transaction(&txn, &conversation_ids)
        .await
        .map_err(ConversationError::Database)?;
    txn.commit().await.map_err(ConversationError::Database)?;
    Ok(())
}

pub async fn suggest_conversation_name(
    app_state: AppState,
    conversation_id: Uuid,
) -> Result<dto::ConversationDto, ConversationError> {
    ensure_ai_live(&app_state).await?;
    let conversation =
        repository::get_conversation(&app_state.database_connection, conversation_id)
            .await
            .map_err(|e| match &e {
                sea_orm::DbErr::RecordNotFound(_) => ConversationError::NotFound,
                _ => ConversationError::Database(e),
            })?;

    let first_message = message_service::get_first_message_for_conversation(
        &app_state.database_connection,
        conversation_id,
    )
    .await?;

    let first_message = first_message.ok_or(ConversationError::NotFound)?;

    let question = message_content_service::get_first_message_content(
        &app_state.database_connection,
        first_message.id,
    )
    .await?
    .map(|c| c.content)
    .unwrap_or_default();

    let suggested_name = conversation_name_agent::suggest_name(&app_state, &question)
        .await
        .unwrap_or_else(|_| conversation.name.clone());

    let updated_payload = dto::ConversationDto {
        id: conversation_id,
        connection_id: conversation.connection_id,
        name: suggested_name,
        status: constants::ConversationStatus::Active,
        created_at: conversation.created_at,
        updated_at: conversation.updated_at,
    };

    let model = repository::update_conversation(
        &app_state.database_connection,
        conversation_id,
        updated_payload,
    )
    .await?;

    dto::ConversationDto::try_from(model).map_err(ConversationError::Conversion)
}

pub async fn append_user_message(
    app_state: AppState,
    conversation_id: Uuid,
    payload: dto::AppendUserMessageDto,
) -> Result<dto::ConversationWithMessagesDto, AppendUserMessageError> {
    ensure_ai_live(&app_state).await?;

    repository::get_conversation(&app_state.database_connection, conversation_id)
        .await
        .map_err(|e| match &e {
            sea_orm::DbErr::RecordNotFound(_) => AppendUserMessageError::NotFound,
            _ => AppendUserMessageError::Database(e),
        })?;

    let last_message = message_service::get_last_message_for_conversation(
        &app_state.database_connection,
        conversation_id,
    )
    .await?;
    let last_message = last_message.ok_or(AppendUserMessageError::ConversationBusy)?;
    let status_ok = last_message
        .status
        .parse::<MessageStatus>()
        .ok()
        .is_some_and(|status| TERMINAL_MESSAGE_STATUSES.contains(&status));
    if last_message.sender != Sender::Assistant.to_string() || !status_ok {
        return Err(AppendUserMessageError::ConversationBusy);
    }

    let segments_json = segment::serialize_segments(&payload.segments);
    let plain_text = segment::plain_text_from_segments(&payload.segments);
    let commands =
        command_constants::parse_dedup(&segment::command_names_from_segments(&payload.segments));

    let question: String = if plain_text.trim().is_empty() && !commands.is_empty() {
        String::new()
    } else {
        validate_question(&plain_text)
            .map_err(|e| match e {
                ExecuteCompletionError::EmptyQuestion => AppendUserMessageError::EmptyQuestion,
                ExecuteCompletionError::QuestionTooLong => AppendUserMessageError::QuestionTooLong,
                _ => AppendUserMessageError::Database(sea_orm::DbErr::Custom(
                    "unexpected validation error".to_string(),
                )),
            })?
            .to_string()
    };

    let txn = app_state.database_connection.begin().await?;

    let new_message = CreateConversationMessageDto {
        conversation_id,
        sequence_number: last_message.sequence_number + 1,
        sender: Sender::Human.to_string(),
        content: question.clone(),
        status: MessageStatus::Processed.to_string(),
    };
    let created_message = message_service::create_message(&txn, new_message).await?;

    let user_message_content = CreateConversationMessageContentDto {
        conversation_message_id: created_message.id,
        sequence_number: 1,
        content_type: ContentType::Text.to_string(),
        content: segments_json,
        status: MessageStatus::Processed.to_string(),
    };
    message_content_service::create_message_content(&txn, user_message_content).await?;

    txn.commit().await?;

    Ok(get_conversation(app_state, conversation_id).await?)
}

fn validate_question(question: &str) -> Result<&str, ExecuteCompletionError> {
    let question = question.trim();
    if question.is_empty() {
        return Err(ExecuteCompletionError::EmptyQuestion);
    }
    if question.chars().count() > constants::MAX_QUESTION_LENGTH {
        return Err(ExecuteCompletionError::QuestionTooLong);
    }
    Ok(question)
}

async fn resolve_last_human_question_for_completion(
    app_state: &AppState,
    conversation_id: Uuid,
) -> Result<(String, Vec<Command>, i32), ExecuteCompletionError> {
    let last_message = message_service::get_last_message_for_conversation(
        &app_state.database_connection,
        conversation_id,
    )
    .await?;
    let last_message = last_message.ok_or(ExecuteCompletionError::NoQuestionsToAnswer)?;
    if last_message.sender != Sender::Human.to_string() {
        return Err(ExecuteCompletionError::LastSenderNotHuman);
    }
    let message_contents = message_content_service::list_message_content_for_message(
        &app_state.database_connection,
        last_message.id,
    )
    .await?;

    if message_contents.len() != 1 {
        return Err(ExecuteCompletionError::InvalidLastMessageContent);
    }
    let raw = &message_contents[0].content;

    let text = segment::extract_plain_text(raw);
    let commands = command_constants::parse_dedup(&segment::command_names_from_content(raw));

    let question = if text.trim().is_empty() && !commands.is_empty() {
        String::new()
    } else {
        validate_question(&text)?.to_string()
    };
    Ok((question, commands, last_message.sequence_number))
}

async fn create_conversation_message_for_completion(
    app_state: &AppState,
    conversation_id: Uuid,
    last_message_sequence_number: i32,
) -> Result<Uuid, ExecuteCompletionError> {
    let conversation_message_id = Uuid::new_v4();
    let now = time_utils::now_utc_into();
    let message = entity::conversation_message::Model {
        id: conversation_message_id,
        conversation_id,
        sequence_number: last_message_sequence_number + 1,
        sender: Sender::Assistant.to_string(),
        status: MessageStatus::Processing.to_string(),
        created_at: now,
        updated_at: now,
    };
    let saved =
        message_repository::save_message_connection(&app_state.database_connection, message)
            .await?;
    Ok(saved.id)
}

pub async fn execute_completion(
    app_state: AppState,
    conversation_id: Uuid,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, ExecuteCompletionError>
{
    ensure_ai_live(&app_state).await?;
    let conversation =
        repository::get_conversation(&app_state.database_connection, conversation_id)
            .await
            .map_err(|e| match &e {
                sea_orm::DbErr::RecordNotFound(_) => ExecuteCompletionError::ConversationNotFound,
                _ => ExecuteCompletionError::Database(e),
            })?;
    let (question, commands, last_message_sequence_number) =
        resolve_last_human_question_for_completion(&app_state, conversation_id).await?;
    let connection =
        connection_service::execute_get_connection(&app_state, conversation.connection_id)
            .await
            .map_err(|e| match &e {
                sea_orm::DbErr::RecordNotFound(_) => ExecuteCompletionError::ConnectionNotFound,
                _ => ExecuteCompletionError::Database(e),
            })?;
    let conversation_message_id = create_conversation_message_for_completion(
        &app_state,
        conversation_id,
        last_message_sequence_number,
    )
    .await?;

    let (sse_tx, sse_rx) = mpsc::channel(constants::CHANNEL_SIZE);
    let (persist_tx, mut persist_rx) = mpsc::channel(constants::CHANNEL_SIZE);
    let db = app_state.database_connection.clone();

    tokio::spawn(async move {
        let channels = EventChannels { sse_tx, persist_tx };
        let result = async {
            let conversation_history_turns =
                load_conversation_history(&app_state, conversation_id, conversation_message_id)
                    .await?;
            run_pipeline(
                &app_state,
                &connection,
                &question,
                &commands,
                &conversation_history_turns,
                &channels,
            )
            .await
        }
        .await;
        if let Err(e) = &result {
            let _ = channels
                .send(crate::ai::dto::Event::Error(e.to_string()))
                .await;
        }

        let status = match &result {
            Ok(PipelineOutcome::Completed) => MessageStatus::Processed,
            Ok(PipelineOutcome::AwaitingClarification) => MessageStatus::AwaitingClarification,
            Ok(PipelineOutcome::Rejected) => MessageStatus::Rejected,
            Err(_) => MessageStatus::Failed,
        };
        let _ = message_repository::update_message_status(
            &app_state.database_connection,
            conversation_message_id,
            status.to_string(),
        )
        .await;
    });

    tokio::spawn(
        async move { persist_events(&db, conversation_message_id, &mut persist_rx).await },
    );

    let stream = ReceiverStream::new(sse_rx)
        .map(|agent_event| Ok(Event::default().data(agent_event.to_json_string())));
    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    ))
}

fn resolve_command_decision(
    command: Command,
    history: &[ConversationHistoryTurn],
) -> crate::ai::dto::RouterDecision {
    use crate::ai::dto::{RouterCode, RouterDecision};
    match command {
        Command::Canvas => {
            let has_data = history.iter().any(|t| {
                matches!(
                    t.terminal_status,
                    ConversationHistoryTerminalStatus::Completed
                ) && t.generated_sql.is_some()
            });
            if has_data {
                RouterDecision {
                    code: RouterCode::GenerateCanvas,
                    message: String::new(),
                }
            } else {
                RouterDecision {
                    code: RouterCode::Clarify,
                    message: "I can generate a canvas once we've run at least one query in this conversation. Ask a data question first, then use /canvas.".to_string(),
                }
            }
        }
    }
}

async fn run_pipeline(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    commands: &[Command],
    conversation_history_turns: &[ConversationHistoryTurn],
    channels: &EventChannels,
) -> anyhow::Result<PipelineOutcome> {
    channels.send(crate::ai::dto::Event::Starting).await?;
    channels.send(crate::ai::dto::Event::Routing).await?;
    let decision = if let Some(command) = commands.first() {
        resolve_command_decision(*command, conversation_history_turns)
    } else {
        router_agent::execute(app_state, question, conversation_history_turns).await?
    };
    channels
        .send(crate::ai::dto::Event::Routed {
            decision: decision.clone(),
        })
        .await?;

    match decision.code {
        RouterCode::TextToSql | RouterCode::Followup => {
            let effective_question = {
                let m = decision.message.trim();
                if m.is_empty() {
                    question
                } else {
                    m
                }
            };
            let sql =
                text_to_sql_agent::execute(app_state, connection, effective_question, channels)
                    .await?;
            visualization_agent::execute(app_state, connection, effective_question, &sql, channels)
                .await?;
            send_followup_questions(
                app_state,
                connection,
                effective_question,
                &sql,
                conversation_history_turns,
                channels,
            )
            .await;
            channels.send(crate::ai::dto::Event::Completed).await?;
            Ok(PipelineOutcome::Completed)
        }
        RouterCode::GenerateCanvas => {
            channels
                .send(crate::ai::dto::Event::GeneratingCanvas)
                .await?;
            let plan =
                crate::ai::canvas_generation_agent::execute(app_state, conversation_history_turns)
                    .await?;
            let summary =
                crate::canvas::service::generate_canvas_from_plan(app_state, connection.id, plan)
                    .await?;
            channels
                .send(crate::ai::dto::Event::CanvasGenerated {
                    canvas_id: summary.canvas_id,
                    name: summary.name,
                    description: summary.description,
                    sql_cell_count: summary.sql_cell_count,
                    chart_cell_count: summary.chart_cell_count,
                    dashboard_charts_count: summary.dashboard_charts_count,
                })
                .await?;
            channels.send(crate::ai::dto::Event::Completed).await?;
            Ok(PipelineOutcome::Completed)
        }
        RouterCode::Clarify => {
            channels
                .send(crate::ai::dto::Event::AwaitingClarification)
                .await?;
            Ok(PipelineOutcome::AwaitingClarification)
        }
        RouterCode::RejectOffTopic | RouterCode::RejectUnsafe => {
            channels.send(crate::ai::dto::Event::Rejected).await?;
            Ok(PipelineOutcome::Rejected)
        }
    }
}

async fn send_followup_questions(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    sql: &str,
    conversation_history_turns: &[ConversationHistoryTurn],
    channels: &EventChannels,
) {
    let catalog = match data_catalog::get_data_catalog_of_connection(connection).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("followup questions: failed to load catalog: {e}");
            return;
        }
    };
    let questions = match followup_questions_agent::suggest_followups(
        app_state,
        connection,
        question,
        sql,
        conversation_history_turns,
        &catalog,
    )
    .await
    {
        Ok(q) => q,
        Err(e) => {
            tracing::error!("followup questions: generation failed: {e}");
            return;
        }
    };
    if questions.is_empty() {
        return;
    }
    let _ = channels
        .send(crate::ai::dto::Event::SuggestedFollowups { questions })
        .await;
}

async fn persist_events(
    db: &sea_orm::DatabaseConnection,
    conversation_message_id: Uuid,
    persist_rx: &mut mpsc::Receiver<crate::ai::dto::Event>,
) {
    let mut sequence_number = 1;
    while let Some(event) = persist_rx.recv().await {
        let create_dto = CreateConversationMessageContentDto {
            conversation_message_id,
            sequence_number,
            content_type: ContentType::DataAnalysisResponse.to_string(),
            content: event.to_json_string(),
            status: ConversationStatus::Active.to_string(),
        };
        let _ = message_content_service::create_message_content_connection(db, create_dto).await;
        sequence_number += 1;
    }
}

async fn load_conversation_history(
    app_state: &AppState,
    conversation_id: Uuid,
    current_assistant_message_id: Uuid,
) -> anyhow::Result<Vec<ConversationHistoryTurn>> {
    let messages = message_service::list_messages_for_conversation(
        &app_state.database_connection,
        conversation_id,
        Some(CONVERSATION_HISTORY_MESSAGE_LIMIT as u64),
        true,
    )
    .await
    .map_err(|e| {
        tracing::error!("conversation history: failed to list messages: {e}");
        anyhow::anyhow!("failed to load conversation history: {e}")
    })?;

    let mut pairs: Vec<(ConversationMessageDto, ConversationMessageDto)> = Vec::new();
    let mut current_human: Option<ConversationMessageDto> = None;
    for msg in messages.into_iter().rev() {
        if msg.id == current_assistant_message_id {
            continue;
        }
        if msg.sender == Sender::Human.to_string() {
            current_human = Some(msg);
        } else if msg.sender == Sender::Assistant.to_string() {
            if let Some(h) = current_human.take() {
                if msg.status != MessageStatus::Processing.to_string() {
                    pairs.push((h, msg));
                }
            }
        }
    }

    let mut conversation_history_turns: Vec<ConversationHistoryTurn> = Vec::new();
    for (human, assistant) in pairs {
        let terminal_status = match assistant.status.parse::<MessageStatus>() {
            Ok(MessageStatus::Processed) => ConversationHistoryTerminalStatus::Completed,
            Ok(MessageStatus::AwaitingClarification) => {
                ConversationHistoryTerminalStatus::AwaitingClarification
            }
            Ok(MessageStatus::Rejected) => ConversationHistoryTerminalStatus::Rejected,
            Ok(MessageStatus::Failed) => ConversationHistoryTerminalStatus::Failed,
            _ => continue,
        };

        let user_question = load_message_text(&app_state.database_connection, human.id).await;
        let assistant_message = load_conversation_history_assistant_message(
            &app_state.database_connection,
            assistant.id,
            &terminal_status,
        )
        .await;
        let generated_sql = load_generated_sql(
            &app_state.database_connection,
            assistant.id,
            &terminal_status,
        )
        .await;

        conversation_history_turns.push(ConversationHistoryTurn {
            user_question,
            terminal_status,
            assistant_message,
            generated_sql,
        });
    }
    Ok(conversation_history_turns)
}

async fn load_message_text(db: &sea_orm::DatabaseConnection, message_id: Uuid) -> String {
    match message_content_service::list_message_content_for_message(db, message_id).await {
        Ok(contents) => contents
            .into_iter()
            .next()
            .map(|c| segment::extract_plain_text(&c.content))
            .unwrap_or_default(),
        Err(_) => String::new(),
    }
}

async fn load_conversation_history_assistant_message(
    db: &sea_orm::DatabaseConnection,
    assistant_message_id: Uuid,
    terminal_status: &ConversationHistoryTerminalStatus,
) -> Option<String> {
    let contents =
        match message_content_service::list_message_content_for_message(db, assistant_message_id)
            .await
        {
            Ok(c) => c,
            Err(_) => return None,
        };

    match terminal_status {
        ConversationHistoryTerminalStatus::AwaitingClarification
        | ConversationHistoryTerminalStatus::Rejected => {
            for c in contents.iter() {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&c.content) {
                    if value["event_name"] == "routed" {
                        if let Some(msg) = value["data"]["decision"]["message"].as_str() {
                            return Some(msg.to_string());
                        }
                    }
                }
            }
            None
        }
        ConversationHistoryTerminalStatus::Completed => Some("<query executed>".to_string()),
        ConversationHistoryTerminalStatus::Failed => None,
    }
}

async fn load_generated_sql(
    db: &sea_orm::DatabaseConnection,
    assistant_message_id: Uuid,
    terminal_status: &ConversationHistoryTerminalStatus,
) -> Option<String> {
    match terminal_status {
        ConversationHistoryTerminalStatus::AwaitingClarification
        | ConversationHistoryTerminalStatus::Rejected => return None,
        ConversationHistoryTerminalStatus::Completed
        | ConversationHistoryTerminalStatus::Failed => {}
    }

    let contents =
        match message_content_service::list_message_content_for_message(db, assistant_message_id)
            .await
        {
            Ok(c) => c,
            Err(_) => return None,
        };
    let mut generated_sql: Option<String> = None;
    for c in contents {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&c.content) {
            if value["event_name"] == "generated_sql" {
                if let Some(sql) = value["data"]["sql"].as_str() {
                    generated_sql = Some(sql.to_string());
                }
            }
        }
    }
    generated_sql
}
