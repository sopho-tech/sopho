use serde::Serialize;
use strum::{Display, EnumString, IntoStaticStr};

pub const EMAIL_DATE_FORMAT: &str = "%-d %B %Y";
pub const EMAIL_TIME_FORMAT: &str = "%-d %B %Y, %H:%M UTC";
pub const PASSWORD_RESET_SUBJECT: &str = "Reset your Sopho password";
pub const NO_PASSWORD_SUBJECT: &str = "Your Sopho account has no password";
pub const PASSWORD_CHANGED_SUBJECT: &str = "Your Sopho password was changed";
pub const EMAIL_LOGO_CONTENT_ID: &str = "sopho-logo";
pub const EMAIL_LOGO_SOURCE: &str = "cid:sopho-logo";
pub const EMAIL_LOGO_CONTENT_TYPE: &str = "image/png";
pub const EMAIL_LOGO_DATA_URI_PREFIX: &str = "data:image/png;base64,";
pub const EMAIL_LOGO_PNG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/email/sopho_logo.png"
));
pub const SAMPLE_INVITER_NAME: &str = "Ada Admin";
pub const SAMPLE_RECIPIENT_NAME: &str = "Bea Builder";
pub const SAMPLE_TENANT_NAME: &str = "Acme Analytics";
pub const SAMPLE_ROLE_NAME: &str = "Member";
pub const SAMPLE_PROVIDER_DISPLAY_NAME: &str = "Google";
pub const SAMPLE_PROVIDER_NAMES: [&str; 2] = ["Google", "Okta"];
pub const SAMPLE_INVITATION_LINK: &str = "https://sopho.example.com/invitation#token=sample-token";
pub const SAMPLE_RESET_LINK: &str = "https://sopho.example.com/reset-password#token=sample-token";
pub const SAMPLE_SIGN_IN_LINK: &str = "https://sopho.example.com/signin";
pub const SAMPLE_FORGOT_PASSWORD_LINK: &str = "https://sopho.example.com/forgot-password";
pub const SAMPLE_INVITATION_EXPIRY_DAYS: i64 = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EmailKind {
    Invitation,
    PasswordReset,
    NoPassword,
    PasswordChanged,
    SignInMethodAdded,
    SignInMethodRemoved,
}

impl EmailKind {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            EmailKind::Invitation => "Invitation",
            EmailKind::PasswordReset => "Password reset",
            EmailKind::NoPassword => "No password",
            EmailKind::PasswordChanged => "Password changed",
            EmailKind::SignInMethodAdded => "Sign-in method added",
            EmailKind::SignInMethodRemoved => "Sign-in method removed",
        }
    }

    pub fn sent_when(&self) -> &'static str {
        match self {
            EmailKind::Invitation => "Someone is invited to a tenant by email",
            EmailKind::PasswordReset => {
                "Someone asks to reset the password of an account that has one"
            }
            EmailKind::NoPassword => {
                "Someone asks to reset the password of an account that only signs in with single sign-on"
            }
            EmailKind::PasswordChanged => "An account's password is changed or reset",
            EmailKind::SignInMethodAdded => "A single sign-on provider is connected to an account",
            EmailKind::SignInMethodRemoved => {
                "A single sign-on provider is disconnected from an account"
            }
        }
    }
}
