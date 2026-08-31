use crate::authorization::dto::TenantContext;
use crate::cell::constants::{
    CELL_CONTENT_NOT_SQL_MESSAGE, CELL_HAS_NO_CONTENT_MESSAGE, CELL_NOT_FOUND_MESSAGE,
    CONNECTION_NOT_FOUND_MESSAGE, INVALID_CHART_CONTENT_MESSAGE, MISSING_CONNECTION_MESSAGE,
    SOURCE_CELL_MISSING_CONNECTION_MESSAGE,
};
use crate::cell::dto;
use crate::cell::error::CellError;
use crate::cell::service;
use crate::common::error_codes::codes;
use crate::common::errors::{ExecuteChartError, ExecuteQueryError, GetDatabaseConnectionError};
use crate::common::AppState;
use crate::database::constants::QueryResult;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use serde_json::Value;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct CellPath {
    id: Uuid,
}

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/", post(create_cell))
        .route("/{id}", get(get_cell))
        .route("/{id}", put(update_cell))
        .route("/{id}", delete(delete_cell))
        .route("/{id}/execute", post(execute_cell))
        .route("/{id}/execute-preview", post(execute_cell_preview))
        .route("/{id}/reorder", patch(reorder_cell))
        .with_state(app_state)
}

async fn get_cell(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Path(CellPath { id }): axum::extract::Path<CellPath>,
) -> impl IntoResponse {
    match service::get_cell(app_state, &tenant_context, id).await {
        Ok(cell) => (StatusCode::OK, axum::Json(serde_json::json!(cell))),
        Err(error) => cell_error_http_response(error),
    }
}

async fn create_cell(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Json(payload): axum::extract::Json<dto::CreateCellDto>,
) -> impl IntoResponse {
    match service::create_cell(app_state, &tenant_context, payload).await {
        Ok(cell) => (StatusCode::CREATED, axum::Json(serde_json::json!(cell))),
        Err(error) => cell_error_http_response(error),
    }
}

async fn update_cell(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Path(CellPath { id }): axum::extract::Path<CellPath>,
    axum::extract::Json(payload): axum::extract::Json<dto::CellDto>,
) -> impl IntoResponse {
    match service::update_cell(app_state, &tenant_context, id, payload).await {
        Ok(cell) => (StatusCode::OK, axum::Json(serde_json::json!(cell))),
        Err(error) => cell_error_http_response(error),
    }
}

async fn execute_cell(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Path(CellPath { id }): axum::extract::Path<CellPath>,
) -> impl IntoResponse {
    match service::execute_cell(app_state, &tenant_context, id).await {
        Ok(result) => query_result_response(&result),
        Err(error) => cell_error_http_response(error),
    }
}

async fn execute_cell_preview(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Path(CellPath { id }): axum::extract::Path<CellPath>,
    axum::extract::Json(payload): axum::extract::Json<dto::ExecuteCellPreviewDto>,
) -> impl IntoResponse {
    match service::execute_cell_preview(app_state, &tenant_context, id, payload).await {
        Ok(result) => query_result_response(&result),
        Err(error) => cell_error_http_response(error),
    }
}

async fn delete_cell(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Path(CellPath { id }): axum::extract::Path<CellPath>,
) -> impl IntoResponse {
    match service::delete_cell(app_state, &tenant_context, id).await {
        Ok(()) => (StatusCode::NO_CONTENT, axum::Json(Value::Null)),
        Err(error) => cell_error_http_response(error),
    }
}

async fn reorder_cell(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    axum::extract::Path(CellPath { id }): axum::extract::Path<CellPath>,
    axum::extract::Json(payload): axum::extract::Json<dto::ReorderCellDto>,
) -> impl IntoResponse {
    match service::reorder_cell(app_state, &tenant_context, id, payload).await {
        Ok(cell) => (StatusCode::OK, axum::Json(serde_json::json!(cell))),
        Err(error) => cell_error_http_response(error),
    }
}

fn query_result_response(result: &QueryResult) -> (StatusCode, axum::Json<Value>) {
    (
        StatusCode::OK,
        axum::Json(serde_json::json!({
            "columns": result.columns,
            "data": result.data,
        })),
    )
}

