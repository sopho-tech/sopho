use crate::authorization::errors::AuthorizationError;
use crate::tenant::membership::constants::{
    LAST_OWNER_MESSAGE, MEMBER_NOT_FOUND_MESSAGE, MEMBER_TOO_POWERFUL_MESSAGE,
    ROLE_TOO_POWERFUL_MESSAGE, UNKNOWN_ROLE_MESSAGE,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MembershipError {
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error("{MEMBER_NOT_FOUND_MESSAGE}")]
    MemberNotFound,
    #[error("{UNKNOWN_ROLE_MESSAGE}")]
    UnknownRole,
    #[error("{ROLE_TOO_POWERFUL_MESSAGE}")]
    RoleTooPowerful,
    #[error("{MEMBER_TOO_POWERFUL_MESSAGE}")]
    MemberTooPowerful,
    #[error("{LAST_OWNER_MESSAGE}")]
    LastOwner,
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Internal(String),
}
