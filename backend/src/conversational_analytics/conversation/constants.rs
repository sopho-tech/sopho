use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

pub const CHANNEL_SIZE: usize = 32;
pub const MAX_QUESTION_LENGTH: usize = 16_384;
pub const DEFAULT_CONVERSATION_NAME: &str = "Untitled";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageStatus {
    Processing,
    Processed,
    Failed,
}

impl fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageStatus::Processing => write!(f, "PROCESSING"),
            MessageStatus::Processed => write!(f, "PROCESSED"),
            MessageStatus::Failed => write!(f, "FAILED"),
        }
    }
}

impl FromStr for MessageStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PROCESSING" => Ok(MessageStatus::Processing),
            "PROCESSED" => Ok(MessageStatus::Processed),
            "FAILED" => Ok(MessageStatus::Failed),
            _ => Err(format!("Invalid message status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConversationStatus {
    Active,
    Inactive,
}

impl fmt::Display for ConversationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversationStatus::Active => write!(f, "ACTIVE"),
            ConversationStatus::Inactive => write!(f, "INACTIVE"),
        }
    }
}

impl FromStr for ConversationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACTIVE" => Ok(ConversationStatus::Active),
            "INACTIVE" => Ok(ConversationStatus::Inactive),
            _ => Err(format!("Invalid conversation status: {}", s)),
        }
    }
}

impl ConversationStatus {
    pub fn deserialize_from_str<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }

    pub fn serialize_to_str<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContentType {
    Text,
    DataAnalysisResponse,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentType::Text => write!(f, "TEXT"),
            ContentType::DataAnalysisResponse => write!(f, "DATA_ANALYSIS_RESPONSE"),
        }
    }
}

impl FromStr for ContentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TEXT" => Ok(ContentType::Text),
            "DATA_ANALYSIS_RESPONSE" => Ok(ContentType::DataAnalysisResponse),
            _ => Err(format!("Invalid content type: {}", s)),
        }
    }
}

impl ContentType {
    pub fn deserialize_from_str<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }

    pub fn serialize_to_str<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }
}
