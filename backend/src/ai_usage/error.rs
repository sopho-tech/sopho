use crate::authorization::errors::AuthorizationError;
use crate::conversational_analytics::conversation::error::ConversationError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiUsageError {
    #[error("AI usage could not be stored: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("AI usage metadata could not be serialised: {0}")]
    Metadata(#[from] serde_json::Error),
    #[error("the {token_kind} token count {token_count} is too large to store")]
    TokenCountOutOfRange {
        token_kind: &'static str,
        token_count: u64,
    },
    #[error("the platform AI key was used for a call that reported no token counts")]
    PlatformCallWithoutTokenCounts,
    #[error("{0}")]
    Unauthorized(#[from] AuthorizationError),
    #[error("The date range starts after it ends")]
    RangeReversed,
    #[error("The date range may not be longer than {maximum_day_count} days")]
    RangeTooLong { maximum_day_count: i64 },
    #[error("Give either a number of days or a first day, not both")]
    RangeAmbiguous,
    #[error("The number of days must be at least 1")]
    RangeWithoutDays,
    #[error("The page is out of range")]
    PageOutOfRange,
    #[error("The date range is outside the supported calendar")]
    RangeOutOfCalendar,
    #[error("AI usage was recorded for an unknown agent: {agent_name}")]
    UnknownAgent { agent_name: String },
    #[error("AI usage holds an unreadable value: {0}")]
    UnreadableStoredValue(String),
    #[error("{0}")]
    Conversation(#[from] ConversationError),
}
