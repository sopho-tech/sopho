use crate::authentication::dto;
use crate::authentication::service;
use crate::authorization::dto::AuthenticatedUser;
use crate::common::AppState;
use axum::{
    Router,
    extract::State,
    response::IntoResponse,
    routing::{delete, get, post},
};
use tower_cookies::Cookies;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/session", get(get_session))
        .route("/session", post(create_session))
        .route("/session", delete(delete_session))
        .route("/refresh", post(refresh_token))
        .route("/me", get(get_current_user))
        .route("/sign_in_option", get(get_sign_in_options))
        .route("/sign_in_method", get(get_sign_in_methods))
        .merge(crate::authentication::password::controller::routes())
        .nest("/oidc", crate::authentication::oidc::controller::routes())
        .with_state(app_state)
}

async fn get_sign_in_options(State(app_state): State<AppState>) -> impl IntoResponse {
    axum::Json(service::sign_in_options(&app_state))
}

async fn get_sign_in_methods(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> axum::response::Response {
    match crate::authentication::oidc::service::list_sign_in_methods(
        &app_state,
        authenticated_user.user_id().get_uuid(),
    )
    .await
    {
        Ok(sign_in_methods) => axum::Json(sign_in_methods).into_response(),
        Err(error) => crate::authentication::oidc::controller::sign_in_method_error_response(error)
            .into_response(),
    }
}

async fn get_session(cookies: Cookies, State(app_state): State<AppState>) -> impl IntoResponse {
    service::get_session(cookies, app_state).await
}

async fn create_session(
    cookies: Cookies,
    State(app_state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<dto::CreateSessionDto>,
) -> impl IntoResponse {
    service::create_session(app_state, payload, cookies).await
}

async fn get_current_user(
    cookies: Cookies,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    service::get_current_user(cookies, app_state).await
}

async fn delete_session(cookies: Cookies, State(app_state): State<AppState>) -> impl IntoResponse {
    service::signout(cookies, app_state).await
}

async fn refresh_token(cookies: Cookies, State(app_state): State<AppState>) -> impl IntoResponse {
    service::refresh_token(cookies, app_state).await
}
