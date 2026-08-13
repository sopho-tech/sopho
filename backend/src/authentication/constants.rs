use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CookieName {
    AccessToken,
    RefreshToken,
}

impl CookieName {
    fn as_str(&self) -> &'static str {
        match self {
            CookieName::AccessToken => "ACCESS_TOKEN",
            CookieName::RefreshToken => "REFRESH_TOKEN",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ACCESS_TOKEN" => Ok(CookieName::AccessToken),
            "REFRESH_TOKEN" => Ok(CookieName::RefreshToken),
            _ => Err(format!("Invalid cookie type: {}", s)),
        }
    }
}

impl fmt::Display for CookieName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<CookieName> for Cow<'static, str> {
    fn from(name: CookieName) -> Self {
        Cow::Borrowed(name.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Inactive,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionStatus::Active => "active",
            SessionStatus::Inactive => "inactive",
        }
    }
}

impl From<SessionStatus> for String {
    fn from(status: SessionStatus) -> Self {
        status.as_str().to_string()
    }
}

pub const ACCESS_TOKEN_EXPIRY_HOURS: i64 = 1;

pub const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 15;
