use crate::ai_summary::dto;
use crate::ai_summary::service::{self, GenerateOutcome, SummaryError};
use crate::common::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::Router;
use uuid::Uuid;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/dashboard/{dashboard_id}", get(get_dashboard_summaries))
        .route(
            "/dashboard/{dashboard_id}",
            post(generate_dashboard_summary),
        )
        .route(
            "/dashboard/{dashboard_id}/charts",
            post(generate_all_chart_summaries),
        )
        .route("/chart/{cell_id}", post(generate_chart_summary))
        .route("/dashboard/{dashboard_id}/prompt", put(set_dashboard_prompt))
        .route("/chart/{cell_id}/prompt", put(set_chart_prompt))
        .with_state(app_state)
}

async fn set_dashboard_prompt(
    State(app_state): State<AppState>,
    axum::extract::Path(dashboard_id): axum::extract::Path<Uuid>,
    axum::extract::Json(payload): axum::extract::Json<dto::UpdateUserPromptDto>,
) -> impl IntoResponse {
    match service::set_dashboard_prompt(&app_state, dashboard_id, payload.user_prompt).await {
        Ok(summary) => (StatusCode::OK, axum::Json(serde_json::json!(summary))),
        Err(e) => summary_error_response(e),
    }
}

async fn set_chart_prompt(
    State(app_state): State<AppState>,
    axum::extract::Path(cell_id): axum::extract::Path<Uuid>,
    axum::extract::Json(payload): axum::extract::Json<dto::UpdateUserPromptDto>,
) -> impl IntoResponse {
    match service::set_chart_prompt(&app_state, cell_id, payload.user_prompt).await {
        Ok(summary) => (StatusCode::OK, axum::Json(serde_json::json!(summary))),
        Err(e) => summary_error_response(e),
    }
}

async fn generate_all_chart_summaries(
    State(app_state): State<AppState>,
    axum::extract::Path(dashboard_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match service::generate_all_chart_summaries(&app_state, dashboard_id).await {
        Ok(summaries) => (
            StatusCode::ACCEPTED,
            axum::Json(serde_json::json!(summaries)),
        ),
        Err(e) => summary_error_response(e),
    }
}

async fn get_dashboard_summaries(
    State(app_state): State<AppState>,
    axum::extract::Path(dashboard_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match service::get_dashboard_summaries(&app_state, dashboard_id).await {
        Ok(summaries) => (StatusCode::OK, axum::Json(serde_json::json!(summaries))),
        Err(e) => summary_error_response(e),
    }
}

async fn generate_dashboard_summary(
    State(app_state): State<AppState>,
    axum::extract::Path(dashboard_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match service::generate_dashboard_summary(&app_state, dashboard_id).await {
        Ok(outcome) => generate_outcome_response(outcome),
        Err(e) => summary_error_response(e),
    }
}

async fn generate_chart_summary(
    State(app_state): State<AppState>,
    axum::extract::Path(cell_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match service::generate_chart_summary(&app_state, cell_id).await {
        Ok(outcome) => generate_outcome_response(outcome),
        Err(e) => summary_error_response(e),
    }
}

fn generate_outcome_response(
    outcome: GenerateOutcome,
) -> (StatusCode, axum::Json<serde_json::Value>) {
    match outcome {
        GenerateOutcome::Generated(summary) => {
            (StatusCode::OK, axum::Json(serde_json::json!(summary)))
        }
        GenerateOutcome::AlreadyGenerating(summary) => {
            (StatusCode::CONFLICT, axum::Json(serde_json::json!(summary)))
        }
    }
}

fn summary_error_response(err: SummaryError) -> (StatusCode, axum::Json<serde_json::Value>) {
    match err {
        SummaryError::NotFound => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Not found" })),
        ),
        SummaryError::NoCharts => (
            StatusCode::PRECONDITION_FAILED,
            axum::Json(serde_json::json!({ "error": "Dashboard has no charts to summarize" })),
        ),
        SummaryError::AiNotConfigured => (
            StatusCode::PRECONDITION_FAILED,
            axum::Json(serde_json::json!({ "error": "AI is not configured" })),
        ),
        SummaryError::PromptTooLong(max) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({
                "error": format!("Prompt must be at most {} characters", max)
            })),
        ),
        SummaryError::Repository(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
