use crate::authorization::dto::TenantContext;
use crate::common::AppState;
use crate::dashboard::constants::{CANVAS_NOT_FOUND_MESSAGE, DASHBOARD_NOT_FOUND_MESSAGE};
use crate::dashboard::dto;
use crate::dashboard::error::DashboardError;
use crate::dashboard::service;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{
    Router,
    response::IntoResponse,
    routing::{get, post, put},
};
use serde_json::Value;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct DashboardPath {
    id: Uuid,
}

#[derive(serde::Deserialize)]
struct DashboardCanvasPath {
    canvas_id: Uuid,
}

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
    tenant_context: TenantContext,
    axum::extract::Path(DashboardPath { id: dashboard_id }): axum::extract::Path<DashboardPath>,
) -> impl IntoResponse {
    match service::get_dashboard(app_state, &tenant_context, dashboard_id).await {
        Ok(dashboard) => (StatusCode::OK, axum::Json(serde_json::json!(dashboard))),
        Err(error) => dashboard_error_http_response(error, DASHBOARD_NOT_FOUND_MESSAGE),
    }
}

async fn get_dashboard_by_canvas_id(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Path(DashboardCanvasPath { canvas_id }): axum::extract::Path<
        DashboardCanvasPath,
    >,
) -> impl IntoResponse {
    match service::get_dashboard_by_canvas_id(app_state, &tenant_context, canvas_id).await {
        Ok(dashboard) => (StatusCode::OK, axum::Json(serde_json::json!(dashboard))),
        Err(error) => dashboard_error_http_response(error, CANVAS_NOT_FOUND_MESSAGE),
    }
}

async fn create_dashboard(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Json(payload): axum::extract::Json<dto::CreateDashboardDto>,
) -> impl IntoResponse {
    match service::create_dashboard(app_state, &tenant_context, payload).await {
        Ok(dashboard) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!(dashboard)),
        ),
        Err(error) => dashboard_error_http_response(error, DASHBOARD_NOT_FOUND_MESSAGE),
    }
}

async fn update_dashboard(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Path(DashboardPath { id: dashboard_id }): axum::extract::Path<DashboardPath>,
    axum::extract::Json(payload): axum::extract::Json<dto::DashboardDto>,
) -> impl IntoResponse {
    match service::update_dashboard(app_state, &tenant_context, dashboard_id, payload).await {
        Ok(dashboard) => (StatusCode::OK, axum::Json(serde_json::json!(dashboard))),
        Err(error) => dashboard_error_http_response(error, DASHBOARD_NOT_FOUND_MESSAGE),
    }
}

fn dashboard_error_http_response(
    error: DashboardError,
    not_found_message: &str,
) -> (StatusCode, axum::Json<Value>) {
    match error {
        DashboardError::Unauthorized(authorization_error) => {
            crate::authorization::service::authorization_error_response(
                authorization_error,
                not_found_message,
            )
        }
        DashboardError::NotFound | DashboardError::CanvasNotFound => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        DashboardError::TooManyCharts(_) => (
            StatusCode::PRECONDITION_FAILED,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        DashboardError::LayoutOutsideCanvas => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        DashboardError::Database(_) | DashboardError::Internal(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}
