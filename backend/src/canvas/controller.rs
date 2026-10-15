use crate::authorization::dto::TenantContext;
use crate::canvas::constants::{CANVAS_DELETED_MESSAGE, CANVAS_NOT_FOUND_MESSAGE};
use crate::canvas::dto;
use crate::canvas::error::CanvasError;
use crate::canvas::service;
use crate::common::constants::UNKNOWN_CREATED_BY_FILTER_MESSAGE;
use crate::common::AppState;
use crate::common::{CreatedByQuery, Pagination};
use axum::extract::Path;
use axum::extract::{Json, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;
use serde_json::Value;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct CanvasPath {
    id: Uuid,
}

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/{id}", get(get_canvas))
        .route("/{id}", put(update_canvas))
        .route("/{id}", delete(delete_canvas))
        .route("/", get(get_all_canvases))
        .route("/", post(create_canvas))
        .with_state(app_state)
}

async fn get_canvas(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Path(CanvasPath { id }): Path<CanvasPath>,
) -> impl IntoResponse {
    match service::get_canvas(app_state, &tenant_context, id).await {
        Ok(canvas) => (StatusCode::OK, axum::Json(serde_json::json!(canvas))),
        Err(error) => canvas_error_http_response(error),
    }
}

async fn get_all_canvases(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    pagination: Query<Pagination>,
    created_by_query: Query<CreatedByQuery>,
) -> impl IntoResponse {
    let Ok(created_by) = created_by_query.filter() else {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": UNKNOWN_CREATED_BY_FILTER_MESSAGE })),
        );
    };
    match service::get_all_canvases(
        app_state,
        &tenant_context,
        pagination.page(),
        pagination.page_size(),
        created_by,
    )
    .await
    {
        Ok(page) => (StatusCode::OK, axum::Json(serde_json::json!(page))),
        Err(error) => canvas_error_http_response(error),
    }
}

async fn update_canvas(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Path(CanvasPath { id }): Path<CanvasPath>,
    Json(payload): Json<dto::CreateCanvasDto>,
) -> impl IntoResponse {
    match service::update_canvas(app_state, &tenant_context, id, payload).await {
        Ok(canvas) => (StatusCode::OK, axum::Json(serde_json::json!(canvas))),
        Err(error) => canvas_error_http_response(error),
    }
}

async fn create_canvas(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Json(payload): Json<dto::CreateCanvasDto>,
) -> impl IntoResponse {
    let owner_user_id = tenant_context.user_id().get_uuid();
    match service::create_canvas(app_state, &tenant_context, owner_user_id, payload).await {
        Ok(canvas) => (StatusCode::CREATED, axum::Json(serde_json::json!(canvas))),
        Err(error) => canvas_error_http_response(error),
    }
}

async fn delete_canvas(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Path(CanvasPath { id }): Path<CanvasPath>,
) -> impl IntoResponse {
    match service::delete_canvas(app_state, &tenant_context, id).await {
        Ok(()) => (
            StatusCode::NO_CONTENT,
            axum::Json(serde_json::json!({ "message": CANVAS_DELETED_MESSAGE })),
        ),
        Err(error) => canvas_error_http_response(error),
    }
}

fn canvas_error_http_response(error: CanvasError) -> (StatusCode, axum::Json<Value>) {
    match error {
        CanvasError::Unauthorized(authorization_error) => {
            crate::authorization::service::authorization_error_response(
                authorization_error,
                CANVAS_NOT_FOUND_MESSAGE,
            )
        }
        CanvasError::NotFound => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        CanvasError::Database(_) | CanvasError::Internal(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}
