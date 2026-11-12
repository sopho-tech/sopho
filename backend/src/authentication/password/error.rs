use crate::authentication::constants::PASSWORD_SIGN_IN_OFF_MESSAGE;
use crate::authentication::error::CredentialsError;
use crate::authentication::password::constants::{
    INVALID_RESET_LINK_MESSAGE, NO_PASSWORD_MESSAGE, PASSWORD_CONTAINS_EMAIL_MESSAGE,
    PASSWORD_CONTAINS_SERVICE_NAME_MESSAGE, PASSWORD_RESET_UNAVAILABLE_MESSAGE,
    PASSWORD_TOO_COMMON_MESSAGE, PASSWORD_TOO_LONG_MESSAGE, PASSWORD_TOO_SHORT_MESSAGE,
    TOO_MANY_RESET_EMAILS_MESSAGE, WRONG_CURRENT_PASSWORD_MESSAGE,
};
use crate::common::error_codes::ErrorCode;
use crate::email_delivery::error::EmailDeliveryError;
use crate::email_template::error::EmailTemplateError;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PasswordRuleError {
    #[error("{PASSWORD_TOO_SHORT_MESSAGE}")]
    TooShort,
    #[error("{PASSWORD_TOO_LONG_MESSAGE}")]
    TooLong,
    #[error("{PASSWORD_TOO_COMMON_MESSAGE}")]
    Common,
    #[error("{PASSWORD_CONTAINS_SERVICE_NAME_MESSAGE}")]
    ContainsServiceName,
    #[error("{PASSWORD_CONTAINS_EMAIL_MESSAGE}")]
    ContainsEmail,
}

impl PasswordRuleError {
    pub fn error_code(&self) -> ErrorCode {
        match self {
            PasswordRuleError::TooShort => ErrorCode::PasswordTooShort,
            PasswordRuleError::TooLong => ErrorCode::PasswordTooLong,
            PasswordRuleError::Common => ErrorCode::PasswordTooCommon,
            PasswordRuleError::ContainsServiceName => ErrorCode::PasswordContainsServiceName,
            PasswordRuleError::ContainsEmail => ErrorCode::PasswordContainsEmail,
        }
    }
}

#[derive(Debug, Error)]
pub enum PasswordFlowError {
    #[error("{PASSWORD_SIGN_IN_OFF_MESSAGE}")]
    PasswordSignInOff,
    #[error("{PASSWORD_RESET_UNAVAILABLE_MESSAGE}")]
    ResetUnavailable,
    #[error("{TOO_MANY_RESET_EMAILS_MESSAGE}")]
    TooManyResetEmails,
    #[error("{INVALID_RESET_LINK_MESSAGE}")]
    InvalidResetLink,
    #[error("{WRONG_CURRENT_PASSWORD_MESSAGE}")]
    WrongCurrentPassword,
    #[error("{NO_PASSWORD_MESSAGE}")]
    NoPassword,
    #[error("{0}")]
    Credentials(CredentialsError),
    #[error("{0}")]
    Rule(#[from] PasswordRuleError),
    #[error("Could not hash the password: {0}")]
    Hashing(String),
    #[error("{0}")]
    EmailDelivery(#[from] EmailDeliveryError),
    #[error("{0}")]
    EmailTemplate(#[from] EmailTemplateError),
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

impl PasswordFlowError {
    pub fn error_code(&self) -> Option<ErrorCode> {
        match self {
            PasswordFlowError::PasswordSignInOff => Some(ErrorCode::PasswordSignInOff),
            PasswordFlowError::ResetUnavailable => Some(ErrorCode::PasswordResetUnavailable),
            PasswordFlowError::TooManyResetEmails => Some(ErrorCode::TooManyResetEmails),
            PasswordFlowError::InvalidResetLink => Some(ErrorCode::InvalidResetLink),
            PasswordFlowError::WrongCurrentPassword => Some(ErrorCode::WrongCurrentPassword),
            PasswordFlowError::NoPassword => Some(ErrorCode::NoPassword),
            PasswordFlowError::Credentials(CredentialsError::TooManyAttempts) => {
                Some(ErrorCode::TooManySignInAttempts)
            }
            PasswordFlowError::Credentials(CredentialsError::AccountDeactivated) => {
                Some(ErrorCode::AccountDeactivated)
            }
            PasswordFlowError::Rule(rule) => Some(rule.error_code()),
            PasswordFlowError::Credentials(_)
            | PasswordFlowError::Hashing(_)
            | PasswordFlowError::EmailDelivery(_)
            | PasswordFlowError::EmailTemplate(_)
            | PasswordFlowError::Database(_) => None,
        }
    }
}
