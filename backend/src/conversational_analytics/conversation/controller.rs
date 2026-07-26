use super::dto;
use super::error::AppendUserMessageError;
use super::error::ConversationError;
use super::error::ExecuteCompletionError;
use super::service;
use crate::common::error_codes::codes;
use crate::common::AppState;
use axum::extract::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;
use serde_json::Value;
use uuid::Uuid;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(list_conversations))
        .route("/", post(create_conversation))
        .route("/bulk-delete", post(bulk_delete_conversations))
        .route("/{conversation_id}", get(get_conversation))
        .route("/{conversation_id}", put(update_conversation))
        .route("/{conversation_id}", delete(delete_conversation))
        .route("/{conversation_id}/completion", post(execute_completion))
        .route("/{conversation_id}/suggest-name", post(suggest_name))
        .route("/{conversation_id}/messages", post(append_user_message))
        .with_state(app_state)
}

async fn list_conversations(
    State(app_state): State<AppState>,
    Query(query): Query<dto::ListConversationsQuery>,
) -> impl IntoResponse {
    match service::list_conversations(app_state, query).await {
        Ok(page) => (StatusCode::OK, axum::Json(serde_json::json!(page))),
        Err(e) => conversation_error_http_response(e),
    }
}

async fn bulk_delete_conversations(
    State(app_state): State<AppState>,
    Json(payload): Json<dto::BulkDeleteConversationsDto>,
) -> impl IntoResponse {
    match service::bulk_delete_conversations(app_state, payload).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => conversation_error_http_response(e).into_response(),
    }
}

async fn get_conversation(
    State(app_state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
) -> impl IntoResponse {
    match service::get_conversation(app_state, conversation_id).await {
        Ok(response_dto) => (StatusCode::OK, axum::Json(serde_json::json!(response_dto))),
        Err(e) => conversation_error_http_response(e),
    }
}

async fn create_conversation(
    State(app_state): State<AppState>,
    Json(payload): Json<dto::CreateConversationDto>,
) -> impl IntoResponse {
    match service::create_conversation(app_state, payload).await {
        Ok(response_dto) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!(response_dto)),
        ),
        Err(e) => conversation_error_http_response(e),
    }
}

async fn update_conversation(
    State(app_state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<dto::ConversationDto>,
) -> impl IntoResponse {
    match service::update_conversation(app_state, conversation_id, payload).await {
        Ok(response_dto) => (StatusCode::OK, axum::Json(serde_json::json!(response_dto))),
        Err(e) => conversation_error_http_response(e),
    }
}

async fn delete_conversation(
    State(app_state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
) -> impl IntoResponse {
    match service::delete_conversation(app_state, conversation_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => conversation_error_http_response(e).into_response(),
    }
}

async fn execute_completion(
    State(app_state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
) -> impl IntoResponse {
    match service::execute_completion(app_state, conversation_id).await {
        Ok(sse) => sse.into_response(),
        Err(e) => {
            let (status, error, code) = match &e {
                ExecuteCompletionError::Database(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string(), None)
                }
                ExecuteCompletionError::ConnectionNotFound => {
                    (StatusCode::NOT_FOUND, e.to_string(), None)
                }
                ExecuteCompletionError::ConversationNotFound => {
                    (StatusCode::NOT_FOUND, e.to_string(), None)
                }
                ExecuteCompletionError::NoQuestionsToAnswer
                | ExecuteCompletionError::LastSenderNotHuman
                | ExecuteCompletionError::InvalidLastMessageContent
                | ExecuteCompletionError::EmptyQuestion
                | ExecuteCompletionError::QuestionTooLong => {
                    (StatusCode::BAD_REQUEST, e.to_string(), None)
                }
                ExecuteCompletionError::AiNotLive => {
                    (StatusCode::FORBIDDEN, e.to_string(), Some(codes::AI_NOT_LIVE.as_str()))
                }
            };
            let body = match code {
                Some(c) => serde_json::json!({ "error": error, "code": c }),
                None => serde_json::json!({ "error": error }),
            };
            (status, axum::Json(body)).into_response()
        }
    }
}

async fn suggest_name(
    State(app_state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
) -> impl IntoResponse {
    match service::suggest_conversation_name(app_state, conversation_id).await {
        Ok(response_dto) => (StatusCode::OK, axum::Json(serde_json::json!(response_dto))),
        Err(e) => conversation_error_http_response(e),
    }
}

async fn append_user_message(
    State(app_state): State<AppState>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<dto::AppendUserMessageDto>,
) -> impl IntoResponse {
    match service::append_user_message(app_state, conversation_id, payload).await {
        Ok(response_dto) => (StatusCode::OK, axum::Json(serde_json::json!(response_dto))).into_response(),
        Err(e) => {
            let (status, error, code) = match &e {
                AppendUserMessageError::Database(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string(), None)
                }
                AppendUserMessageError::NotFound => (StatusCode::NOT_FOUND, e.to_string(), None),
                AppendUserMessageError::ConversationBusy => (
                    StatusCode::CONFLICT,
                    e.to_string(),
                    Some(codes::CONVERSATION_BUSY.as_str()),
                ),
                AppendUserMessageError::EmptyQuestion
                | AppendUserMessageError::QuestionTooLong => {
                    (StatusCode::BAD_REQUEST, e.to_string(), None)
                }
                AppendUserMessageError::AiNotLive => {
                    (StatusCode::FORBIDDEN, e.to_string(), Some(codes::AI_NOT_LIVE.as_str()))
                }
            };
            let body = match code {
                Some(c) => serde_json::json!({ "error": error, "code": c }),
                None => serde_json::json!({ "error": error }),
            };
            (status, axum::Json(body)).into_response()
        }
    }
}

fn conversation_error_http_response(e: ConversationError) -> (StatusCode, axum::Json<Value>) {
    let (status, error, code) = match &e {
        ConversationError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string(), None),
        ConversationError::Conversion(_) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string(), None),
        ConversationError::NotFound => (StatusCode::NOT_FOUND, e.to_string(), None),
        ConversationError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, e.to_string(), None),
        ConversationError::AiNotLive => (
            StatusCode::FORBIDDEN,
            e.to_string(),
            Some(codes::AI_NOT_LIVE.as_str()),
        ),
    };
    let body = match code {
        Some(c) => serde_json::json!({ "error": error, "code": c }),
        None => serde_json::json!({ "error": error }),
    };
    (status, axum::Json(body))
}
