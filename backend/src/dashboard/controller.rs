use crate::common::AppState;
use crate::dashboard::dto;
use crate::dashboard::service;
use axum::extract::State;
use axum::{
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use uuid::Uuid;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/{id}", get(get_dashboard))
        .route("/canvas/{canvas_id}", get(get_dashboard_by_canvas_id))
        .route("/", post(create_dashboard))
        .route("/{id}", put(update_dashboard))
        .with_state(app_state)
}

async fn get_dashboard(
    State(app_state): State<AppState>,
    axum::extract::Path(dashboard_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    service::get_dashboard(app_state, dashboard_id).await
}

async fn get_dashboard_by_canvas_id(
    State(app_state): State<AppState>,
    axum::extract::Path(canvas_id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    service::get_dashboard_by_canvas_id(app_state, canvas_id).await
}

async fn create_dashboard(
    State(app_state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<dto::CreateDashboardDto>,
) -> impl IntoResponse {
    service::create_dashboard(app_state, payload).await
}

async fn update_dashboard(
    State(app_state): State<AppState>,
    axum::extract::Path(dashboard_id): axum::extract::Path<Uuid>,
    axum::extract::Json(payload): axum::extract::Json<dto::DashboardDto>,
) -> impl IntoResponse {
    service::update_dashboard(app_state, dashboard_id, payload).await
}
