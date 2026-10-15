use crate::attribution::error::AttributionError;
use crate::authorization::errors::AuthorizationError;
use crate::canvas::constants::CANVAS_NOT_FOUND_MESSAGE;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanvasError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error("{CANVAS_NOT_FOUND_MESSAGE}")]
    NotFound,
    #[error("{0}")]
    Internal(String),
}

impl From<AttributionError> for CanvasError {
    fn from(error: AttributionError) -> Self {
        match error {
            AttributionError::Database(database_error) => CanvasError::Database(database_error),
            other => CanvasError::Internal(other.to_string()),
        }
    }
}
