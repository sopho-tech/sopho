use serde::Serialize;
use strum::{Display, EnumString, IntoStaticStr};

pub const INVITATION_EXPIRY_DAYS: i64 = 7;
pub const INVITATION_NOT_FOUND_MESSAGE: &str = "Invitation not found";
pub const INVALID_EMAIL_MESSAGE: &str = "Enter a valid email address";
pub const ALREADY_A_MEMBER_MESSAGE: &str = "This person is already a member of the workspace";
pub const ALREADY_INVITED_MESSAGE: &str = "This email already has a pending invitation";
pub const INVITATION_EXPIRED_MESSAGE: &str = "This invitation has expired";
pub const SINGLE_SIGN_ON_REQUIRED_MESSAGE: &str =
    "Invitees join through single sign-on. Configure an OIDC provider before inviting people";

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
