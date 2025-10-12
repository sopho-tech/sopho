use axum::{
    routing::{post, get},
    Router,
    extract::State,
    response::IntoResponse,
};
use crate::common::AppState;
use crate::authentication::dto;
use crate::authentication::service;
use tower_cookies::Cookies;


pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/session", get(get_session))
        .route("/session", post(create_session))
        .with_state(app_state)
}

async fn get_session(
    cookies: Cookies,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    service::get_session(cookies, app_state).await
}

async fn create_session(
    cookies: Cookies,
    State(app_state): State<AppState>,
    axum::extract::Json(payload): axum::extract::Json<dto::CreateSessionDto>
) -> impl IntoResponse {
    service::create_session(app_state, payload, cookies).await
}
