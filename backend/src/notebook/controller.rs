use crate::authorization::dto::TenantContext;
use crate::common::AppState;
use crate::common::Pagination;
use crate::notebook::constants::{
    QueryFilters, CANVAS_NOT_FOUND_MESSAGE, NOTEBOOK_NOT_FOUND_MESSAGE,
};
use crate::notebook::dto;
use crate::notebook::error::NotebookError;
use crate::notebook::service;
use axum::extract::Path;
use axum::extract::{Json, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use serde_json::Value;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct NotebookPath {
    id: Uuid,
}

#[derive(serde::Deserialize)]
struct NotebookCanvasPath {
    canvas_id: Uuid,
}

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/canvas/{canvas_id}", get(get_notebooks_by_canvas_id))
        .route("/{id}", get(get_notebook))
        .route("/{id}/cells", get(get_cells_by_notebook_id))
        .route("/", get(get_all_notebooks))
        .route("/", post(create_notebook))
        .with_state(app_state)
}

async fn get_notebook(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Path(NotebookPath { id }): Path<NotebookPath>,
) -> impl IntoResponse {
    match service::get_notebook(app_state, &tenant_context, id).await {
        Ok(notebook) => (StatusCode::OK, axum::Json(serde_json::json!(notebook))),
        Err(error) => notebook_error_http_response(error, NOTEBOOK_NOT_FOUND_MESSAGE),
    }
}

async fn get_all_notebooks(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    match service::get_all_notebooks(
        app_state,
        &tenant_context,
        pagination.page(),
        pagination.page_size(),
    )
    .await
    {
        Ok(page) => (StatusCode::OK, axum::Json(serde_json::json!(page))),
        Err(error) => notebook_error_http_response(error, NOTEBOOK_NOT_FOUND_MESSAGE),
    }
}

async fn get_cells_by_notebook_id(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Path(NotebookPath { id }): Path<NotebookPath>,
    filters: Query<QueryFilters>,
) -> impl IntoResponse {
    match service::get_cells_by_notebook_id(app_state, &tenant_context, id, filters.cell_type())
        .await
    {
        Ok(notebook) => (StatusCode::OK, axum::Json(serde_json::json!(notebook))),
        Err(error) => notebook_error_http_response(error, NOTEBOOK_NOT_FOUND_MESSAGE),
    }
}

async fn create_notebook(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Json(payload): Json<dto::CreateNotebookDto>,
) -> impl IntoResponse {
    match service::create_notebook(app_state, &tenant_context, payload).await {
        Ok(notebook) => (StatusCode::CREATED, axum::Json(serde_json::json!(notebook))),
        Err(error) => notebook_error_http_response(error, NOTEBOOK_NOT_FOUND_MESSAGE),
    }
}

async fn get_notebooks_by_canvas_id(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Path(NotebookCanvasPath { canvas_id }): Path<NotebookCanvasPath>,
) -> impl IntoResponse {
    match service::get_notebooks_by_canvas_id(app_state, &tenant_context, canvas_id).await {
        Ok(notebooks) => (StatusCode::OK, axum::Json(serde_json::json!(notebooks))),
        Err(error) => notebook_error_http_response(error, CANVAS_NOT_FOUND_MESSAGE),
    }
}

fn notebook_error_http_response(
    error: NotebookError,
    not_found_message: &str,
) -> (StatusCode, axum::Json<Value>) {
    match error {
        NotebookError::Unauthorized(authorization_error) => {
            crate::authorization::service::authorization_error_response(
                authorization_error,
                not_found_message,
            )
        }
        NotebookError::NotFound | NotebookError::CanvasNotFound => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        NotebookError::Database(_) | NotebookError::Internal(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}
