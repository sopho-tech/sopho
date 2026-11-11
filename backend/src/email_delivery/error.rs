use crate::email_delivery::constants::{
    EMAIL_DISABLED_MESSAGE, EMAIL_NOT_FOUND_MESSAGE, EMAIL_NOT_RETRYABLE_MESSAGE,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmailDeliveryError {
    #[error("{EMAIL_DISABLED_MESSAGE}")]
    Disabled,
    #[error("{EMAIL_NOT_FOUND_MESSAGE}")]
    NotFound,
    #[error("{EMAIL_NOT_RETRYABLE_MESSAGE}")]
    NotRetryable,
    #[error("{recipient:?} is not an email address")]
    InvalidRecipient { recipient: String },
    #[error("Could not encrypt the email: {0}")]
    Encryption(String),
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Internal(String),
}
