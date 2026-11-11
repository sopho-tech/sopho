use crate::authentication::service as authentication_service;
use crate::authorization::dto::{AuthenticatedUser, TenantContext};
use crate::common::AppState;
use crate::common::error_codes::ErrorCode;
use crate::tenant::invitation::constants::INVITATION_NOT_FOUND_MESSAGE;
use crate::tenant::invitation::dto::{
    CreateInvitationDto, InvitationLinkTokenDto, InvitationSignUpDto,
};
use crate::tenant::invitation::error::InvitationError;
use crate::tenant::invitation::service;
use axum::Router;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use serde::Deserialize;
use serde_json::Value;
use tower_cookies::Cookies;
use uuid::Uuid;

#[derive(Deserialize)]
struct InvitationPath {
    invitation_id: Uuid,
}

pub fn tenant_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(list_tenant_invitations).post(create_invitation))
        .route("/delivery", get(get_invitation_delivery))
        .route("/{invitation_id}", delete(revoke_invitation))
        .route("/{invitation_id}/resend", post(resend_invitation))
        .with_state(app_state)
}

pub fn link_routes(app_state: AppState) -> Router {
    Router::new()
        .route("/lookup", post(lookup_invitation_link))
        .route("/sign_up", post(sign_up_from_invitation_link))
        .route("/accept", post(accept_invitation_link))
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

async fn get_invitation_delivery(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
) -> impl IntoResponse {
    match service::get_invitation_delivery(&app_state, &tenant_context) {
        Ok(delivery) => (StatusCode::OK, axum::Json(serde_json::json!(delivery))),
        Err(error) => invitation_error_http_response(error),
    }
}

async fn resend_invitation(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Path(InvitationPath { invitation_id }): Path<InvitationPath>,
) -> impl IntoResponse {
    match service::resend_invitation(&app_state, &tenant_context, invitation_id).await {
        Ok(invitation) => (StatusCode::OK, axum::Json(serde_json::json!(invitation))),
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

async fn lookup_invitation_link(
    State(app_state): State<AppState>,
    cookies: Cookies,
    Json(link): Json<InvitationLinkTokenDto>,
) -> impl IntoResponse {
    let signed_in_user = authentication_service::authenticate_from_cookies(&cookies, &app_state)
        .await
        .ok()
        .map(|authenticated_user| authenticated_user.user_id());
    match service::lookup_invitation_link(&app_state, &link.token, signed_in_user).await {
        Ok(invitation_link) => (
            StatusCode::OK,
            axum::Json(serde_json::json!(invitation_link)),
        ),
        Err(error) => invitation_link_error_http_response(error),
    }
}

async fn sign_up_from_invitation_link(
    State(app_state): State<AppState>,
    cookies: Cookies,
    Json(sign_up): Json<InvitationSignUpDto>,
) -> impl IntoResponse {
    match service::sign_up_from_invitation_link(&app_state, &cookies, sign_up).await {
        Ok(accepted) => (StatusCode::CREATED, axum::Json(serde_json::json!(accepted))),
        Err(error) => invitation_link_error_http_response(error),
    }
}

async fn accept_invitation_link(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Json(link): Json<InvitationLinkTokenDto>,
) -> impl IntoResponse {
    match service::accept_invitation_link(&app_state, authenticated_user.user_id(), &link.token)
        .await
    {
        Ok(accepted) => (StatusCode::OK, axum::Json(serde_json::json!(accepted))),
        Err(error) => invitation_link_error_http_response(error),
    }
}

fn invitation_link_error_http_response(error: InvitationError) -> (StatusCode, axum::Json<Value>) {
    let (status, error_code) = match &error {
        InvitationError::LinkInvalid => (
            StatusCode::NOT_FOUND,
            Some(ErrorCode::InvitationLinkInvalid),
        ),
        InvitationError::InvitationForAnotherAccount => (
            StatusCode::FORBIDDEN,
            Some(ErrorCode::InvitationForAnotherAccount),
        ),
        InvitationError::AccountExists => (StatusCode::CONFLICT, Some(ErrorCode::AccountExists)),
        InvitationError::FullNameRequired => {
            (StatusCode::BAD_REQUEST, Some(ErrorCode::FullNameRequired))
        }
        InvitationError::PasswordRule(rule) => (StatusCode::BAD_REQUEST, Some(rule.error_code())),
        _ => (invitation_error_status(&error), None),
    };
    (
        status,
        axum::Json(serde_json::json!({
            "message": error.to_string(),
            "error_code": error_code.map(|error_code| error_code.as_str()),
        })),
    )
}

fn invitation_error_http_response(error: InvitationError) -> (StatusCode, axum::Json<Value>) {
    if let InvitationError::Unauthorized(authorization_error) = error {
        return crate::authorization::service::authorization_error_response(
            authorization_error,
            INVITATION_NOT_FOUND_MESSAGE,
        );
    }
    (
        invitation_error_status(&error),
        axum::Json(serde_json::json!({ "error": error.to_string() })),
    )
}

fn invitation_error_status(error: &InvitationError) -> StatusCode {
    match error {
        InvitationError::InvalidEmail
        | InvitationError::UnknownRole
        | InvitationError::FullNameRequired
        | InvitationError::PasswordRule(_) => StatusCode::BAD_REQUEST,
        InvitationError::Unauthorized(_)
        | InvitationError::RoleTooPowerful
        | InvitationError::InvitationForAnotherAccount => StatusCode::FORBIDDEN,
        InvitationError::AlreadyMember
        | InvitationError::AlreadyInvited
        | InvitationError::Expired
        | InvitationError::InvitationsUnavailable
        | InvitationError::ResendNeedsEmail
        | InvitationError::AccountExists => StatusCode::CONFLICT,
        InvitationError::NotFound | InvitationError::LinkInvalid => StatusCode::NOT_FOUND,
        InvitationError::Database(_)
        | InvitationError::EmailDelivery(_)
        | InvitationError::EmailTemplate(_)
        | InvitationError::AccountCreation(_)
        | InvitationError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
