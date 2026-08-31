use crate::authorization::errors::AuthorizationError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SuggestedQuestionError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
}
