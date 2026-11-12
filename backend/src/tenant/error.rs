use crate::authorization::errors::AuthorizationError;
use crate::tenant::constants::{TENANT_NAME_REQUIRED_MESSAGE, TENANT_NAME_TOO_LONG_MESSAGE};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TenantError {
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error("You have created the maximum of {limit} tenants")]
    SelfServeLimitReached { limit: u64 },
    #[error("{TENANT_NAME_REQUIRED_MESSAGE}")]
    NameRequired,
    #[error("{TENANT_NAME_TOO_LONG_MESSAGE}")]
    NameTooLong,
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
}
