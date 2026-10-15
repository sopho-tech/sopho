use crate::authorization::dto::TenantContext;
use crate::common::AppState;
use crate::search::dto::SearchRequestDto;
use crate::search::service;
use axum::Router;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::get;

pub fn routes(app_state: AppState) -> Router {
    Router::new().route("/", get(search)).with_state(app_state)
}

async fn search(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Query(request): Query<SearchRequestDto>,
) -> impl IntoResponse {
    service::search(app_state, &tenant_context, request).await
}
