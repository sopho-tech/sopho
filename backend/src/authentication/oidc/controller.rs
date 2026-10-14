use crate::authentication::oidc::constants::{
    AFTER_SIGN_IN_PATH, LOGIN_ATTEMPT_EXPIRY_MINUTES, OIDC_COOKIE_PATH, OIDC_STATE_COOKIE_NAME,
    SIGN_IN_ERROR_PARAMETER, SIGN_IN_PATH,
};
use crate::authentication::oidc::dto::CallbackQuery;
use crate::authentication::oidc::error::OidcLoginError;
use crate::authentication::oidc::service;
use crate::common::AppState;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use tower_cookies::cookie::{Cookie, SameSite};
use tower_cookies::Cookies;

#[derive(Deserialize)]
struct ProviderPath {
    provider_name: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/provider", get(list_providers))
        .route("/{provider_name}/start", get(start_login))
        .route("/{provider_name}/callback", get(complete_login))
}

async fn list_providers(State(app_state): State<AppState>) -> impl IntoResponse {
    Json(service::list_providers(&app_state))
}

async fn start_login(
    State(app_state): State<AppState>,
    cookies: Cookies,
    Path(ProviderPath { provider_name }): Path<ProviderPath>,
) -> Response {
    match service::start_login(&app_state, &provider_name).await {
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
        Ok(()) => Redirect::to(AFTER_SIGN_IN_PATH).into_response(),
        Err(error) => {
            tracing::error!("Sign-in with {provider_name} failed: {error}");
            sign_in_failure(&error).into_response()
        }
    }
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
