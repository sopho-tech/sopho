use crate::ai_configuration::dto::{
    TestAiConfigurationRequest, TestAiConfigurationResponse, UpsertAiConfigurationRequest,
};
use crate::ai_configuration::error::AiConfigurationError;
use crate::ai_configuration::service;
use crate::common::AppState;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;
use serde_json::json;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(get_status))
        .route("/", put(upsert))
        .route("/", delete(delete_handler))
        .route("/test", post(test_handler))
        .with_state(app_state)
}

async fn get_status(State(app_state): State<AppState>) -> impl IntoResponse {
    match service::get_status(&app_state).await {
        Ok(dto) => (StatusCode::OK, Json(json!(dto))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn upsert(
    State(app_state): State<AppState>,
    Json(payload): Json<UpsertAiConfigurationRequest>,
) -> impl IntoResponse {
    match service::upsert(&app_state, payload.provider, payload.api_key).await {
        Ok(dto) => (StatusCode::OK, Json(json!(dto))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn delete_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    match service::delete(&app_state).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => error_response(e),
    }
}

async fn test_handler(
    State(_app_state): State<AppState>,
    Json(payload): Json<TestAiConfigurationRequest>,
) -> impl IntoResponse {
    match service::test(payload.provider, payload.api_key).await {
        Ok(resp) => (StatusCode::OK, Json(json!(resp))).into_response(),
        Err(AiConfigurationError::EmptyApiKey) => (
            StatusCode::BAD_REQUEST,
            Json(json!(TestAiConfigurationResponse {
                ok: false,
                error: Some("api_key must not be empty".to_string()),
            })),
        )
            .into_response(),
        Err(e) => error_response(e),
    }
}

fn error_response(e: AiConfigurationError) -> axum::response::Response {
    let status = match &e {
        AiConfigurationError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AiConfigurationError::Encryption(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AiConfigurationError::ClientBuild(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AiConfigurationError::EmptyApiKey => StatusCode::BAD_REQUEST,
        AiConfigurationError::NotFound => StatusCode::NOT_FOUND,
    };
    (status, Json(json!({ "error": e.to_string() }))).into_response()
}
