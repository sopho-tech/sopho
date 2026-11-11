use crate::authentication::constants::{
    ACCOUNT_DEACTIVATED_MESSAGE, INVALID_CREDENTIALS_MESSAGE, TOO_MANY_SIGN_IN_ATTEMPTS_MESSAGE,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccountCreationError {
    #[error("Could not hash the password: {0}")]
    PasswordHashing(String),
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error("{INVALID_CREDENTIALS_MESSAGE}")]
    InvalidCredentials,
    #[error("{TOO_MANY_SIGN_IN_ATTEMPTS_MESSAGE}")]
    TooManyAttempts,
    #[error("{ACCOUNT_DEACTIVATED_MESSAGE}")]
    AccountDeactivated,
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}
