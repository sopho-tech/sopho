use crate::authorization::errors::AuthorizationError;
use crate::grant::constants::{GRANT_NOT_FOUND_MESSAGE, RESOURCE_NOT_FOUND_MESSAGE};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GrantError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error("{0}")]
    InvalidRequest(String),
    #[error("{RESOURCE_NOT_FOUND_MESSAGE}")]
    ResourceNotFound,
    #[error("{GRANT_NOT_FOUND_MESSAGE}")]
    GrantNotFound,
    #[error("{0}")]
    Internal(String),
}
