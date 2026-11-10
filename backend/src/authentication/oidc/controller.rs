use crate::authentication::oidc::dto::CallbackQuery;
use crate::authentication::oidc::error::SignInMethodError;
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
) -> Redirect {
    Redirect::to(&service::start_sign_in(&app_state, &cookies, &provider_name).await)
}

async fn complete_login(
    State(app_state): State<AppState>,
    cookies: Cookies,
    Path(ProviderPath { provider_name }): Path<ProviderPath>,
    Query(query): Query<CallbackQuery>,
) -> Redirect {
    Redirect::to(&service::complete_callback(&app_state, &cookies, &provider_name, query).await)
}

async fn start_connect(
    State(app_state): State<AppState>,
    cookies: Cookies,
    authenticated_user: AuthenticatedUser,
    Path(ProviderPath { provider_name }): Path<ProviderPath>,
) -> Redirect {
    Redirect::to(
        &service::start_connect(
            &app_state,
            &cookies,
            &provider_name,
            authenticated_user.user_id().get_uuid(),
        )
        .await,
    )
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
