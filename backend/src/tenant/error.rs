use crate::authorization::errors::AuthorizationError;
use crate::tenant::constants::WORKSPACE_NAME_REQUIRED_MESSAGE;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TenantError {
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error("{WORKSPACE_NAME_REQUIRED_MESSAGE}")]
    NameRequired,
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
}
