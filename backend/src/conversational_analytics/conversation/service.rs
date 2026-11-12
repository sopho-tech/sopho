use super::canvas;
use super::constants;
use super::constants::{CONNECTION_NOT_FOUND_MESSAGE, CONVERSATION_NOT_FOUND_MESSAGE};
use super::constants::{ConversationStatus, PipelineOutcome};
use super::dto;
use super::error::AppendUserMessageError;
use super::error::ConversationError;
use super::error::ExecuteCompletionError;
use super::history;
use super::repository;
use crate::ai::conversation_name_agent;
use crate::ai::dto::EventChannels;
use crate::ai::dto::{ConversationHistory, RouterCode};
use crate::ai::followup_questions_agent;
use crate::ai::result_narration_agent;
use crate::ai::router_agent;
use crate::ai::text_to_sql_agent;
use crate::ai::visualization_agent;
use crate::ai_configuration::dto::AiConfigurationStatus;
use crate::ai_configuration::service as ai_config_service;
use crate::attribution::service::{self as attribution_service, AttributionNames};
use crate::authorization::constants::{Action, Permission, ResourceType, Visibility};
use crate::authorization::dto::Actor;
use crate::authorization::dto::{AuthorizationSubject, TenantContext, TenantId};
use crate::authorization::errors::AuthorizationError;
use crate::authorization::service as authorization_service;
use crate::canvas::dto::CanvasAuthorship;
use crate::common::AppState;
use crate::common::errors::GetDatabaseConnectionError;
use crate::common::time_utils;
use crate::connection::service as connection_service;
use crate::conversational_analytics::command::constants::{self as command_constants, Command};
use crate::conversational_analytics::conversation::constants::ContentType;
use crate::conversational_analytics::conversation::constants::MessageStatus;
use crate::conversational_analytics::conversation::constants::TERMINAL_MESSAGE_STATUSES;
use crate::conversational_analytics::message::constants::Sender;
use crate::conversational_analytics::message::dto::{
    ConversationMessageWithContentDto, CreateConversationMessageDto,
};
use crate::conversational_analytics::message::service as message_service;
use crate::conversational_analytics::message_content::dto::{
    ConversationMessageContentDto, CreateConversationMessageContentDto,
};
use crate::conversational_analytics::message_content::service as message_content_service;
use crate::conversational_analytics::segment;
use crate::data_catalog;
use crate::database::service::execute_sql_query;
use crate::entity;
use axum::response::sse::{Event, Sse};
use sea_orm::TransactionTrait;
use std::collections::HashMap;
use std::{convert::Infallible, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

async fn ensure_ai_live(
    app_state: &AppState,
    tenant_id: &TenantId,
) -> Result<(), ConversationError> {
    let status = ai_config_service::execute_get_status(app_state, tenant_id)
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

fn reachable_conversation_condition(tenant_context: &TenantContext) -> sea_orm::Condition {
    authorization_service::reachable_resource_condition(
        tenant_context,
        ResourceType::Conversation,
        entity::conversation::Column::Id,
        entity::conversation::Column::OwnerUserId,
        entity::conversation::Column::Visibility,
    )
}

async fn authorize_conversation(
    app_state: &AppState,
    tenant_context: &TenantContext,
    conversation_id: Uuid,
    action: Action,
) -> Result<entity::conversation::Model, ConversationError> {
    let conversation = repository::get_conversation(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        conversation_id,
    )
    .await
    .map_err(|_| ConversationError::NotFound)?
    .ok_or(ConversationError::NotFound)?;

    let subject = authorization_service::subject_from_parts(
        tenant_context.tenant_id(),
        ResourceType::Conversation,
        conversation.id,
        conversation.owner_user_id,
        &conversation.visibility,
    )
    .map_err(|_| ConversationError::NotFound)?;

    authorization_service::require_access(app_state, tenant_context, &subject, action)
        .await
        .map_err(|_| ConversationError::NotFound)?;

    Ok(conversation)
}

fn conversation_subject(
    tenant_context: &TenantContext,
    conversation: &entity::conversation::Model,
) -> Result<AuthorizationSubject, ConversationError> {
    authorization_service::subject_from_parts(
        tenant_context.tenant_id(),
        ResourceType::Conversation,
        conversation.id,
        conversation.owner_user_id,
        &conversation.visibility,
    )
    .map_err(|_| ConversationError::NotFound)
}

async fn conversation_response_dto(
    app_state: &AppState,
    tenant_context: &TenantContext,
    model: entity::conversation::Model,
) -> Result<dto::ConversationDto, ConversationError> {
    let names = attribution_service::load_names(
        app_state,
        [model.created_by_user_id, model.modified_by_user_id],
    )
    .await?;
    conversation_response_dto_with_names(app_state, tenant_context, model, &names).await
}

async fn conversation_response_dto_with_names(
    app_state: &AppState,
    tenant_context: &TenantContext,
    model: entity::conversation::Model,
    names: &AttributionNames,
) -> Result<dto::ConversationDto, ConversationError> {
    let subject = conversation_subject(tenant_context, &model)?;
    let created_by = names.describe(&model.created_by_actor, model.created_by_user_id)?;
    let modified_by = names.describe(&model.modified_by_actor, model.modified_by_user_id)?;
    let mut conversation =
        dto::ConversationDto::try_from(model).map_err(ConversationError::Conversion)?;
    conversation.created_by = Some(created_by);
    conversation.modified_by = Some(modified_by);
    conversation.capabilities = Some(
        authorization_service::resolve_capabilities(app_state, tenant_context, &subject)
            .await
            .map_err(|error| match error {
                AuthorizationError::Database(database_error) => {
                    ConversationError::Database(database_error)
                }
                _ => ConversationError::NotFound,
            })?,
    );
    Ok(conversation)
}

pub async fn get_conversation(
    app_state: AppState,
    tenant_context: &TenantContext,
    conversation_id: Uuid,
) -> Result<dto::ConversationWithMessagesDto, ConversationError> {
    authorize_conversation(&app_state, tenant_context, conversation_id, Action::Read).await?;
    let tenant_id = tenant_context.tenant_id();
    let model =
        repository::get_conversation(&app_state.database_connection, tenant_id, conversation_id)
            .await
            .map_err(ConversationError::Database)?
            .ok_or(ConversationError::NotFound)?;
    let conversation = conversation_response_dto(&app_state, tenant_context, model).await?;
    let messages = message_service::list_messages_for_conversation(
        &app_state.database_connection,
        tenant_id,
        conversation_id,
        None,
        false,
    )
    .await?;
    let message_ids: Vec<Uuid> = messages.iter().map(|m| m.id).collect();
    let contents = message_content_service::list_message_content_for_messages(
        &app_state.database_connection,
        tenant_id,
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
    let should_execute_completion = messages.last().is_some_and(is_human_message);
    let user_message_limit_reached = constants::user_message_limit_reached(
        messages.iter().filter(|m| is_human_message(m)).count(),
    );
    Ok(dto::ConversationWithMessagesDto {
        conversation,
        messages,
        should_execute_completion,
        user_message_limit_reached,
    })
}

fn is_human_message(message: &ConversationMessageWithContentDto) -> bool {
    matches!(message.message.sender.parse::<Sender>(), Ok(Sender::Human))
}

fn normalize_search(search: Option<String>) -> Option<String> {
    search
        .map(|term| term.trim().to_string())
        .filter(|term| !term.is_empty())
}

pub async fn list_conversations(
    app_state: AppState,
    tenant_context: &TenantContext,
    query: dto::ListConversationsQuery,
) -> Result<dto::PaginatedConversationsDto, ConversationError> {
    authorization_service::require_capability(
        tenant_context,
        ResourceType::Conversation.permission_for(Action::Read),
    )
    .map_err(|_| ConversationError::NotFound)?;
    let tenant_id = tenant_context.tenant_id();
    let page = query.page.unwrap_or(0);
    let page_size = query
        .page_size
        .unwrap_or(constants::DEFAULT_PAGE_SIZE)
        .clamp(1, constants::MAX_PAGE_SIZE);
    let search = normalize_search(query.search);

    let (models, total) = repository::list_conversations(
        &app_state.database_connection,
        tenant_id,
        reachable_conversation_condition(tenant_context),
        search.as_deref(),
        page,
        page_size,
    )
    .await?;

    let conversation_ids: Vec<Uuid> = models.iter().map(|model| model.id).collect();
    let user_message_counts = message_service::count_messages_by_sender_for_conversations(
        &app_state.database_connection,
        tenant_id,
        &conversation_ids,
        &Sender::Human.to_string(),
    )
    .await?;

    let names = attribution_service::load_names(
        &app_state,
        models
            .iter()
            .flat_map(|model| [model.created_by_user_id, model.modified_by_user_id]),
    )
    .await?;
    let mut items = Vec::with_capacity(models.len());
    for model in models {
        let user_message_count = user_message_counts
            .get(&model.id)
            .copied()
            .unwrap_or_default();
        let conversation =
            conversation_response_dto_with_names(&app_state, tenant_context, model, &names).await?;
        items.push(dto::ConversationListItemDto {
            conversation,
            user_message_count,
        });
    }

    Ok(dto::PaginatedConversationsDto {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn search_conversations_by_name(
    app_state: &AppState,
    tenant_context: &TenantContext,
    search_query: Option<&str>,
    limit: u64,
) -> Result<Vec<entity::conversation::Model>, sea_orm::DbErr> {
    repository::list_conversations(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        reachable_conversation_condition(tenant_context),
        search_query,
        0,
        limit,
    )
    .await
    .map(|(conversations, _)| conversations)
}

/// Sequence number of conversation messages start from `1` not `0`
pub async fn create_conversation(
    app_state: AppState,
    tenant_context: &TenantContext,
    payload: dto::CreateConversationDto,
) -> Result<dto::ConversationDto, ConversationError> {
    authorization_service::require_capability(tenant_context, Permission::ConversationCreate)
        .map_err(|_| ConversationError::NotFound)?;
    connection_service::require_connection_access(
        &app_state,
        tenant_context,
        payload.connection_id,
        Action::Read,
    )
    .await
    .map_err(|error| match error {
        AuthorizationError::Database(database_error) => ConversationError::Database(database_error),
        _ => ConversationError::InvalidRequest(CONNECTION_NOT_FOUND_MESSAGE.to_string()),
    })?;
    let tenant_id = tenant_context.tenant_id();
    ensure_ai_live(&app_state, tenant_id).await?;
    let actor = tenant_context.actor();
    let now = time_utils::now_utc_into();
    let conversation_id = Uuid::new_v4();
    let conversation = entity::conversation::Model {
        id: conversation_id,
        tenant_id: tenant_id.get_uuid(),
        owner_user_id: tenant_context.user_id().get_uuid(),
        visibility: Visibility::Tenant.as_str().to_string(),
        connection_id: payload.connection_id,
        name: constants::DEFAULT_CONVERSATION_NAME.to_string(),
        status: ConversationStatus::Active.to_string(),
        created_by_user_id: actor.user_id(),
        created_by_actor: actor.kind_name(),
        modified_by_user_id: actor.user_id(),
        modified_by_actor: actor.kind_name(),
        created_at: now,
        updated_at: now,
    };
    let segments_json = segment::serialize_segments(&payload.segments);
    let plain_text = segment::plain_text_from_segments(&payload.segments);

    let txn = app_state.database_connection.begin().await?;

    let saved_conversation =
        repository::save_conversation_transaction(&txn, tenant_id, conversation).await?;

    let user_message =
        CreateConversationMessageDto::initial_user_message(saved_conversation.id, &plain_text);
    let created_message =
        message_service::create_message(&txn, tenant_id, user_message, &actor).await?;

    let user_message_content = CreateConversationMessageContentDto {
        conversation_message_id: created_message.id,
        sequence_number: 1,
        content_type: ContentType::Text.to_string(),
        content: segments_json,
        status: MessageStatus::Processed.to_string(),
    };
    message_content_service::create_message_content(&txn, tenant_id, user_message_content, &actor)
        .await?;

    txn.commit().await?;
    conversation_response_dto(&app_state, tenant_context, saved_conversation).await
}

pub async fn update_conversation(
    app_state: AppState,
    tenant_context: &TenantContext,
    conversation_id: Uuid,
    payload: dto::ConversationDto,
) -> Result<dto::ConversationDto, ConversationError> {
    authorize_conversation(&app_state, tenant_context, conversation_id, Action::Write).await?;
    let tenant_id = tenant_context.tenant_id();
    let model = repository::update_conversation(
        &app_state.database_connection,
        tenant_id,
        conversation_id,
        payload,
        &tenant_context.actor(),
    )
    .await?
    .ok_or(ConversationError::NotFound)?;
    conversation_response_dto(&app_state, tenant_context, model).await
}

pub async fn delete_conversation(
    app_state: AppState,
    tenant_context: &TenantContext,
    conversation_id: Uuid,
) -> Result<(), ConversationError> {
    authorize_conversation(&app_state, tenant_context, conversation_id, Action::Delete).await?;
    let tenant_id = tenant_context.tenant_id();
    let _ =
        repository::get_conversation(&app_state.database_connection, tenant_id, conversation_id)
            .await
            .map_err(ConversationError::Database)?
            .ok_or(ConversationError::NotFound)?;
    let txn = app_state.database_connection.begin().await?;
    message_content_service::delete_message_contents_for_conversation_transaction(
        &txn,
        tenant_id,
        conversation_id,
    )
    .await?;
    message_service::delete_messages_for_conversation_transaction(&txn, tenant_id, conversation_id)
        .await?;
    repository::delete_conversation_transaction(&txn, tenant_id, conversation_id)
        .await
        .map_err(ConversationError::Database)?;
    txn.commit().await.map_err(ConversationError::Database)?;
    Ok(())
}

pub async fn bulk_delete_conversations(
    app_state: AppState,
    tenant_context: &TenantContext,
    payload: dto::BulkDeleteConversationsDto,
) -> Result<(), ConversationError> {
    let tenant_id = tenant_context.tenant_id();
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

    let existing_count = repository::count_conversations_by_ids(
        &app_state.database_connection,
        tenant_id,
        &conversation_ids,
    )
    .await?;
    if existing_count != conversation_ids.len() as u64 {
        return Err(ConversationError::NotFound);
    }

    for conversation_id in &conversation_ids {
        authorize_conversation(&app_state, tenant_context, *conversation_id, Action::Delete)
            .await?;
    }

    let txn = app_state.database_connection.begin().await?;
    message_content_service::delete_message_contents_for_conversations_transaction(
        &txn,
        tenant_id,
        &conversation_ids,
    )
    .await?;
    message_service::delete_messages_for_conversations_transaction(
        &txn,
        tenant_id,
        &conversation_ids,
    )
    .await?;
    repository::delete_conversations_transaction(&txn, tenant_id, &conversation_ids)
        .await
        .map_err(ConversationError::Database)?;
    txn.commit().await.map_err(ConversationError::Database)?;
    Ok(())
}

pub async fn suggest_conversation_name(
    app_state: AppState,
    tenant_context: &TenantContext,
    conversation_id: Uuid,
) -> Result<dto::ConversationDto, ConversationError> {
    authorize_conversation(&app_state, tenant_context, conversation_id, Action::Write).await?;
    let tenant_id = tenant_context.tenant_id();
    ensure_ai_live(&app_state, tenant_id).await?;
    let conversation =
        repository::get_conversation(&app_state.database_connection, tenant_id, conversation_id)
            .await
            .map_err(ConversationError::Database)?
            .ok_or(ConversationError::NotFound)?;

    let first_message = message_service::get_first_message_for_conversation(
        &app_state.database_connection,
        tenant_id,
        conversation_id,
    )
    .await?;

    let first_message = first_message.ok_or(ConversationError::NotFound)?;

    let question = message_content_service::get_first_message_content(
        &app_state.database_connection,
        tenant_id,
        first_message.id,
    )
    .await?
    .map(|c| c.content)
    .unwrap_or_default();

    let suggested_name = conversation_name_agent::suggest_name(&app_state, tenant_id, &question)
        .await
        .unwrap_or_else(|_| conversation.name.clone());

    let updated_payload = dto::ConversationDto {
        id: conversation_id,
        connection_id: conversation.connection_id,
        name: suggested_name,
        status: constants::ConversationStatus::Active,
        created_at: conversation.created_at,
        updated_at: conversation.updated_at,
        capabilities: None,
        created_by: None,
        modified_by: None,
    };

    let model = repository::update_conversation(
        &app_state.database_connection,
        tenant_id,
        conversation_id,
        updated_payload,
        &tenant_context.ai_actor(),
    )
    .await?
    .ok_or(ConversationError::NotFound)?;

    conversation_response_dto(&app_state, tenant_context, model).await
}

pub async fn append_user_message(
    app_state: AppState,
    tenant_context: &TenantContext,
    conversation_id: Uuid,
    payload: dto::AppendUserMessageDto,
) -> Result<dto::ConversationWithMessagesDto, AppendUserMessageError> {
    authorize_conversation(&app_state, tenant_context, conversation_id, Action::Write)
        .await
        .map_err(|_| AppendUserMessageError::NotFound)?;
    let tenant_id = tenant_context.tenant_id();
    ensure_ai_live(&app_state, tenant_id).await?;

    repository::get_conversation(&app_state.database_connection, tenant_id, conversation_id)
        .await
        .map_err(|e| match &e {
            sea_orm::DbErr::RecordNotFound(_) => AppendUserMessageError::NotFound,
            _ => AppendUserMessageError::Database(e),
        })?;

    let last_message = message_service::get_last_message_for_conversation(
        &app_state.database_connection,
        tenant_id,
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

    let user_message_count = message_service::count_user_messages_for_conversation(
        &app_state.database_connection,
        tenant_id,
        conversation_id,
    )
    .await?;
    if constants::user_message_limit_reached(user_message_count as usize) {
        return Err(AppendUserMessageError::UserMessageLimitReached);
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
    let actor = tenant_context.actor();
    let created_message =
        message_service::create_message(&txn, tenant_id, new_message, &actor).await?;

    let user_message_content = CreateConversationMessageContentDto {
        conversation_message_id: created_message.id,
        sequence_number: 1,
        content_type: ContentType::Text.to_string(),
        content: segments_json,
        status: MessageStatus::Processed.to_string(),
    };
    message_content_service::create_message_content(&txn, tenant_id, user_message_content, &actor)
        .await?;

    txn.commit().await?;

    Ok(get_conversation(app_state, tenant_context, conversation_id).await?)
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
    tenant_id: &TenantId,
    conversation_id: Uuid,
) -> Result<(String, Vec<Command>, i32), ExecuteCompletionError> {
    let last_message = message_service::get_last_message_for_conversation(
        &app_state.database_connection,
        tenant_id,
        conversation_id,
    )
    .await?;
    let last_message = last_message.ok_or(ExecuteCompletionError::NoQuestionsToAnswer)?;
    if last_message.sender != Sender::Human.to_string() {
        return Err(ExecuteCompletionError::LastSenderNotHuman);
    }
    let message_contents = message_content_service::list_message_content_for_message(
        &app_state.database_connection,
        tenant_id,
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
    tenant_id: &TenantId,
    conversation_id: Uuid,
    last_message_sequence_number: i32,
    actor: &Actor,
) -> Result<Uuid, ExecuteCompletionError> {
    let transaction = app_state.database_connection.begin().await?;
    let created_message = message_service::create_message(
        &transaction,
        tenant_id,
        CreateConversationMessageDto {
            conversation_id,
            sequence_number: last_message_sequence_number + 1,
            sender: Sender::Assistant.to_string(),
            content: String::new(),
            status: MessageStatus::Processing.to_string(),
        },
        actor,
    )
    .await?;
    transaction.commit().await?;
    Ok(created_message.id)
}

pub async fn execute_completion(
    app_state: AppState,
    tenant_context: &TenantContext,
    conversation_id: Uuid,
) -> Result<
    Sse<impl futures_util::Stream<Item = Result<Event, Infallible>> + use<>>,
    ExecuteCompletionError,
> {
    authorize_conversation(&app_state, tenant_context, conversation_id, Action::Write)
        .await
        .map_err(|_| ExecuteCompletionError::ConversationNotFound)?;
    let tenant_id = tenant_context.tenant_id();
    ensure_ai_live(&app_state, tenant_id).await?;
    let conversation =
        repository::get_conversation(&app_state.database_connection, tenant_id, conversation_id)
            .await
            .map_err(ExecuteCompletionError::Database)?
            .ok_or(ExecuteCompletionError::ConversationNotFound)?;
    let (question, commands, last_message_sequence_number) =
        resolve_last_human_question_for_completion(&app_state, tenant_id, conversation_id).await?;
    let connection = connection_service::execute_get_connection(
        &app_state,
        tenant_id,
        conversation.connection_id,
    )
    .await
    .map_err(|e| match e {
        GetDatabaseConnectionError::ConnectionNotFound => {
            ExecuteCompletionError::ConnectionNotFound
        }
        other => ExecuteCompletionError::GetConnection(other),
    })?;
    connection_service::ensure_connection_can_be_used(&app_state, &connection)
        .map_err(ExecuteCompletionError::GetConnection)?;
    let ai_actor = tenant_context.ai_actor();
    let conversation_message_id = create_conversation_message_for_completion(
        &app_state,
        tenant_id,
        conversation_id,
        last_message_sequence_number,
        &ai_actor,
    )
    .await?;

    let (sse_tx, sse_rx) = mpsc::channel(constants::CHANNEL_SIZE);
    let (persist_tx, persist_rx) = mpsc::channel(constants::CHANNEL_SIZE);
    let db = app_state.database_connection.clone();
    let tenant_id = tenant_id.clone();
    let persist_tenant_id = tenant_id.clone();

    let persistence_task = tokio::spawn(async move {
        persist_events(
            &db,
            &persist_tenant_id,
            conversation_message_id,
            persist_rx,
            &ai_actor,
        )
        .await
    });

    tokio::spawn(async move {
        let tenant_id = &tenant_id;
        let channels = EventChannels { sse_tx, persist_tx };
        let result = async {
            let conversation_history = history::load(
                &app_state,
                tenant_id,
                conversation_id,
                conversation_message_id,
            )
            .await?;
            run_pipeline(
                &app_state,
                tenant_id,
                &connection,
                conversation_id,
                PipelineRequest {
                    question: &question,
                    commands: &commands,
                    conversation_history: &conversation_history,
                },
                &channels,
                &ai_actor,
            )
            .await
        }
        .await;

        let (final_event, status) = match result {
            Ok(PipelineOutcome::Completed) => {
                (crate::ai::dto::Event::Completed, MessageStatus::Processed)
            }
            Ok(PipelineOutcome::AwaitingClarification) => (
                crate::ai::dto::Event::AwaitingClarification,
                MessageStatus::AwaitingClarification,
            ),
            Ok(PipelineOutcome::Rejected) => {
                (crate::ai::dto::Event::Rejected, MessageStatus::Rejected)
            }
            Err(e) => (
                crate::ai::dto::Event::Error(e.to_string()),
                MessageStatus::Failed,
            ),
        };

        let EventChannels { sse_tx, persist_tx } = channels;
        let final_event = complete_assistant_message(
            &app_state.database_connection,
            tenant_id,
            conversation_message_id,
            EventPersistence {
                persist_tx,
                persistence_task,
            },
            final_event,
            status,
            &ai_actor,
        )
        .await;
        if let Err(e) = sse_tx.send(final_event).await {
            tracing::error!(
                "conversation message {conversation_message_id}: failed to send the final event: {e}"
            );
        }
    });

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
    history: &ConversationHistory,
) -> crate::ai::dto::RouterDecision {
    use crate::ai::dto::{RouterCode, RouterDecision};
    match command {
        Command::Canvas => {
            if history.has_completed_query() {
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

struct PipelineRequest<'a> {
    question: &'a str,
    commands: &'a [Command],
    conversation_history: &'a ConversationHistory,
}

async fn run_pipeline(
    app_state: &AppState,
    tenant_id: &TenantId,
    connection: &entity::connection::Model,
    conversation_id: Uuid,
    request: PipelineRequest<'_>,
    channels: &EventChannels,
    actor: &Actor,
) -> anyhow::Result<PipelineOutcome> {
    let PipelineRequest {
        question,
        commands,
        conversation_history,
    } = request;
    channels.send(crate::ai::dto::Event::Starting).await?;
    channels.send(crate::ai::dto::Event::Routing).await?;
    let decision = if let Some(command) = commands.first() {
        resolve_command_decision(*command, conversation_history)
    } else {
        router_agent::execute(app_state, tenant_id, question, conversation_history).await?
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
                if m.is_empty() { question } else { m }
            };
            let sql = text_to_sql_agent::execute(
                app_state,
                tenant_id,
                connection,
                effective_question,
                channels,
            )
            .await?;

            let followup_questions_handle = {
                let app_state = app_state.clone();
                let connection = connection.clone();
                let followup_question = effective_question.to_string();
                let followup_sql = sql.clone();
                let conversation_history = conversation_history.clone();
                let followup_tenant_id = tenant_id.clone();
                tokio::spawn(async move {
                    compute_followup_questions(
                        &app_state,
                        &followup_tenant_id,
                        &connection,
                        &followup_question,
                        &followup_sql,
                        &conversation_history,
                    )
                    .await
                })
            };

            channels
                .send_sse_only(crate::ai::dto::Event::ExecutingQuery)
                .await;
            let query_result = execute_sql_query(app_state, connection, &sql).await?;
            channels
                .send_sse_only(crate::ai::dto::Event::ExecutedQuery {
                    columns: query_result.columns.clone(),
                    data: query_result.data.clone(),
                })
                .await;

            let (_, visualization_result) = tokio::join!(
                result_narration_agent::execute(
                    app_state,
                    tenant_id,
                    effective_question,
                    &sql,
                    &query_result,
                    channels
                ),
                visualization_agent::execute(
                    app_state,
                    tenant_id,
                    effective_question,
                    &query_result,
                    channels
                ),
            );
            visualization_result?;

            let followup_questions = followup_questions_handle.await.unwrap_or_default();
            if !followup_questions.is_empty() {
                let _ = channels
                    .send(crate::ai::dto::Event::SuggestedFollowups {
                        questions: followup_questions,
                    })
                    .await;
            }
            Ok(PipelineOutcome::Completed)
        }
        RouterCode::GenerateCanvas => {
            let conversation_subject =
                load_authorization_subject(app_state, tenant_id, conversation_id)
                    .await
                    .map_err(|error| anyhow::anyhow!(error))?;
            canvas::generate(
                app_state,
                tenant_id,
                CanvasAuthorship {
                    ownership: conversation_subject.ownership(),
                    actor: *actor,
                },
                conversation_id,
                connection.id,
                conversation_history,
                channels,
            )
            .await?;
            Ok(PipelineOutcome::Completed)
        }
        RouterCode::Clarify => Ok(PipelineOutcome::AwaitingClarification),
        RouterCode::RejectOffTopic | RouterCode::RejectUnsafe => Ok(PipelineOutcome::Rejected),
    }
}

async fn compute_followup_questions(
    app_state: &AppState,
    tenant_id: &TenantId,
    connection: &entity::connection::Model,
    question: &str,
    sql: &str,
    conversation_history: &ConversationHistory,
) -> Vec<String> {
    let catalog = match data_catalog::get_data_catalog_of_connection(app_state, connection).await {
        Ok(catalog) => catalog,
        Err(error) => {
            tracing::error!("followup questions: failed to load catalog: {error}");
            return Vec::new();
        }
    };
    match followup_questions_agent::suggest_followups(
        app_state,
        tenant_id,
        connection,
        question,
        sql,
        conversation_history,
        &catalog,
    )
    .await
    {
        Ok(questions) => questions,
        Err(error) => {
            tracing::error!("followup questions: generation failed: {error}");
            Vec::new()
        }
    }
}

fn content_type_for_event(event: &crate::ai::dto::Event) -> ContentType {
    match event {
        crate::ai::dto::Event::Narrated { .. } => ContentType::Narration,
        _ => ContentType::DataAnalysisResponse,
    }
}

async fn persist_events(
    db: &sea_orm::DatabaseConnection,
    tenant_id: &TenantId,
    conversation_message_id: Uuid,
    mut persist_rx: mpsc::Receiver<crate::ai::dto::Event>,
    actor: &Actor,
) -> Result<(), ConversationError> {
    let mut sequence_number = 1;
    while let Some(event) = persist_rx.recv().await {
        let create_dto = CreateConversationMessageContentDto {
            conversation_message_id,
            sequence_number,
            content_type: content_type_for_event(&event).to_string(),
            content: event.to_json_string(),
            status: ConversationStatus::Active.to_string(),
        };
        message_content_service::create_message_content_connection(
            db, tenant_id, create_dto, actor,
        )
        .await?;
        sequence_number += 1;
    }
    Ok(())
}

struct EventPersistence {
    persist_tx: mpsc::Sender<crate::ai::dto::Event>,
    persistence_task: tokio::task::JoinHandle<Result<(), ConversationError>>,
}

async fn persist_final_event(
    persist_tx: mpsc::Sender<crate::ai::dto::Event>,
    persistence_task: tokio::task::JoinHandle<Result<(), ConversationError>>,
    final_event: crate::ai::dto::Event,
) -> anyhow::Result<()> {
    let final_event_queued = persist_tx.send(final_event).await;
    drop(persist_tx);
    persistence_task.await??;
    final_event_queued?;
    Ok(())
}

async fn complete_assistant_message(
    db: &sea_orm::DatabaseConnection,
    tenant_id: &TenantId,
    conversation_message_id: Uuid,
    event_persistence: EventPersistence,
    final_event: crate::ai::dto::Event,
    status: MessageStatus,
    actor: &Actor,
) -> crate::ai::dto::Event {
    let (final_event, status) = match persist_final_event(
        event_persistence.persist_tx,
        event_persistence.persistence_task,
        final_event.clone(),
    )
    .await
    {
        Ok(()) => (final_event, status),
        Err(e) => {
            tracing::error!(
                "conversation message {conversation_message_id}: failed to save the answer: {e:#}"
            );
            (
                crate::ai::dto::Event::Error(format!("Failed to save the answer: {e}")),
                MessageStatus::Failed,
            )
        }
    };

    match message_service::update_message_status(
        db,
        tenant_id,
        conversation_message_id,
        status.to_string(),
        actor,
    )
    .await
    {
        Ok(()) => final_event,
        Err(e) => {
            tracing::error!(
                "conversation message {conversation_message_id}: failed to update the status: {e}"
            );
            crate::ai::dto::Event::Error(format!("Failed to save the answer status: {e}"))
        }
    }
}

pub async fn load_authorization_subject(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<AuthorizationSubject, sea_orm::DbErr> {
    let conversation = repository::get_conversation(&app_state.database_connection, tenant_id, id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(CONVERSATION_NOT_FOUND_MESSAGE.into()))?;
    authorization_service::subject_from_parts(
        tenant_id,
        ResourceType::Conversation,
        conversation.id,
        conversation.owner_user_id,
        &conversation.visibility,
    )
    .map_err(sea_orm::DbErr::Custom)
}

pub async fn set_visibility(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
    visibility: Visibility,
    actor: &Actor,
) -> Result<(), sea_orm::DbErr> {
    let updated_rows = repository::update_conversation_visibility(
        &app_state.database_connection,
        tenant_id,
        id,
        visibility.as_str(),
        actor,
    )
    .await?;
    if updated_rows == 0 {
        return Err(sea_orm::DbErr::RecordNotFound(
            CONVERSATION_NOT_FOUND_MESSAGE.into(),
        ));
    }
    Ok(())
}

pub async fn stamp_modified(
    connection: &impl sea_orm::ConnectionTrait,
    tenant_id: &TenantId,
    id: Uuid,
    actor: &Actor,
) -> Result<(), sea_orm::DbErr> {
    let stamped_rows = repository::stamp_modified(connection, tenant_id, id, actor).await?;
    if stamped_rows == 0 {
        return Err(sea_orm::DbErr::RecordNotFound(
            CONVERSATION_NOT_FOUND_MESSAGE.into(),
        ));
    }
    Ok(())
}
