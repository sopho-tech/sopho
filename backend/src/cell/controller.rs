use axum::Router;
use axum::extract::State;   
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use crate::cell::dto;
use uuid::Uuid;
use crate::common::AppState;
use crate::cell::service;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", post(create_cell))
        .route("/{id}", get(get_cell))
        .route("/{id}", put(update_cell))
        .route("/{id}/execute", post(execute_cell))
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
