mod authentication;
mod canvas;
mod cell;
mod common;
mod connection;
mod dashboard;
mod db;
mod entity;
mod middlewares;
mod monitor;
mod notebook;

use crate::common::AppState;
use axum::http::HeaderValue;
use axum::http::Method;
use axum::response::Html;
use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use dotenv::dotenv;
use http::header;
use http_body_util::BodyExt;
use std::env;
use std::path::PathBuf;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let app_state = AppState::from_env().await.unwrap();

    db::run_migrations(&app_state.database_connection)
        .await
        .unwrap();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    authentication::service::seed_admin_user(&app_state).await;

    let frontend_dir = PathBuf::from(app_state.config.frontend_dir.as_ref());
    let index_path = frontend_dir.join("index.html");
    let static_files_service = ServeDir::new(&frontend_dir);

    let mut app = Router::new()
        .nest("/api/v1/monitor", monitor::routes())
        .nest("/api/v1/auth", authentication::routes(app_state.clone()))
        .nest(
            "/api/v1/dashboard",
            dashboard::routes(app_state.clone()).layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                middlewares::auth_middleware_fn,
            )),
        )
        .nest(
            "/api/v1/connection",
            connection::routes(app_state.clone()).layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                middlewares::auth_middleware_fn,
            )),
        )
        .nest(
            "/api/v1/notebook",
            notebook::routes(app_state.clone()).layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                middlewares::auth_middleware_fn,
            )),
        )
        .nest(
            "/api/v1/cell",
            cell::routes(app_state.clone()).layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                middlewares::auth_middleware_fn,
            )),
        )
        .nest(
            "/api/v1/canvas",
            canvas::routes(app_state.clone()).layer(axum::middleware::from_fn_with_state(
                app_state.clone(),
                middlewares::auth_middleware_fn,
            )),
        )
        .fallback_service(static_files_service)
        .layer(middleware::from_fn(move |req: Request, next: Next| {
            let index_path = index_path.clone();
            async move {
                let res = next.run(req).await;
                let (parts, body) = res.into_parts();
                let status = parts.status;

                // If the response is 404, serve index.html instead
                if status == StatusCode::NOT_FOUND {
                    match tokio::fs::read_to_string(&index_path).await {
                        Ok(html) => Html(html).into_response(),
                        Err(_) => Response::from_parts(parts, body),
                    }
                } else {
                    Response::from_parts(parts, body)
                }
            }
        }))
        .layer(middleware::from_fn(print_request_response))
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new());

    if app_state.config.environment == "development" {
        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_origin(HeaderValue::from_str("http://localhost:3000").unwrap())
            .allow_credentials(true)
            .allow_headers([header::CONTENT_TYPE]);
        app = app.layer(cors);
    }

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{direction} body = {body:?}");
    }

    Ok(bytes)
}
