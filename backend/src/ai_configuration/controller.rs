use crate::ai_configuration::dto::{
    TestAiConfigurationRequest, TestAiConfigurationResponse, UpsertAiConfigurationRequest,
};
use crate::ai_configuration::error::AiConfigurationError;
use crate::ai_configuration::service;
use crate::authorization::dto::TenantContext;
use crate::common::AppState;
use axum::Router;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use serde_json::json;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(get_status))
        .route("/", put(upsert))
        .route("/", delete(delete_handler))
        .route("/test", post(test_handler))
        .with_state(app_state)
}

async fn get_status(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
) -> impl IntoResponse {
    match service::get_status(&app_state, &tenant_context).await {
        Ok(dto) => (StatusCode::OK, Json(json!(dto))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn upsert(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Json(payload): Json<UpsertAiConfigurationRequest>,
) -> impl IntoResponse {
    match service::upsert(
        &app_state,
        &tenant_context,
        payload.provider,
        payload.api_key,
    )
    .await
    {
        Ok(dto) => (StatusCode::OK, Json(json!(dto))).into_response(),
        Err(e) => error_response(e),
    }
}

async fn delete_handler(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
) -> impl IntoResponse {
    match service::delete(&app_state, &tenant_context).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => error_response(e),
    }
}

async fn test_handler(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Json(payload): Json<TestAiConfigurationRequest>,
) -> impl IntoResponse {
    match service::verify_provider_api_key(
        &app_state,
        &tenant_context,
        payload.provider,
        payload.api_key,
    )
    .await
    {
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
    if let AiConfigurationError::Unauthorized(error) = e {
        let (status, body) = crate::authorization::service::authorization_error_response(
            error,
            "AI configuration not found",
        );
        return (status, body).into_response();
    }
    let status = match &e {
        AiConfigurationError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AiConfigurationError::Encryption(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AiConfigurationError::ClientBuild(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AiConfigurationError::EmptyApiKey => StatusCode::BAD_REQUEST,
        AiConfigurationError::Unauthorized(_) => unreachable!("handled above"),
    };
    (status, Json(json!({ "error": e.to_string() }))).into_response()
}
