use crate::authorization::dto::TenantContext;
use crate::common::constants::UNKNOWN_CREATED_BY_FILTER_MESSAGE;
use crate::common::error_codes::codes;
use crate::common::errors::{ExecuteQueryError, ExecuteSqlError, GetDatabaseConnectionError};
use crate::common::{AppState, CreatedByQuery};
use crate::connection::constants::CONNECTION_NOT_FOUND_MESSAGE;
use crate::connection::dto;
use crate::connection::error::ConnectionError;
use crate::connection::service;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct ConnectionPath {
    id: Uuid,
}

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/{id}", get(get_connection))
        .route("/", get(get_all_connections))
        .route("/", post(create_connection))
        .route("/{id}", put(update_connection))
        .route("/{id}/execute-query", post(execute_query))
        .with_state(app_state)
}

async fn get_connection(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Path(ConnectionPath { id }): axum::extract::Path<ConnectionPath>,
) -> impl IntoResponse {
    match service::get_connection(app_state, &tenant_context, id).await {
        Ok(connection) => (StatusCode::OK, axum::Json(serde_json::json!(connection))),
        Err(error) => connection_error_http_response(error),
    }
}

async fn create_connection(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Json(payload): axum::extract::Json<dto::CreateConnectionDto>,
) -> impl IntoResponse {
    match service::create_connection(&app_state, &tenant_context, payload).await {
        Ok(connection) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!(connection)),
        ),
        Err(error) => connection_error_http_response(error),
    }
}

async fn get_all_connections(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    created_by_query: Query<CreatedByQuery>,
) -> impl IntoResponse {
    let Ok(created_by) = created_by_query.filter() else {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": UNKNOWN_CREATED_BY_FILTER_MESSAGE })),
        );
    };
    match service::get_all_connections(app_state, &tenant_context, created_by).await {
        Ok(connections) => (StatusCode::OK, axum::Json(serde_json::json!(connections))),
        Err(error) => connection_error_http_response(error),
    }
}

async fn update_connection(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Path(ConnectionPath { id }): axum::extract::Path<ConnectionPath>,
    axum::extract::Json(payload): axum::extract::Json<dto::UpdateConnectionDto>,
) -> impl IntoResponse {
    match service::update_connection(app_state, &tenant_context, id, payload).await {
        Ok(connection) => (StatusCode::OK, axum::Json(serde_json::json!(connection))),
        Err(error) => connection_error_http_response(error),
    }
}

async fn execute_query(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Path(ConnectionPath { id }): axum::extract::Path<ConnectionPath>,
    axum::extract::Json(payload): axum::extract::Json<dto::ExecuteQueryDto>,
) -> impl IntoResponse {
    match service::execute_query(&app_state, &tenant_context, id, payload).await {
        Ok(result) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "columns": result.columns,
                "data": result.data,
            })),
        ),
        Err(ExecuteSqlError::GetConnection(GetDatabaseConnectionError::ConnectionNotFound)) => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Connection not found" })),
        ),
        Err(ExecuteSqlError::GetConnection(e @ GetDatabaseConnectionError::Decryption(_))) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
        Err(ExecuteSqlError::GetConnection(e)) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
        Err(ExecuteSqlError::ExecuteQuery(ExecuteQueryError::Database(sqlx::Error::Database(
            e,
        )))) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({
                "status": StatusCode::BAD_REQUEST.as_u16(),
                "code": codes::SYNTAX_ERROR.as_str(),
                "message": e.to_string()
            })),
        ),
        Err(ExecuteSqlError::ExecuteQuery(ExecuteQueryError::Database(e))) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
        Err(ExecuteSqlError::ExecuteQuery(ExecuteQueryError::UnhandledDataType(type_name))) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "error": format!("data type '{}' not handled", type_name)
            })),
        ),
    }
}

fn connection_error_http_response(
    error: ConnectionError,
) -> (StatusCode, axum::Json<serde_json::Value>) {
    match error {
        ConnectionError::Unauthorized(authorization_error) => {
            crate::authorization::service::authorization_error_response(
                authorization_error,
                CONNECTION_NOT_FOUND_MESSAGE,
            )
        }
        ConnectionError::NotFound => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        ConnectionError::InvalidRequest(_) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        ConnectionError::Encryption
        | ConnectionError::Database(_)
        | ConnectionError::Internal(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}
