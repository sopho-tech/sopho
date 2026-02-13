use crate::cell::dto;
use crate::cell::service;
use crate::common::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use uuid::Uuid;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", post(create_cell))
        .route("/{id}", get(get_cell))
        .route("/{id}", put(update_cell))
        .route("/{id}", delete(delete_cell))
        .route("/{id}/execute", post(execute_cell))
        .route("/{id}/execute-preview", post(execute_cell_preview))
        .route("/{id}/reorder", patch(reorder_cell))
        .with_state(app_state)
}

async fn get_cell(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    service::get_cell(app_state, id).await
}

async fn create_cell(
    State(app_state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<dto::CreateCellDto>,
) -> impl IntoResponse {
    service::create_cell(app_state, payload).await
}

async fn update_cell(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    axum::extract::Json(payload): axum::extract::Json<dto::CellDto>,
) -> impl IntoResponse {
    service::update_cell(app_state, id, payload).await
}

async fn execute_cell(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    service::execute_cell(app_state, id).await
}

async fn execute_cell_preview(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    axum::extract::Json(payload): axum::extract::Json<dto::ExecuteCellPreviewDto>,
) -> impl IntoResponse {
    service::execute_cell_preview(app_state, id, payload).await
}

async fn delete_cell(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    service::delete_cell(app_state, id).await
}

async fn reorder_cell(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    axum::extract::Json(payload): axum::extract::Json<dto::ReorderCellDto>,
) -> impl IntoResponse {
    service::reorder_cell(app_state, id, payload).await
}
