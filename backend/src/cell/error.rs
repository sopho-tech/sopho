use crate::attribution::error::AttributionError;
use crate::authorization::errors::AuthorizationError;
use crate::cell::constants::{
    CELL_NOT_FOUND_MESSAGE, INVALID_SQL_CONTENT_MESSAGE, MISSING_CONNECTION_MESSAGE,
    NON_SERIALIZABLE_CONTENT_MESSAGE, NOTEBOOK_NOT_FOUND_MESSAGE, UNSUPPORTED_CELL_TYPE_MESSAGE,
    UNSUPPORTED_PREVIEW_CELL_TYPE_MESSAGE,
};
use crate::common::errors::{
    CreateCellError, ExecuteChartError, ExecuteQueryError, ExecuteSqlError,
    GetDatabaseConnectionError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CellError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error("{CELL_NOT_FOUND_MESSAGE}")]
    NotFound,
    #[error("{NOTEBOOK_NOT_FOUND_MESSAGE}")]
    NotebookNotFound,
    #[error("{0}")]
    NotebookFull(String),
    #[error("{0}")]
    InvalidRequest(String),
    #[error("{MISSING_CONNECTION_MESSAGE}")]
    MissingConnection,
    #[error("{NON_SERIALIZABLE_CONTENT_MESSAGE}")]
    NonSerializableContent,
    #[error("{INVALID_SQL_CONTENT_MESSAGE}")]
    InvalidSqlContent,
    #[error("{UNSUPPORTED_CELL_TYPE_MESSAGE}")]
    UnsupportedCellType,
    #[error("{UNSUPPORTED_PREVIEW_CELL_TYPE_MESSAGE}")]
    UnsupportedPreviewCellType,
    #[error("{0}")]
    GetConnection(#[from] GetDatabaseConnectionError),
    #[error("{0}")]
    ExecuteQuery(#[from] ExecuteQueryError),
    #[error("{0}")]
    ExecuteChart(#[from] ExecuteChartError),
    #[error("{0}")]
    Internal(String),
}

impl From<CreateCellError> for CellError {
    fn from(error: CreateCellError) -> Self {
        match error {
            CreateCellError::NotebookNotFound => CellError::NotebookNotFound,
            full @ CreateCellError::NotebookFull(_) => CellError::NotebookFull(full.to_string()),
            other => CellError::Internal(other.to_string()),
        }
    }
}

impl From<ExecuteSqlError> for CellError {
    fn from(error: ExecuteSqlError) -> Self {
        match error {
            ExecuteSqlError::GetConnection(connection_error) => {
                CellError::GetConnection(connection_error)
            }
            ExecuteSqlError::ExecuteQuery(query_error) => CellError::ExecuteQuery(query_error),
        }
    }
}

impl From<AttributionError> for CellError {
    fn from(error: AttributionError) -> Self {
        match error {
            AttributionError::Database(database_error) => CellError::Database(database_error),
            other => CellError::Internal(other.to_string()),
        }
    }
}
