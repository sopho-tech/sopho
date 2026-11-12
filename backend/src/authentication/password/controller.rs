use crate::authentication::error::CredentialsError;
use crate::authentication::password::constants::RESET_REQUESTED_MESSAGE;
use crate::authentication::password::dto::{
    ChangePasswordDto, PasswordResetDto, PasswordResetRequestDto, PasswordResetRequestedDto,
};
use crate::authentication::password::error::PasswordFlowError;
use crate::authentication::password::service;
use crate::authorization::dto::AuthenticatedUser;
use crate::common::AppState;
use axum::Router;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{post, put};
use serde_json::Value;
use tower_cookies::Cookies;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/password_reset_request", post(request_password_reset))
        .route("/password_reset", post(reset_password))
        .route("/password", put(change_password))
}

async fn request_password_reset(
    State(app_state): State<AppState>,
    Json(request): Json<PasswordResetRequestDto>,
) -> Response {
    match service::request_password_reset(app_state, request.email) {
        Ok(()) => (
            StatusCode::ACCEPTED,
            axum::Json(PasswordResetRequestedDto {
                message: RESET_REQUESTED_MESSAGE.to_string(),
            }),
        )
            .into_response(),
        Err(error) => password_error_response(error).into_response(),
    }
}

async fn reset_password(
    State(app_state): State<AppState>,
    Json(reset): Json<PasswordResetDto>,
) -> Response {
    match service::reset_password(&app_state, reset).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => password_error_response(error).into_response(),
    }
}

async fn change_password(
    State(app_state): State<AppState>,
    cookies: Cookies,
    authenticated_user: AuthenticatedUser,
    Json(change): Json<ChangePasswordDto>,
) -> Response {
    match service::change_password(&app_state, &cookies, authenticated_user.user_id(), change).await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => password_error_response(error).into_response(),
    }
}

fn password_error_response(error: PasswordFlowError) -> (StatusCode, axum::Json<Value>) {
    let status = match &error {
        PasswordFlowError::ResetUnavailable | PasswordFlowError::NoPassword => StatusCode::CONFLICT,
        PasswordFlowError::TooManyResetEmails
        | PasswordFlowError::Credentials(CredentialsError::TooManyAttempts) => {
            StatusCode::TOO_MANY_REQUESTS
        }
        PasswordFlowError::InvalidResetLink
        | PasswordFlowError::WrongCurrentPassword
        | PasswordFlowError::Rule(_) => StatusCode::BAD_REQUEST,
        PasswordFlowError::PasswordSignInOff
        | PasswordFlowError::Credentials(CredentialsError::AccountDeactivated) => {
            StatusCode::FORBIDDEN
        }
        PasswordFlowError::Credentials(CredentialsError::InvalidCredentials) => {
            StatusCode::UNAUTHORIZED
        }
        PasswordFlowError::Credentials(CredentialsError::Database(_))
        | PasswordFlowError::Hashing(_)
        | PasswordFlowError::EmailDelivery(_)
        | PasswordFlowError::EmailTemplate(_)
        | PasswordFlowError::Database(_) => {
            tracing::error!("Password request failed: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    (
        status,
        axum::Json(serde_json::json!({
            "message": error.to_string(),
            "error_code": error.error_code().map(|error_code| error_code.as_str()),
        })),
    )
}
