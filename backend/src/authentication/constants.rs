use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
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

    pub fn deserialize_from_str<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }

    pub fn serialize_to_str<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(value.as_str())
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
