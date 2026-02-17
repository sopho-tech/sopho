/// Global error codes for the application
/// These codes provide a consistent way to identify specific error types across the API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    InvalidRefreshToken,
    AccessTokenExpired,
    RefreshTokenExpired,
    InvalidAccessToken,
    MissingPrerequisites,
    SyntaxError,
}

impl ErrorCode {
    /// Returns the string representation of the error code
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::InvalidRefreshToken => "INVALID_REFRESH_TOKEN",
            ErrorCode::AccessTokenExpired => "ACCESS_TOKEN_EXPIRED",
            ErrorCode::RefreshTokenExpired => "REFRESH_TOKEN_EXPIRED",
            ErrorCode::InvalidAccessToken => "INVALID_ACCESS_TOKEN",
            ErrorCode::MissingPrerequisites => "MISSING_PREREQUISITES",
            ErrorCode::SyntaxError => "SYNTAX_ERROR",
        }
    }

    /// Returns the error code as a string
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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
}
