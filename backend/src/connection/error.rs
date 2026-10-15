use crate::attribution::error::AttributionError;
use crate::authorization::errors::AuthorizationError;
use crate::common::errors::{
    CreateConnectionError, GetDatabaseConnectionError, UpdateConnectionError,
};
use crate::connection::constants::CONNECTION_NOT_FOUND_MESSAGE;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error("{CONNECTION_NOT_FOUND_MESSAGE}")]
    NotFound,
    #[error("{0}")]
    InvalidRequest(String),
    #[error("Failed to encrypt connection")]
    Encryption,
    #[error("{0}")]
    Internal(String),
}

impl From<CreateConnectionError> for ConnectionError {
    fn from(error: CreateConnectionError) -> Self {
        match error {
            CreateConnectionError::ConnectionStatus(status_error) => {
                ConnectionError::InvalidRequest(status_error.to_string())
            }
            CreateConnectionError::Encryption => ConnectionError::Encryption,
            decryption_error @ CreateConnectionError::Decryption(_) => {
                ConnectionError::Internal(decryption_error.to_string())
            }
            CreateConnectionError::Repository(database_error) => {
                ConnectionError::Database(database_error)
            }
        }
    }
}

impl From<GetDatabaseConnectionError> for ConnectionError {
    fn from(error: GetDatabaseConnectionError) -> Self {
        match error {
            GetDatabaseConnectionError::ConnectionNotFound => ConnectionError::NotFound,
            GetDatabaseConnectionError::Connection(database_error) => {
                ConnectionError::Database(database_error)
            }
            other => ConnectionError::Internal(other.to_string()),
        }
    }
}

impl From<UpdateConnectionError> for ConnectionError {
    fn from(error: UpdateConnectionError) -> Self {
        match error {
            UpdateConnectionError::GetConnection(get_connection_error) => {
                get_connection_error.into()
            }
            UpdateConnectionError::ConnectionStatus(status_error) => {
                ConnectionError::InvalidRequest(status_error.to_string())
            }
            UpdateConnectionError::Encryption => ConnectionError::Encryption,
            decryption_error @ UpdateConnectionError::Decryption(_) => {
                ConnectionError::Internal(decryption_error.to_string())
            }
            UpdateConnectionError::Repository(database_error) => {
                ConnectionError::Database(database_error)
            }
        }
    }
}

impl From<AttributionError> for ConnectionError {
    fn from(error: AttributionError) -> Self {
        match error {
            AttributionError::Database(database_error) => ConnectionError::Database(database_error),
            other => ConnectionError::Internal(other.to_string()),
        }
    }
}
