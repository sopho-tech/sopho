use crate::authorization::dto::TenantContext;
use crate::common::AppState;
use crate::tenant::membership::constants::MEMBER_NOT_FOUND_MESSAGE;
use crate::tenant::membership::dto::UpdateMemberRoleDto;
use crate::tenant::membership::error::MembershipError;
use crate::tenant::membership::service;
use axum::Router;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, put};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
struct MemberPath {
    user_id: Uuid,
}

pub fn member_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(list_members))
        .route("/{user_id}", put(update_member_role).delete(remove_member))
        .with_state(app_state)
}

pub fn role_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(list_roles))
        .with_state(app_state)
}

async fn list_members(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
) -> impl IntoResponse {
    match service::list_members(&app_state, &tenant_context).await {
        Ok(members) => (StatusCode::OK, axum::Json(serde_json::json!(members))),
        Err(error) => membership_error_http_response(error),
    }
}

async fn update_member_role(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Path(MemberPath { user_id }): Path<MemberPath>,
    Json(payload): Json<UpdateMemberRoleDto>,
) -> impl IntoResponse {
    match service::update_member_role(&app_state, &tenant_context, user_id, &payload.role_name)
        .await
    {
        Ok(member) => (StatusCode::OK, axum::Json(serde_json::json!(member))),
        Err(error) => membership_error_http_response(error),
    }
}

async fn remove_member(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Path(MemberPath { user_id }): Path<MemberPath>,
) -> impl IntoResponse {
    match service::remove_member(&app_state, &tenant_context, user_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => membership_error_http_response(error).into_response(),
    }
}

async fn list_roles(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
) -> impl IntoResponse {
    match service::list_roles(&app_state, &tenant_context).await {
        Ok(roles) => (StatusCode::OK, axum::Json(serde_json::json!(roles))),
        Err(error) => membership_error_http_response(error),
    }
}

fn membership_error_http_response(error: MembershipError) -> (StatusCode, axum::Json<Value>) {
    let status = match error {
        MembershipError::Unauthorized(authorization_error) => {
            return crate::authorization::service::authorization_error_response(
                authorization_error,
                MEMBER_NOT_FOUND_MESSAGE,
            );
        }
        MembershipError::MemberNotFound => StatusCode::NOT_FOUND,
        MembershipError::UnknownRole => StatusCode::BAD_REQUEST,
        MembershipError::RoleTooPowerful | MembershipError::MemberTooPowerful => {
            StatusCode::FORBIDDEN
        }
        MembershipError::LastOwner => StatusCode::CONFLICT,
        MembershipError::Database(_) | MembershipError::Internal(_) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    (
        status,
        axum::Json(serde_json::json!({ "error": error.to_string() })),
    )
}
