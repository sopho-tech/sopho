use crate::canvas::dto;
use crate::canvas::service;
use crate::common::AppState;
use crate::common::Pagination;
use axum::extract::Path;
use axum::extract::{Json, Query, State};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;
use uuid::Uuid;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/{id}", get(get_canvas))
        .route("/{id}", put(update_canvas))
        .route("/{id}", delete(delete_canvas))
        .route("/", get(get_all_canvases))
        .route("/", post(create_canvas))
        .with_state(app_state)
}

async fn get_canvas(State(app_state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    service::get_canvas(app_state, id).await
}

async fn get_all_canvases(
    State(app_state): State<AppState>,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    service::get_all_canvases(app_state, pagination).await
}

async fn update_canvas(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<dto::CreateCanvasDto>,
) -> impl IntoResponse {
    service::update_canvas(app_state, id, payload).await
}

async fn create_canvas(
    State(app_state): State<AppState>,
    Json(payload): Json<dto::CreateCanvasDto>,
) -> impl IntoResponse {
    service::create_canvas(app_state, payload).await
}

async fn delete_canvas(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    service::delete_canvas(app_state, id).await
}
