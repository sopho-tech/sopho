use crate::authorization::errors::AuthorizationError;
use crate::tenant::constants::{
    MAXIMUM_TENANT_NAME_CHARACTERS, SINGLE_TENANT_LIMIT, TENANT_NAME_REQUIRED_MESSAGE, TENANT_NOUN,
    TENANTS_NOUN,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TenantError {
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error(
        "You have created the maximum of {limit} {}",
        match *.limit {
            SINGLE_TENANT_LIMIT => TENANT_NOUN,
            _ => TENANTS_NOUN,
        }
    )]
    SelfServeLimitReached { limit: u64 },
    #[error("{TENANT_NAME_REQUIRED_MESSAGE}")]
    NameRequired,
    #[error("Use at most {MAXIMUM_TENANT_NAME_CHARACTERS} characters for the tenant name")]
    NameTooLong,
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
}
