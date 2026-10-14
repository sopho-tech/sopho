use crate::authorization::dto::{AuthenticatedUser, TenantContext};
use crate::common::AppState;
use crate::tenant::invitation::constants::INVITATION_NOT_FOUND_MESSAGE;
use crate::tenant::invitation::dto::CreateInvitationDto;
use crate::tenant::invitation::error::InvitationError;
use crate::tenant::invitation::service;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::Router;
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
struct InvitationPath {
    invitation_id: Uuid,
}

pub fn tenant_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(list_tenant_invitations).post(create_invitation))
        .route("/{invitation_id}", delete(revoke_invitation))
        .with_state(app_state)
}

pub fn user_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(list_received_invitations))
        .route("/{invitation_id}/accept", post(accept_invitation))
        .with_state(app_state)
}

async fn create_invitation(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Json(payload): Json<CreateInvitationDto>,
) -> impl IntoResponse {
    match service::create_invitation(&app_state, &tenant_context, payload).await {
        Ok(invitation) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!(invitation)),
        ),
        Err(error) => invitation_error_http_response(error),
    }
}

async fn list_tenant_invitations(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
) -> impl IntoResponse {
    match service::list_tenant_invitations(&app_state, &tenant_context).await {
        Ok(invitations) => (StatusCode::OK, axum::Json(serde_json::json!(invitations))),
        Err(error) => invitation_error_http_response(error),
    }
}

async fn revoke_invitation(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Path(InvitationPath { invitation_id }): Path<InvitationPath>,
) -> impl IntoResponse {
    match service::revoke_invitation(&app_state, &tenant_context, invitation_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => invitation_error_http_response(error).into_response(),
    }
}

async fn list_received_invitations(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> impl IntoResponse {
    match service::list_received_invitations(&app_state, authenticated_user.user_id()).await {
        Ok(invitations) => (StatusCode::OK, axum::Json(serde_json::json!(invitations))),
        Err(error) => invitation_error_http_response(error),
    }
}

async fn accept_invitation(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(InvitationPath { invitation_id }): Path<InvitationPath>,
) -> impl IntoResponse {
    match service::accept_invitation(&app_state, authenticated_user.user_id(), invitation_id).await
    {
        Ok(accepted) => (StatusCode::OK, axum::Json(serde_json::json!(accepted))),
        Err(error) => invitation_error_http_response(error),
    }
}

fn invitation_error_http_response(error: InvitationError) -> (StatusCode, axum::Json<Value>) {
    let status = match error {
        InvitationError::Unauthorized(authorization_error) => {
            return crate::authorization::service::authorization_error_response(
                authorization_error,
                INVITATION_NOT_FOUND_MESSAGE,
            );
        }
        InvitationError::InvalidEmail | InvitationError::UnknownRole => StatusCode::BAD_REQUEST,
        InvitationError::RoleTooPowerful => StatusCode::FORBIDDEN,
        InvitationError::AlreadyMember
        | InvitationError::AlreadyInvited
        | InvitationError::Expired
        | InvitationError::SingleSignOnRequired => StatusCode::CONFLICT,
        InvitationError::NotFound => StatusCode::NOT_FOUND,
        InvitationError::Database(_) | InvitationError::Internal(_) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    (
        status,
        axum::Json(serde_json::json!({ "error": error.to_string() })),
    )
}
