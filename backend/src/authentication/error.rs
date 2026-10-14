use crate::authentication::constants::INVALID_CREDENTIALS_MESSAGE;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error("{INVALID_CREDENTIALS_MESSAGE}")]
    InvalidCredentials,
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}
