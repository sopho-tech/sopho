use crate::authorization::dto::TenantContext;
use crate::common::AppState;
use crate::grant::dto::CreateGrantDto;
use crate::grant::error::GrantError;
use crate::grant::service;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::Router;
use serde_json::Value;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct GrantResourcePath {
    resource_type: String,
    resource_id: Uuid,
}

#[derive(serde::Deserialize)]
struct GrantPath {
    grant_id: Uuid,
}

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", post(create_grant))
        .route("/{resource_type}/{resource_id}", get(list_grants))
        .route("/{grant_id}", delete(revoke_grant))
        .with_state(app_state)
}

async fn create_grant(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Json(payload): Json<CreateGrantDto>,
) -> impl IntoResponse {
    match service::create_grant(app_state, &tenant_context, payload).await {
        Ok(grant) => (StatusCode::CREATED, axum::Json(serde_json::json!(grant))),
        Err(error) => grant_error_http_response(error),
    }
}

async fn list_grants(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Path(GrantResourcePath {
        resource_type,
        resource_id,
    }): Path<GrantResourcePath>,
) -> impl IntoResponse {
    match service::list_grants(app_state, &tenant_context, resource_type, resource_id).await {
        Ok(grants) => (StatusCode::OK, axum::Json(serde_json::json!(grants))),
        Err(error) => grant_error_http_response(error),
    }
}

async fn revoke_grant(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Path(GrantPath { grant_id }): Path<GrantPath>,
) -> impl IntoResponse {
    match service::revoke_grant(app_state, &tenant_context, grant_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => grant_error_http_response(error).into_response(),
    }
}

fn grant_error_http_response(error: GrantError) -> (StatusCode, axum::Json<Value>) {
    match error {
        GrantError::Unauthorized(authorization_error) => {
            crate::authorization::service::authorization_error_response(
                authorization_error,
                crate::grant::constants::RESOURCE_NOT_FOUND_MESSAGE,
            )
        }
        GrantError::InvalidRequest(_) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        GrantError::ResourceNotFound | GrantError::GrantNotFound => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        GrantError::Database(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}
