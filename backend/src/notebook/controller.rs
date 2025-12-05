use crate::common::AppState;
use crate::common::Pagination;
use crate::notebook::dto;
use crate::notebook::service;
use axum::extract::Path;
use axum::extract::{Json, Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use uuid::Uuid;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/canvas/{canvas_id}", get(get_notebooks_by_canvas_id))
        .route("/{id}", get(get_notebook))
        .route("/", get(get_all_notebooks))
        .route("/", post(create_notebook))
        .with_state(app_state)
}

async fn get_notebook(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    service::get_notebook(app_state, id).await
}

async fn get_all_notebooks(
    State(app_state): State<AppState>,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    service::get_all_notebooks(app_state, pagination).await
}

async fn create_notebook(
    State(app_state): State<AppState>,
    Json(payload): Json<dto::CreateNotebookDto>,
) -> impl IntoResponse {
    service::create_notebook(app_state, payload).await
}

async fn get_notebooks_by_canvas_id(
    State(app_state): State<AppState>,
    Path(canvas_id): Path<Uuid>,
) -> impl IntoResponse {
    service::get_notebooks_by_canvas_id(app_state, canvas_id).await
}
