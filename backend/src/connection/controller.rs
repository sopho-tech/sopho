use axum::Router;
use axum::extract::State;   
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use crate::connection::dto;
use crate::common::AppState;
use crate::connection::service;
use uuid::Uuid;


pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/{id}", get(get_connection))
        .route("/", get(get_all_connections))
        .route("/", post(create_connection))
        .route("/{id}", put(update_connection))
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
    service::create_connection(app_state, payload).await
}

async fn get_all_connections(
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    service::get_all_connections(app_state).await
}

async fn update_connection(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    axum::extract::Json(payload): axum::extract::Json<dto::ConnectionDto>,
) -> impl IntoResponse {
    service::update_connection(app_state, id, payload).await
}
