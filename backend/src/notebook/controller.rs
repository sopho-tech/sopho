use crate::common::AppState;
use crate::notebook::service;
use crate::notebook::dto;
use axum::routing::{get, post};
use axum::Router;
use axum::extract::{State, Json};
use axum::response::IntoResponse;
use uuid::Uuid;
use axum::extract::Path;


pub fn routes(app_state: AppState) -> Router {
    Router::new()
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
) -> impl IntoResponse {
    service::get_all_notebooks(app_state).await
}

async fn create_notebook(
    State(app_state): State<AppState>,
    Json(payload): Json<dto::CreateNotebookDto>,
) -> impl IntoResponse {
    service::create_notebook(app_state, payload).await
}
