use crate::authentication::service::PasswordFlowError;
use crate::authorization::errors::AuthorizationError;
use crate::common::error_codes::ErrorCode;
use crate::email_delivery::error::EmailDeliveryError;
use crate::platform::constants::{
    APPOINTEE_DEACTIVATED_MESSAGE, CANNOT_DEACTIVATE_YOURSELF_MESSAGE, INVALID_PAGE_MESSAGE,
    LAST_PLATFORM_ADMINISTRATOR_MESSAGE, PLATFORM_ACCOUNT_DEACTIVATED_MESSAGE,
    PLATFORM_TENANT_NOT_FOUND_MESSAGE, PLATFORM_USER_NOT_FOUND_MESSAGE,
};
use crate::tenant::error::TenantError;
use crate::tenant::membership::error::MembershipError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error("{PLATFORM_USER_NOT_FOUND_MESSAGE}")]
    UserNotFound,
    #[error("{PLATFORM_TENANT_NOT_FOUND_MESSAGE}")]
    TenantNotFound,
    #[error("{CANNOT_DEACTIVATE_YOURSELF_MESSAGE}")]
    CannotDeactivateYourself,
    #[error("{LAST_PLATFORM_ADMINISTRATOR_MESSAGE}")]
    LastPlatformAdministrator,
    #[error("{APPOINTEE_DEACTIVATED_MESSAGE}")]
    AppointeeDeactivated,
    #[error("{PLATFORM_ACCOUNT_DEACTIVATED_MESSAGE}")]
    AccountDeactivated,
    #[error("{INVALID_PAGE_MESSAGE}")]
    InvalidPage,
    #[error("{0}")]
    PasswordReset(PasswordFlowError),
    #[error("{0}")]
    EmailDelivery(EmailDeliveryError),
    #[error("{0}")]
    Membership(MembershipError),
    #[error("{0}")]
    Tenant(TenantError),
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Internal(String),
}

impl PlatformError {
    pub fn error_code(&self) -> Option<ErrorCode> {
        match self {
            PlatformError::UserNotFound => Some(ErrorCode::UserNotFound),
            PlatformError::TenantNotFound => Some(ErrorCode::TenantNotFound),
            PlatformError::CannotDeactivateYourself => Some(ErrorCode::CannotDeactivateYourself),
            PlatformError::LastPlatformAdministrator => Some(ErrorCode::LastPlatformAdministrator),
            PlatformError::AppointeeDeactivated => Some(ErrorCode::AppointeeDeactivated),
            PlatformError::AccountDeactivated => Some(ErrorCode::AccountDeactivated),
            PlatformError::PasswordReset(password_error) => password_error.error_code(),
            PlatformError::EmailDelivery(EmailDeliveryError::NotFound) => {
                Some(ErrorCode::EmailNotFound)
            }
            PlatformError::EmailDelivery(EmailDeliveryError::NotRetryable) => {
                Some(ErrorCode::EmailNotRetryable)
            }
            PlatformError::Unauthorized(_)
            | PlatformError::InvalidPage
            | PlatformError::EmailDelivery(_)
            | PlatformError::Membership(_)
            | PlatformError::Tenant(_)
            | PlatformError::Database(_)
            | PlatformError::Internal(_) => None,
        }
    }
}
