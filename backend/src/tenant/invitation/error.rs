use crate::authentication::error::AccountCreationError;
use crate::authentication::service::{PasswordFlowError, PasswordRuleError};
use crate::authorization::errors::AuthorizationError;
use crate::email_delivery::error::EmailDeliveryError;
use crate::email_template::error::EmailTemplateError;
use crate::tenant::invitation::constants::{
    ACCOUNT_EXISTS_MESSAGE, FULL_NAME_REQUIRED_MESSAGE, INVITATION_FOR_ANOTHER_ACCOUNT_MESSAGE,
    INVITATION_LINK_INVALID_MESSAGE,
};
use crate::tenant::invitation::constants::{
    ALREADY_A_MEMBER_MESSAGE, ALREADY_INVITED_MESSAGE, INVALID_EMAIL_MESSAGE,
    INVITATION_EXPIRED_MESSAGE, INVITATION_NOT_FOUND_MESSAGE, INVITATIONS_UNAVAILABLE_MESSAGE,
    MEMBER_NOUN, MEMBERS_NOUN, RESEND_NEEDS_EMAIL_MESSAGE, SINGLE_MEMBER_LIMIT,
};
use crate::tenant::membership::constants::{ROLE_TOO_POWERFUL_MESSAGE, UNKNOWN_ROLE_MESSAGE};
use crate::tenant::membership::error::MembershipError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvitationError {
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error("{INVITATIONS_UNAVAILABLE_MESSAGE}")]
    InvitationsUnavailable,
    #[error("{RESEND_NEEDS_EMAIL_MESSAGE}")]
    ResendNeedsEmail,
    #[error("{INVITATION_LINK_INVALID_MESSAGE}")]
    LinkInvalid,
    #[error("{INVITATION_FOR_ANOTHER_ACCOUNT_MESSAGE}")]
    InvitationForAnotherAccount,
    #[error("{ACCOUNT_EXISTS_MESSAGE}")]
    AccountExists,
    #[error("{FULL_NAME_REQUIRED_MESSAGE}")]
    FullNameRequired,
    #[error("{0}")]
    PasswordSignInOff(PasswordFlowError),
    #[error("{0}")]
    PasswordRule(#[from] PasswordRuleError),
    #[error("{0}")]
    AccountCreation(#[from] AccountCreationError),
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
    #[error(
        "This tenant has reached its limit of {limit} {}",
        match *.limit {
            SINGLE_MEMBER_LIMIT => MEMBER_NOUN,
            _ => MEMBERS_NOUN,
        }
    )]
    MemberLimitReached { limit: u64 },
    #[error("{INVITATION_NOT_FOUND_MESSAGE}")]
    NotFound,
    #[error("{INVITATION_EXPIRED_MESSAGE}")]
    Expired,
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    EmailDelivery(EmailDeliveryError),
    #[error("{0}")]
    EmailTemplate(#[from] EmailTemplateError),
    #[error("{0}")]
    Internal(String),
}

impl From<EmailDeliveryError> for InvitationError {
    fn from(error: EmailDeliveryError) -> Self {
        match error {
            EmailDeliveryError::InvalidRecipient { .. } => InvitationError::InvalidEmail,
            other => InvitationError::EmailDelivery(other),
        }
    }
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
