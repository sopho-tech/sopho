use super::dto;
use super::error::ConversationError;
use super::error::ExecuteCompletionError;
use super::service;
use crate::common::AppState;
use axum::extract::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;
use serde_json::Value;
use uuid::Uuid;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(get_all_conversations))
        .route("/", post(create_conversation))
        .route("/{conversation_id}", get(get_conversation))
        .route("/{conversation_id}", put(update_conversation))
        .route("/{conversation_id}", delete(delete_conversation))
        .route("/{conversation_id}/completion", post(execute_completion))
        .route("/{conversation_id}/suggest-name", post(suggest_name))
        .with_state(app_state)
}

async fn get_all_conversations(State(app_state): State<AppState>) -> impl IntoResponse {
    match service::get_all_conversations(app_state).await {
        Ok(list) => (StatusCode::OK, axum::Json(serde_json::json!(list))),
        Err(e) => conversation_error_http_response(e),
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
            let (status, error) = match &e {
                ExecuteCompletionError::Database(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                }
                ExecuteCompletionError::ConnectionNotFound => {
                    (StatusCode::NOT_FOUND, e.to_string())
                }
                ExecuteCompletionError::ConversationNotFound => {
                    (StatusCode::NOT_FOUND, e.to_string())
                }
                ExecuteCompletionError::NoQuestionsToAnswer
                | ExecuteCompletionError::LastSenderNotHuman
                | ExecuteCompletionError::InvalidLastMessageContent
                | ExecuteCompletionError::EmptyQuestion
                | ExecuteCompletionError::QuestionTooLong => {
                    (StatusCode::BAD_REQUEST, e.to_string())
                }
                ExecuteCompletionError::AnthropicNotConfigured => {
                    (StatusCode::SERVICE_UNAVAILABLE, e.to_string())
                }
            };
            (status, axum::Json(serde_json::json!({ "error": error }))).into_response()
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

fn conversation_error_http_response(e: ConversationError) -> (StatusCode, axum::Json<Value>) {
    let (status, error) = match &e {
        ConversationError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        ConversationError::Conversion(_) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()),
        ConversationError::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
    };
    (status, axum::Json(serde_json::json!({ "error": error })))
}
