use crate::authorization::dto::{AuthenticatedUser, TenantContext};
use crate::common::AppState;
use crate::tenant::dto::CreateTenantDto;
use crate::tenant::service;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(list_tenants))
        .route("/", post(create_tenant))
        .with_state(app_state)
}

async fn list_tenants(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> impl IntoResponse {
    match service::list_tenants_for_user(&app_state, authenticated_user.user_id()).await {
        Ok(tenants) => (StatusCode::OK, axum::Json(serde_json::json!(tenants))),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}

async fn create_tenant(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Json(payload): Json<CreateTenantDto>,
) -> impl IntoResponse {
    match service::create_tenant_for_user(&app_state, authenticated_user.user_id(), payload.name)
        .await
    {
        Ok(created_tenant) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({
                "id": created_tenant.id,
                "name": created_tenant.name,
            })),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}

pub fn member_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(list_members))
        .with_state(app_state)
}

async fn list_members(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
) -> impl IntoResponse {
    match service::list_members(&app_state, &tenant_context).await {
        Ok(members) => (StatusCode::OK, axum::Json(serde_json::json!(members))),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}
