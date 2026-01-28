use crate::authentication::dto;
use crate::authentication::service;
use crate::common::AppState;
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use tower_cookies::Cookies;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/session", get(get_session))
        .route("/session", post(create_session))
        .route("/session", delete(delete_session))
        .route("/refresh", post(refresh_token))
        .route("/me", get(get_current_user))
        .with_state(app_state)
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