fn cell_error_http_response(error: CellError) -> (StatusCode, axum::Json<Value>) {
    match error {
        CellError::Unauthorized(authorization_error) => {
            crate::authorization::service::authorization_error_response(
                authorization_error,
                CELL_NOT_FOUND_MESSAGE,
            )
        }
        CellError::NotFound => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        CellError::NotebookNotFound
        | CellError::InvalidSqlContent
        | CellError::UnsupportedPreviewCellType
        | CellError::InvalidRequest(_) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        CellError::NotebookFull(_) => (
            StatusCode::PRECONDITION_FAILED,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        CellError::UnsupportedCellType => (
            StatusCode::EXPECTATION_FAILED,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        CellError::MissingConnection => (
            StatusCode::PRECONDITION_FAILED,
            axum::Json(serde_json::json!({
                "status": StatusCode::PRECONDITION_FAILED.as_u16(),
                "code": codes::MISSING_PREREQUISITES.as_str(),
                "message": MISSING_CONNECTION_MESSAGE
            })),
        ),
        CellError::NonSerializableContent | CellError::Database(_) | CellError::Internal(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        CellError::GetConnection(connection_error) => {
            get_database_connection_error_response(connection_error)
        }
        CellError::ExecuteQuery(query_error) => execute_query_error_response(query_error),
        CellError::ExecuteChart(chart_error) => execute_chart_error_response(chart_error),
    }
}

fn get_database_connection_error_response(
    error: GetDatabaseConnectionError,
) -> (StatusCode, axum::Json<Value>) {
    match error {
        GetDatabaseConnectionError::ConnectionNotFound => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": CONNECTION_NOT_FOUND_MESSAGE })),
        ),
        GetDatabaseConnectionError::Connection(_)
        | GetDatabaseConnectionError::SourceTypeParse(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
        GetDatabaseConnectionError::DatabaseConnection(_)
        | GetDatabaseConnectionError::UnsupportedSourceType(_)
        | GetDatabaseConnectionError::PoolUnavailable(_) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}

fn execute_query_error_response(error: ExecuteQueryError) -> (StatusCode, axum::Json<Value>) {
    match error {
        ExecuteQueryError::Database(sqlx::Error::Database(database_error)) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({
                "status": StatusCode::BAD_REQUEST.as_u16(),
                "code": codes::SYNTAX_ERROR.as_str(),
                "message": database_error.to_string()
            })),
        ),
        ExecuteQueryError::Database(_) | ExecuteQueryError::UnhandledDataType(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}

fn execute_chart_error_response(error: ExecuteChartError) -> (StatusCode, axum::Json<Value>) {
    match error {
        ExecuteChartError::CellNotFound => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": CELL_NOT_FOUND_MESSAGE })),
        ),
        ExecuteChartError::MissingContent => (
            StatusCode::PRECONDITION_FAILED,
            axum::Json(serde_json::json!({ "error": CELL_HAS_NO_CONTENT_MESSAGE })),
        ),
        ExecuteChartError::InvalidChartContent => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": INVALID_CHART_CONTENT_MESSAGE })),
        ),
        ExecuteChartError::SourceNotSql => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": CELL_CONTENT_NOT_SQL_MESSAGE })),
        ),
        ExecuteChartError::MissingConnection => (
            StatusCode::PRECONDITION_FAILED,
            axum::Json(serde_json::json!({ "error": SOURCE_CELL_MISSING_CONNECTION_MESSAGE })),
        ),
        ExecuteChartError::MissingChartSetting(_)
        | ExecuteChartError::TooManySeries { .. }
        | ExecuteChartError::DuplicateSeriesAlias(_)
        | ExecuteChartError::InvalidIdentifier(_) => (
            StatusCode::PRECONDITION_FAILED,
            axum::Json(serde_json::json!({ "message": error.to_string() })),
        ),
        ExecuteChartError::ExecuteSql(sql_error) => match sql_error {
            crate::common::errors::ExecuteSqlError::GetConnection(connection_error) => {
                get_database_connection_error_response(connection_error)
            }
            crate::common::errors::ExecuteSqlError::ExecuteQuery(query_error) => {
                execute_query_error_response(query_error)
            }
        },
        ExecuteChartError::Repository(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}
