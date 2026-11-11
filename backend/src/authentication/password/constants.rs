pub const MINIMUM_PASSWORD_CHARACTERS: usize = 15;
pub const MAXIMUM_PASSWORD_CHARACTERS: usize = 128;
pub const SERVICE_NAME_IN_PASSWORD: &str = "sopho";
pub const COMMON_PASSWORDS: &str = include_str!("common_passwords.txt");
pub const COMMON_PASSWORDS_HEADER_PREFIX: &str = "# ";
pub const PASSWORD_TOO_SHORT_MESSAGE: &str = "Use at least 15 characters";
pub const PASSWORD_TOO_LONG_MESSAGE: &str = "Use at most 128 characters";
pub const PASSWORD_TOO_COMMON_MESSAGE: &str = "This password is too common. Choose another one";
pub const PASSWORD_CONTAINS_SERVICE_NAME_MESSAGE: &str = "Choose a password without the word sopho";
pub const PASSWORD_CONTAINS_EMAIL_MESSAGE: &str = "Choose a password without your email address";
pub const RESET_LINK_PATH: &str = "/reset-password#token=";
pub const FORGOT_PASSWORD_PATH: &str = "/forgot-password";
pub const SIGN_IN_PAGE_PATH: &str = "/signin";
pub const RESET_LINK_EXPIRY_MINUTES: i64 = 60;
pub const RESET_EMAILS_PER_HOUR: u64 = 3;
pub const RESET_REQUESTED_MESSAGE: &str = "If an account uses this email, we've sent a link";
pub const PASSWORD_RESET_UNAVAILABLE_MESSAGE: &str =
    "Password reset needs email. Ask your Sopho administrator for help";
pub const TOO_MANY_RESET_EMAILS_MESSAGE: &str =
    "This account was sent 3 reset emails in the last hour. Try again later";
pub const INVALID_RESET_LINK_MESSAGE: &str =
    "This link has expired or was already used. Request a new one";
pub const WRONG_CURRENT_PASSWORD_MESSAGE: &str = "The current password is wrong";
pub const NO_PASSWORD_MESSAGE: &str = "This account signs in without a password";
