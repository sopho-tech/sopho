use crate::authentication::oidc::constants::SignInErrorCode;
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
            OidcLoginError::UnknownProvider(_)
            | OidcLoginError::MissingProfile
            | OidcLoginError::Provider(_)
            | OidcLoginError::Database(_)
            | OidcLoginError::Internal(_) => SignInErrorCode::SsoFailed,
        }
    }
}
