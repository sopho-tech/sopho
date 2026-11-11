use crate::authorization::errors::AuthorizationError;
use crate::notebook::constants::{CANVAS_NOT_FOUND_MESSAGE, NOTEBOOK_NOT_FOUND_MESSAGE};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotebookError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error("{NOTEBOOK_NOT_FOUND_MESSAGE}")]
    NotFound,
    #[error("{CANVAS_NOT_FOUND_MESSAGE}")]
    CanvasNotFound,
    #[error("{0}")]
    Internal(String),
}
