use crate::common::error_codes::codes;
use crate::common::errors::{ExecuteQueryError, ExecuteSqlError, GetDatabaseConnectionError};
use crate::common::AppState;
use crate::connection::dto;
use crate::connection::service;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::Router;
use uuid::Uuid;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/{id}", get(get_connection))
        .route("/", get(get_all_connections))
        .route("/", post(create_connection))
        .route("/{id}", put(update_connection))
        .route("/{id}/execute-query", post(execute_query))
        .with_state(app_state)
}

async fn get_connection(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    service::get_connection(app_state, id).await
}

async fn create_connection(
    State(app_state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<dto::CreateConnectionDto>,
) -> impl IntoResponse {
    service::create_connection(&app_state, payload).await
}

async fn get_all_connections(State(app_state): State<AppState>) -> impl IntoResponse {
    service::get_all_connections(app_state).await
}

async fn update_connection(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    axum::extract::Json(payload): axum::extract::Json<dto::ConnectionDto>,
) -> impl IntoResponse {
    service::update_connection(app_state, id, payload).await
}

async fn execute_query(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    axum::extract::Json(payload): axum::extract::Json<dto::ExecuteQueryDto>,
) -> impl IntoResponse {
    match service::execute_query(&app_state, id, payload).await {
        Ok(result) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "columns": result.columns,
                "data": result.data,
            })),
        ),
        Err(ExecuteSqlError::GetConnection(GetDatabaseConnectionError::ConnectionNotFound)) => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Connection not found" })),
        ),
        Err(ExecuteSqlError::GetConnection(e)) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
        Err(ExecuteSqlError::ExecuteQuery(ExecuteQueryError::Database(sqlx::Error::Database(
            e,
        )))) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({
                "status": StatusCode::BAD_REQUEST.as_u16(),
                "code": codes::SYNTAX_ERROR.as_str(),
                "message": e.to_string()
            })),
        ),
        Err(ExecuteSqlError::ExecuteQuery(ExecuteQueryError::Database(e))) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
        Err(ExecuteSqlError::ExecuteQuery(ExecuteQueryError::UnhandledDataType(type_name))) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "error": format!("data type '{}' not handled", type_name)
            })),
        ),
    }
}
