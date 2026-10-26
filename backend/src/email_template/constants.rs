use serde::Serialize;
use strum::{Display, EnumString, IntoStaticStr};

pub const EMAIL_DATE_FORMAT: &str = "%-d %B %Y";
pub const EMAIL_TIME_FORMAT: &str = "%-d %B %Y, %H:%M UTC";
pub const PASSWORD_RESET_SUBJECT: &str = "Reset your Sopho password";
pub const NO_PASSWORD_SUBJECT: &str = "Your Sopho account has no password";
pub const PASSWORD_CHANGED_SUBJECT: &str = "Your Sopho password was changed";

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
}
