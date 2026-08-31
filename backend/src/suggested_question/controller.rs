use crate::authorization::dto::TenantContext;
use crate::common::AppState;
use crate::suggested_question::errors::SuggestedQuestionError;
use crate::suggested_question::service;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct SuggestedQuestionConnectionPath {
    connection_id: Uuid,
}

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/connection/{connection_id}", get(get_for_connection))
        .with_state(app_state)
}

async fn get_for_connection(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Path(SuggestedQuestionConnectionPath { connection_id }): axum::extract::Path<
        SuggestedQuestionConnectionPath,
    >,
) -> impl IntoResponse {
    match service::list_for_connection(&app_state, &tenant_context, connection_id).await {
        Ok(list) => (StatusCode::OK, axum::Json(serde_json::json!(list))),
        Err(SuggestedQuestionError::Unauthorized(error)) => {
            crate::authorization::service::authorization_error_response(
                error,
                "Connection not found",
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
