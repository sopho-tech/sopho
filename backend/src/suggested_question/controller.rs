use crate::common::AppState;
use crate::suggested_question::service;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use uuid::Uuid;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/connection/{connection_id}", get(get_for_connection))
        .with_state(app_state)
}

async fn get_for_connection(
    State(app_state): State<AppState>,
    axum::extract::Path(connection_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    match service::list_for_connection(&app_state, connection_id).await {
        Ok(list) => (StatusCode::OK, axum::Json(serde_json::json!(list))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
