use serde::Serialize;
use strum::{Display, IntoStaticStr};

pub const OIDC_STATE_COOKIE_NAME: &str = "OIDC_STATE";
pub const OIDC_COOKIE_PATH: &str = "/api/v1/auth/oidc";
pub const LOGIN_ATTEMPT_EXPIRY_MINUTES: i64 = 10;
pub const SIGN_IN_PATH: &str = "/signin";
pub const AFTER_SIGN_IN_PATH: &str = "/";
pub const SIGN_IN_ERROR_PARAMETER: &str = "error";
pub const EMAIL_SCOPE: &str = "email";
pub const PROFILE_SCOPE: &str = "profile";
pub const PROFILE_PATH: &str = "/profile";
pub const GOOGLE_ISSUER_HOST: &str = "accounts.google.com";
pub const ONLY_SIGN_IN_METHOD_REASON: &str =
    "This is your only way to sign in. Connect another sign-in method before you disconnect it.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProviderIcon {
    Google,
    Generic,
}
pub const CONNECT_RESULT_PARAMETER: &str = "connect_result";
pub const CONNECTED_PROVIDER_PARAMETER: &str = "provider";
pub const SIGN_IN_METHOD_NOT_CONNECTED_MESSAGE: &str = "This sign-in method is not connected";
pub const LAST_SIGN_IN_METHOD_MESSAGE: &str =
    "This is your only way to sign in. Add another one before removing it";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum SignInErrorCode {
    SsoFailed,
    EmailNotVerified,
    SsoExpired,
    PasswordAccount,
    ContactAdministrator,
    AccountDeactivated,
    TooManyAttempts,
}

impl SignInErrorCode {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum ConnectResult {
    Connected,
    EmailMismatch,
    FederatedIdentityInUse,
    AlreadyConnected,
    EmailNotVerified,
    AccountDeactivated,
    SsoExpired,
    SsoFailed,
}

impl ConnectResult {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackPurpose {
    SignIn,
    Connect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignInMethodChange {
    Added,
    Removed,
}
