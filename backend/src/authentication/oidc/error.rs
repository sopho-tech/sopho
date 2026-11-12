use crate::authentication::oidc::constants::{
    CallbackPurpose, ConnectResult, LAST_SIGN_IN_METHOD_MESSAGE,
    SIGN_IN_METHOD_NOT_CONNECTED_MESSAGE, SignInErrorCode,
};
use crate::common::error_codes::ErrorCode;
use crate::email_delivery::error::EmailDeliveryError;
use crate::email_template::error::EmailTemplateError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OidcLoginError {
    #[error("Unknown sign-in provider {0}")]
    UnknownProvider(String),
    #[error("The sign-in attempt was not found")]
    AttemptNotFound,
    #[error("The sign-in attempt expired")]
    AttemptExpired,
    #[error("The sign-in did not start in this browser")]
    StateMismatch,
    #[error("The provider did not share an email address")]
    MissingEmail,
    #[error("The provider has not verified this email address")]
    EmailNotVerified,
    #[error("The provider did not share a name")]
    MissingProfile,
    #[error("This email belongs to an account that signs in with a password")]
    AccountRequiresPasswordSignIn,
    #[error("This email belongs to an account that has no way to sign in while passwords are off")]
    AccountCannotSignIn,
    #[error("This account has been deactivated")]
    AccountDeactivated,
    #[error("This account already signs in with another account at this provider")]
    SecondFederatedIdentityAtProvider,
    #[error("The provider's email does not match this account's email")]
    EmailMismatch,
    #[error("This sign-in is connected to another account")]
    FederatedIdentityInUse,
    #[error("This account is already connected to this provider")]
    AlreadyConnected,
    #[error("{0}")]
    EmailDelivery(#[from] EmailDeliveryError),
    #[error("{0}")]
    EmailTemplate(#[from] EmailTemplateError),
    #[error("The provider rejected the sign-in: {0}")]
    Provider(String),
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Internal(String),
}

impl OidcLoginError {
    pub fn sign_in_error_code(&self) -> SignInErrorCode {
        match self {
            OidcLoginError::MissingEmail | OidcLoginError::EmailNotVerified => {
                SignInErrorCode::EmailNotVerified
            }
            OidcLoginError::AttemptNotFound
            | OidcLoginError::AttemptExpired
            | OidcLoginError::StateMismatch => SignInErrorCode::SsoExpired,
            OidcLoginError::AccountRequiresPasswordSignIn => SignInErrorCode::PasswordAccount,
            OidcLoginError::AccountCannotSignIn => SignInErrorCode::ContactAdministrator,
            OidcLoginError::AccountDeactivated => SignInErrorCode::AccountDeactivated,
            OidcLoginError::SecondFederatedIdentityAtProvider
            | OidcLoginError::EmailMismatch
            | OidcLoginError::FederatedIdentityInUse
            | OidcLoginError::AlreadyConnected
            | OidcLoginError::EmailDelivery(_)
            | OidcLoginError::EmailTemplate(_)
            | OidcLoginError::UnknownProvider(_)
            | OidcLoginError::MissingProfile
            | OidcLoginError::Provider(_)
            | OidcLoginError::Database(_)
            | OidcLoginError::Internal(_) => SignInErrorCode::SsoFailed,
        }
    }
}

impl OidcLoginError {
    pub fn connect_result(&self) -> ConnectResult {
        match self {
            OidcLoginError::MissingEmail | OidcLoginError::EmailNotVerified => {
                ConnectResult::EmailNotVerified
            }
            OidcLoginError::EmailMismatch => ConnectResult::EmailMismatch,
            OidcLoginError::FederatedIdentityInUse => ConnectResult::FederatedIdentityInUse,
            OidcLoginError::AlreadyConnected
            | OidcLoginError::SecondFederatedIdentityAtProvider => ConnectResult::AlreadyConnected,
            OidcLoginError::AccountDeactivated => ConnectResult::AccountDeactivated,
            OidcLoginError::AttemptNotFound
            | OidcLoginError::AttemptExpired
            | OidcLoginError::StateMismatch => ConnectResult::SsoExpired,
            OidcLoginError::UnknownProvider(_)
            | OidcLoginError::MissingProfile
            | OidcLoginError::AccountRequiresPasswordSignIn
            | OidcLoginError::AccountCannotSignIn
            | OidcLoginError::Provider(_)
            | OidcLoginError::EmailDelivery(_)
            | OidcLoginError::EmailTemplate(_)
            | OidcLoginError::Database(_)
            | OidcLoginError::Internal(_) => ConnectResult::SsoFailed,
        }
    }
}

#[derive(Debug)]
pub struct CallbackFailure {
    pub purpose: CallbackPurpose,
    pub error: OidcLoginError,
}

#[derive(Debug, Error)]
pub enum SignInMethodError {
    #[error("{SIGN_IN_METHOD_NOT_CONNECTED_MESSAGE}")]
    NotConnected,
    #[error("{LAST_SIGN_IN_METHOD_MESSAGE}")]
    LastSignInMethod,
    #[error("{0}")]
    EmailDelivery(#[from] EmailDeliveryError),
    #[error("{0}")]
    EmailTemplate(#[from] EmailTemplateError),
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

impl SignInMethodError {
    pub fn error_code(&self) -> Option<ErrorCode> {
        match self {
            SignInMethodError::NotConnected => Some(ErrorCode::SignInMethodNotConnected),
            SignInMethodError::LastSignInMethod => Some(ErrorCode::LastSignInMethod),
            SignInMethodError::EmailDelivery(_)
            | SignInMethodError::EmailTemplate(_)
            | SignInMethodError::Database(_) => None,
        }
    }
}
