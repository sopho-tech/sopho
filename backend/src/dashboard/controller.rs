use axum::{
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use crate::dashboard::dto;
use uuid::Uuid;
use axum::extract::State;
use crate::common::AppState;
use crate::dashboard::service;


pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/{id}", get(get_dashboard))
        .route("/", post(create_dashboard))
        .with_state(app_state)
}

async fn get_dashboard(
    State(app_state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    service::get_dashboard(app_state, id).await
}

async fn create_dashboard(
    State(app_state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<dto::CreateDashboardDto>,
) -> impl IntoResponse {
    service::create_dashboard(app_state, payload).await
}
