use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde_json;

pub fn routes() -> Router {
    Router::new().route("/health", get(health_check))
}

async fn health_check() -> impl IntoResponse {
    axum::Json(serde_json::json!({ "status": "OK" }))
}
