use strum::{Display, EnumString, IntoStaticStr};

pub const INPUT_TOKEN_KIND: &str = "input";
pub const OUTPUT_TOKEN_KIND: &str = "output";
pub const CACHED_INPUT_TOKEN_KIND: &str = "cached input";
pub const TOTAL_TOKEN_KIND: &str = "total";

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Display,
    EnumString,
    IntoStaticStr,
    serde::Serialize,
    serde::Deserialize,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KeySource {
    Tenant,
    Platform,
}

pub const DEFAULT_RANGE_DAY_COUNT: i64 = 30;
pub const MAXIMUM_RANGE_DAY_COUNT: i64 = 366;
pub const DEFAULT_CONVERSATION_PAGE_SIZE: u64 = 10;
pub const MAXIMUM_CONVERSATION_PAGE_SIZE: u64 = 50;

pub const BACKGROUND_WORK_LABEL: &str = "Background";
pub const DELETED_ACCOUNT_LABEL: &str = "Deleted account";
pub const PRIVATE_CONVERSATION_LABEL: &str = "Private conversation";
pub const DELETED_CONVERSATION_LABEL: &str = "Deleted conversation";
pub const PRIVATE_QUESTION_LABEL: &str = "Question in a private conversation";
pub const DELETED_QUESTION_LABEL: &str = "Question in a deleted conversation";
pub const CANVAS_COMMAND_LABEL: &str = "Canvas command";
pub const NO_QUESTION_LABEL: &str = "Conversation naming and other model calls outside a question";
pub const USAGE_NOT_FOUND_MESSAGE: &str = "AI usage not found";
pub const ONE_CALL_WITHOUT_TOKEN_COUNTS_NOTE: &str = "1 model call in this range reported no token counts. It is included in the number of model calls but adds no tokens.";

pub const SQLITE_UTC_DAY_EXPRESSION: &str = "strftime('%Y-%m-%d', created_at)";
pub const POSTGRES_UTC_DAY_EXPRESSION: &str =
    "to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD')";
pub const UTC_DAY_FORMAT: &str = "%Y-%m-%d";
pub const CONVERSATION_ID_EXPRESSION: &str = "metadata ->> 'conversation_id'";
pub const CONVERSATION_MESSAGE_ID_EXPRESSION: &str = "metadata ->> 'conversation_message_id'";
pub const BIG_INTEGER_TYPE: &str = "bigint";

pub const INPUT_TOKENS_ALIAS: &str = "input_tokens";
pub const OUTPUT_TOKENS_ALIAS: &str = "output_tokens";
pub const CACHED_INPUT_TOKENS_ALIAS: &str = "cached_input_tokens";
pub const CALL_COUNT_ALIAS: &str = "call_count";
pub const DAY_ALIAS: &str = "day";
pub const USER_ID_ALIAS: &str = "user_id";
pub const AGENT_NAME_ALIAS: &str = "agent_name";
pub const CONVERSATION_ID_ALIAS: &str = "conversation_id";
pub const CONVERSATION_MESSAGE_ID_ALIAS: &str = "conversation_message_id";
pub const TENANT_ID_ALIAS: &str = "tenant_id";
pub const FIRST_CALL_AT_ALIAS: &str = "first_call_at";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoStaticStr)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AiUsageKind {
    AnsweringQuestions,
    FollowupQuestions,
    CanvasGeneration,
    Summaries,
    SuggestedQuestions,
    ConversationNaming,
    AiKeyTest,
}

impl AiUsageKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::AnsweringQuestions => "Answering questions",
            Self::FollowupQuestions => "Follow-up questions",
            Self::CanvasGeneration => "Canvas generation",
            Self::Summaries => "Summaries",
            Self::SuggestedQuestions => "Suggested questions",
            Self::ConversationNaming => "Conversation naming",
            Self::AiKeyTest => "AI key test",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoStaticStr, serde::Serialize)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConversationNameKind {
    Name,
    Private,
    Deleted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoStaticStr, serde::Serialize)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuestionTextKind {
    Text,
    Command,
    Private,
    Deleted,
    NoQuestion,
}
pub const TOKENS_OF_ALLOWANCE_SEPARATOR: &str = " of ";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AllowanceUsageLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageSortColumn {
    Name,
    TenantName,
    UserName,
    TotalTokens,
    CallCount,
    AllowanceUsage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

pub const DEFAULT_USAGE_SORT_COLUMN: UsageSortColumn = UsageSortColumn::TotalTokens;
pub const DEFAULT_SORT_DIRECTION: SortDirection = SortDirection::Descending;
pub const NO_ALLOWANCE_USAGE_SORT_NUMBER: i64 = -1;

pub const WHOLE_ALLOWANCE_PERCENTAGE: u64 = 100;
pub const MEDIUM_ALLOWANCE_USAGE_FROM_PERCENTAGE: u64 = 70;
pub const HIGH_ALLOWANCE_USAGE_FROM_PERCENTAGE: u64 = 90;
