use serde::Serialize;
use strum::{Display, EnumString, IntoStaticStr};

pub const INVITATION_EXPIRY_DAYS: i64 = 7;
pub const HOURS_IN_HALF_A_DAY: i64 = 12;
pub const INVITATION_EXPIRED_LABEL: &str = "Expired";
pub const INVITATION_EXPIRES_TODAY_LABEL: &str = "Expires today";
pub const INVITATION_EXPIRES_TOMORROW_LABEL: &str = "Expires tomorrow";
pub const INVITATION_NOT_FOUND_MESSAGE: &str = "Invitation not found";
pub const INVALID_EMAIL_MESSAGE: &str = "Enter a valid email address";
pub const ALREADY_A_MEMBER_MESSAGE: &str = "This person is already a member of the tenant";
pub const ALREADY_INVITED_MESSAGE: &str = "This email already has a pending invitation";
pub const INVITATION_EXPIRED_MESSAGE: &str = "This invitation has expired";
pub const INVITATIONS_UNAVAILABLE_MESSAGE: &str = "Inviting people needs email or single sign-on. Configure SOPHO_SMTP_HOST or SOPHO_OIDC_PROVIDERS before inviting people";
pub const RESEND_NEEDS_EMAIL_MESSAGE: &str =
    "Resending an invitation needs email. Configure SOPHO_SMTP_HOST first";
pub const INVITATION_LINK_PATH: &str = "/invitation#token=";
pub const INVITATION_LINK_INVALID_MESSAGE: &str =
    "This invitation is no longer valid. Ask the person who invited you for a new one";
pub const INVITATION_FOR_ANOTHER_ACCOUNT_MESSAGE: &str =
    "This invitation is for another email address";
pub const ACCOUNT_EXISTS_MESSAGE: &str = "An account already uses this email. Sign in to join";
pub const FULL_NAME_REQUIRED_MESSAGE: &str = "Enter your full name";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Revoked,
    Expired,
}

impl InvitationStatus {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvitationDeliveryMethod {
    Email,
    SingleSignOn,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvitationLinkNextStep {
    SignUp,
    SignIn,
    Join,
    WrongAccount,
}
pub const SINGLE_MEMBER_LIMIT: u64 = 1;
pub const MEMBER_NOUN: &str = "member";
pub const MEMBERS_NOUN: &str = "members";
