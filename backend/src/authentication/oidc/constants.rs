use strum::{Display, IntoStaticStr};

pub const OIDC_STATE_COOKIE_NAME: &str = "OIDC_STATE";
pub const OIDC_COOKIE_PATH: &str = "/api/v1/auth/oidc";
pub const LOGIN_ATTEMPT_EXPIRY_MINUTES: i64 = 10;
pub const SIGN_IN_PATH: &str = "/signin";
pub const AFTER_SIGN_IN_PATH: &str = "/";
pub const SIGN_IN_ERROR_PARAMETER: &str = "error";
pub const EMAIL_SCOPE: &str = "email";
pub const PROFILE_SCOPE: &str = "profile";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum SignInErrorCode {
    SsoFailed,
    EmailNotVerified,
    SsoExpired,
    PasswordAccount,
}

impl SignInErrorCode {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}
