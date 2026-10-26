use crate::authentication::oidc::constants::{
    AFTER_SIGN_IN_PATH, CONNECT_RESULT_PARAMETER, CONNECTED_PROVIDER_PARAMETER, CallbackPurpose,
    ConnectResult, LOGIN_ATTEMPT_EXPIRY_MINUTES, OIDC_COOKIE_PATH, OIDC_STATE_COOKIE_NAME,
    PROFILE_PATH, SIGN_IN_ERROR_PARAMETER, SIGN_IN_PATH,
};
use crate::authentication::oidc::dto::{CallbackOutcome, CallbackQuery};
use crate::authentication::oidc::error::{OidcLoginError, SignInMethodError};
use crate::authentication::oidc::service;
use crate::authorization::dto::AuthenticatedUser;
use crate::common::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::Deserialize;
use tower_cookies::Cookies;
use tower_cookies::cookie::{Cookie, SameSite};

#[derive(Deserialize)]
struct ProviderPath {
    provider_name: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/provider", get(list_providers))
        .route("/{provider_name}/start", get(start_login))
        .route("/{provider_name}/connect", get(start_connect))
        .route("/{provider_name}/callback", get(complete_login))
        .route("/{provider_name}/identity", delete(disconnect_provider))
}

async fn list_providers(State(app_state): State<AppState>) -> impl IntoResponse {
    Json(service::list_providers(&app_state))
}

async fn start_login(
    State(app_state): State<AppState>,
    cookies: Cookies,
    Path(ProviderPath { provider_name }): Path<ProviderPath>,
) -> Response {
    match service::start_login(&app_state, &provider_name, None).await {
        Ok(started) => {
            cookies.add(state_cookie(&app_state, started.state));
            Redirect::to(&started.authorization_url).into_response()
        }
        Err(error) => {
            tracing::error!("Sign-in with {provider_name} could not start: {error}");
            sign_in_failure(&error).into_response()
        }
    }
}

async fn complete_login(
    State(app_state): State<AppState>,
    cookies: Cookies,
    Path(ProviderPath { provider_name }): Path<ProviderPath>,
    Query(query): Query<CallbackQuery>,
) -> Response {
    let cookie_state = cookies
        .get(OIDC_STATE_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string());
    cookies.remove(state_cookie_removal());

    match service::complete_login(&app_state, &cookies, &provider_name, query, cookie_state).await {
        Ok(CallbackOutcome::SignedIn) => Redirect::to(AFTER_SIGN_IN_PATH).into_response(),
        Ok(CallbackOutcome::Connected { provider_name }) => Redirect::to(&format!(
            "{PROFILE_PATH}?{CONNECT_RESULT_PARAMETER}={}&{CONNECTED_PROVIDER_PARAMETER}={provider_name}",
            ConnectResult::Connected.as_str()
        ))
        .into_response(),
        Err(failure) => {
            tracing::error!("Single sign-on with {provider_name} failed: {}", failure.error);
            match failure.purpose {
                CallbackPurpose::SignIn => sign_in_failure(&failure.error).into_response(),
                CallbackPurpose::Connect => {
                    connect_failure(failure.error.connect_result()).into_response()
                }
            }
        }
    }
}

async fn start_connect(
    State(app_state): State<AppState>,
    cookies: Cookies,
    authenticated_user: AuthenticatedUser,
    Path(ProviderPath { provider_name }): Path<ProviderPath>,
) -> Response {
    match service::start_login(
        &app_state,
        &provider_name,
        Some(authenticated_user.user_id().get_uuid()),
    )
    .await
    {
        Ok(started) => {
            cookies.add(state_cookie(&app_state, started.state));
            Redirect::to(&started.authorization_url).into_response()
        }
        Err(error) => {
            tracing::error!("Connecting {provider_name} could not start: {error}");
            connect_failure(error.connect_result()).into_response()
        }
    }
}

async fn disconnect_provider(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(ProviderPath { provider_name }): Path<ProviderPath>,
) -> Response {
    match service::disconnect_provider(
        &app_state,
        authenticated_user.user_id().get_uuid(),
        &provider_name,
    )
    .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => sign_in_method_error_response(error).into_response(),
    }
}

pub fn sign_in_method_error_response(
    error: SignInMethodError,
) -> (StatusCode, Json<serde_json::Value>) {
    let status = match &error {
        SignInMethodError::NotConnected => StatusCode::NOT_FOUND,
        SignInMethodError::LastSignInMethod => StatusCode::CONFLICT,
        SignInMethodError::EmailDelivery(_)
        | SignInMethodError::EmailTemplate(_)
        | SignInMethodError::Database(_) => {
            tracing::error!("Sign-in method request failed: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    (
        status,
        Json(serde_json::json!({
            "message": error.to_string(),
            "error_code": error.error_code().map(|error_code| error_code.as_str()),
        })),
    )
}

fn connect_failure(connect_result: ConnectResult) -> Redirect {
    Redirect::to(&format!(
        "{PROFILE_PATH}?{CONNECT_RESULT_PARAMETER}={}",
        connect_result.as_str()
    ))
}

fn state_cookie(app_state: &AppState, state: String) -> Cookie<'static> {
    Cookie::build((OIDC_STATE_COOKIE_NAME, state))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path(OIDC_COOKIE_PATH)
        .secure(app_state.config.cookie_secure)
        .max_age(time::Duration::minutes(LOGIN_ATTEMPT_EXPIRY_MINUTES))
        .build()
}

fn state_cookie_removal() -> Cookie<'static> {
    Cookie::build((OIDC_STATE_COOKIE_NAME, ""))
        .path(OIDC_COOKIE_PATH)
        .build()
}

fn sign_in_failure(error: &OidcLoginError) -> Redirect {
    Redirect::to(&format!(
        "{SIGN_IN_PATH}?{SIGN_IN_ERROR_PARAMETER}={}",
        error.sign_in_error_code().as_str()
    ))
}
