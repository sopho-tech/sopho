use super::constants;
use super::constants::ConversationStatus;
use super::dto;
use super::error::ConversationError;
use super::error::ExecuteCompletionError;
use super::repository;
use crate::ai::conversation_name_agent;
use crate::ai::dto::EventChannels;
use crate::ai::text_to_sql_agent;
use crate::ai::visualization_agent;
use crate::ai_configuration::dto::AiConfigurationStatus;
use crate::ai_configuration::service as ai_config_service;
use crate::common::time_utils;
use crate::common::AppState;
use crate::connection::service as connection_service;
use crate::conversational_analytics::conversation::constants::ContentType;
use crate::conversational_analytics::conversation::constants::MessageStatus;
use crate::conversational_analytics::message::constants::Sender;
use crate::conversational_analytics::message::dto::{
    ConversationMessageWithContentDto, CreateConversationMessageDto,
};
use crate::conversational_analytics::message::repository as message_repository;
use crate::conversational_analytics::message::service as message_service;
use crate::conversational_analytics::message_content::dto::{
    ConversationMessageContentDto, CreateConversationMessageContentDto,
};
use crate::conversational_analytics::message_content::service as message_content_service;
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
    let status = ai_config_service::get_status(app_state).await.map_err(
        |e| match e {
            crate::ai_configuration::error::AiConfigurationError::Database(d) => {
                ConversationError::Database(d)
            }
            other => ConversationError::Conversion(other.to_string()),
        },
    )?;
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

pub async fn get_all_conversations(
    app_state: AppState,
) -> Result<Vec<dto::ConversationDto>, ConversationError> {
    let models = repository::get_all_conversations(&app_state.database_connection).await?;
    models
        .into_iter()
        .map(dto::ConversationDto::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ConversationError::Conversion)
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
    let txn = app_state.database_connection.begin().await?;

    let saved_conversation = repository::save_conversation_transaction(&txn, conversation).await?;

    let user_message = CreateConversationMessageDto::initial_user_message(
        saved_conversation.id,
        &payload.user_message,
    );
    let created_message = message_service::create_message(&txn, user_message).await?;

    let user_message_content = CreateConversationMessageContentDto {
        conversation_message_id: created_message.id,
        sequence_number: 1,
        content_type: ContentType::Text.to_string(),
        content: payload.user_message.clone(),
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
) -> Result<(String, i32), ExecuteCompletionError> {
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
    let message_content = &message_contents[0];

    let question = validate_question(&message_content.content)?;
    Ok((question.to_string(), last_message.sequence_number))
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
    let (question, last_message_sequence_number) =
        resolve_last_human_question_for_completion(&app_state, conversation_id).await?;

    let conversation =
        repository::get_conversation(&app_state.database_connection, conversation_id)
            .await
            .map_err(|e| match &e {
                sea_orm::DbErr::RecordNotFound(_) => ExecuteCompletionError::ConversationNotFound,
                _ => ExecuteCompletionError::Database(e),
            })?;

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
        let result = run_pipeline(&app_state, &connection, &question, &channels).await;
        if let Err(e) = &result {
            let _ = channels
                .send(crate::ai::dto::Event::Error(e.to_string()))
                .await;
        }
        let _ = channels.send(crate::ai::dto::Event::Completed).await;

        let status = if result.is_ok() {
            MessageStatus::Processed
        } else {
            MessageStatus::Failed
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

async fn run_pipeline(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    channels: &EventChannels,
) -> anyhow::Result<()> {
    channels.send(crate::ai::dto::Event::Starting).await?;
    let sql = text_to_sql_agent::execute(app_state, connection, question, channels).await?;
    visualization_agent::execute(app_state, connection, question, &sql, channels).await?;
    Ok(())
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
