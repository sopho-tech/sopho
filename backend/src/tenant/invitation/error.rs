use crate::authorization::errors::AuthorizationError;
use crate::tenant::invitation::constants::{
    ALREADY_A_MEMBER_MESSAGE, ALREADY_INVITED_MESSAGE, INVALID_EMAIL_MESSAGE,
    INVITATION_EXPIRED_MESSAGE, INVITATION_NOT_FOUND_MESSAGE, SINGLE_SIGN_ON_REQUIRED_MESSAGE,
};
use crate::tenant::membership::constants::{ROLE_TOO_POWERFUL_MESSAGE, UNKNOWN_ROLE_MESSAGE};
use crate::tenant::membership::error::MembershipError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvitationError {
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error("{SINGLE_SIGN_ON_REQUIRED_MESSAGE}")]
    SingleSignOnRequired,
    #[error("{INVALID_EMAIL_MESSAGE}")]
    InvalidEmail,
    #[error("{UNKNOWN_ROLE_MESSAGE}")]
    UnknownRole,
    #[error("{ROLE_TOO_POWERFUL_MESSAGE}")]
    RoleTooPowerful,
    #[error("{ALREADY_A_MEMBER_MESSAGE}")]
    AlreadyMember,
    #[error("{ALREADY_INVITED_MESSAGE}")]
    AlreadyInvited,
    #[error("{INVITATION_NOT_FOUND_MESSAGE}")]
    NotFound,
    #[error("{INVITATION_EXPIRED_MESSAGE}")]
    Expired,
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    Internal(String),
}

impl From<MembershipError> for InvitationError {
    fn from(error: MembershipError) -> Self {
        match error {
            MembershipError::Unauthorized(authorization_error) => {
                InvitationError::Unauthorized(authorization_error)
            }
            MembershipError::UnknownRole => InvitationError::UnknownRole,
            MembershipError::RoleTooPowerful => InvitationError::RoleTooPowerful,
            MembershipError::Database(database_error) => InvitationError::Database(database_error),
            other => InvitationError::Internal(other.to_string()),
        }
    }
}
