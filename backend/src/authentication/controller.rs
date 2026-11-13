use crate::authentication::dto;
use crate::authentication::oidc::controller as oidc_controller;
use crate::authentication::password::controller as password_controller;
use crate::authentication::service;
use crate::authorization::dto::AuthenticatedUser;
use crate::authorization::service as authorization_service;
use crate::common::AppState;
use crate::rate_limit::service as rate_limit_service;
use axum::{
    Router,
    extract::State,
    response::IntoResponse,
    routing::{delete, get, post},
};
use tower_cookies::Cookies;

fn open_routes() -> Router<AppState> {
    Router::new()
        .route("/session", get(get_session))
        .route("/session", delete(delete_session))
        .route("/refresh", post(refresh_token))
        .route("/sign_in_page_option", get(get_sign_in_page_options))
}

fn sign_in_routes() -> Router<AppState> {
    Router::new()
        .route("/session", post(create_session))
        .merge(password_controller::sign_in_routes())
}

fn authenticated_routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_current_user))
        .route("/my_sign_in_method", get(get_my_sign_in_methods))
        .merge(password_controller::authenticated_routes())
}

pub fn routes(app_state: AppState) -> Router {
    let rate_limiters = &app_state.rate_limiters;
    let behind_the_authentication_guard = |router: Router<AppState>| {
        rate_limit_service::limit_api_requests(rate_limiters, router).route_layer(
            authorization_service::require_authentication_layer(&app_state),
        )
    };
    let oidc_routes = oidc_controller::open_routes()
        .merge(rate_limit_service::limit_sign_in_navigations(
            rate_limiters,
            oidc_controller::sign_in_navigation_routes(),
        ))
        .merge(behind_the_authentication_guard(
            oidc_controller::authenticated_routes(),
        ));
    open_routes()
        .merge(rate_limit_service::limit_sign_in_requests(
            rate_limiters,
            sign_in_routes(),
        ))
        .merge(behind_the_authentication_guard(authenticated_routes()))
        .nest("/oidc", oidc_routes)
        .with_state(app_state.clone())
}

async fn get_sign_in_page_options(State(app_state): State<AppState>) -> impl IntoResponse {
    axum::Json(service::sign_in_page_options(&app_state))
}

async fn get_my_sign_in_methods(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> axum::response::Response {
    match crate::authentication::oidc::service::list_my_sign_in_methods(
        &app_state,
        authenticated_user.user_id().get_uuid(),
    )
    .await
    {
        Ok(my_sign_in_methods) => axum::Json(my_sign_in_methods).into_response(),
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
