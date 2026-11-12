use strum::{Display, IntoStaticStr};

/// Global error codes for the application
/// These codes provide a consistent way to identify specific error types across the API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidRefreshToken,
    AccessTokenExpired,
    RefreshTokenExpired,
    InvalidAccessToken,
    MissingPrerequisites,
    SyntaxError,
    AiNotLive,
    ConversationBusy,
    UserMessageLimitReached,
    InvitationLinkInvalid,
    InvitationForAnotherAccount,
    AccountExists,
    FullNameRequired,
    PasswordTooShort,
    PasswordTooLong,
    PasswordTooCommon,
    PasswordContainsServiceName,
    PasswordContainsEmail,
    PasswordResetUnavailable,
    TooManyResetEmails,
    InvalidResetLink,
    WrongCurrentPassword,
    NoPassword,
    TooManySignInAttempts,
    AccountDeactivated,
    SignInMethodNotConnected,
    LastSignInMethod,
    CannotDeactivateYourself,
    CannotRemoveYourOwnAdministratorRole,
    LastPlatformAdministrator,
    AppointeeDeactivated,
    EmailNotRetryable,
    UserNotFound,
    TenantNotFound,
    EmailNotFound,
    TenantCreationLimitReached,
    PasswordSignInOff,
    ConnectionTypeDisabled,
    ConnectionHostNotAllowed,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

/// Convenience constants for easy access to error codes
pub mod codes {
    use super::ErrorCode;

    pub const INVALID_REFRESH_TOKEN: ErrorCode = ErrorCode::InvalidRefreshToken;
    pub const ACCESS_TOKEN_EXPIRED: ErrorCode = ErrorCode::AccessTokenExpired;
    pub const REFRESH_TOKEN_EXPIRED: ErrorCode = ErrorCode::RefreshTokenExpired;
    pub const INVALID_ACCESS_TOKEN: ErrorCode = ErrorCode::InvalidAccessToken;
    pub const MISSING_PREREQUISITES: ErrorCode = ErrorCode::MissingPrerequisites;
    pub const SYNTAX_ERROR: ErrorCode = ErrorCode::SyntaxError;
    pub const AI_NOT_LIVE: ErrorCode = ErrorCode::AiNotLive;
    pub const CONVERSATION_BUSY: ErrorCode = ErrorCode::ConversationBusy;
    pub const USER_MESSAGE_LIMIT_REACHED: ErrorCode = ErrorCode::UserMessageLimitReached;
}
