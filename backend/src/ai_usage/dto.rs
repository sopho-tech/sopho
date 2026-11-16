use crate::ai_usage::constants::{
    AllowanceUsageLevel, ConversationNameKind, KeySource, QuestionTextKind, SortDirection,
    UsageSortColumn,
};
use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UsageRangeQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub last_day_count: Option<i64>,
    pub key_source: Option<KeySource>,
}

#[derive(Debug, Deserialize)]
pub struct UsageListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub search: Option<String>,
    pub sort_by: Option<UsageSortColumn>,
    pub sort_direction: Option<SortDirection>,
}

#[derive(Debug, Serialize)]
pub struct UsageSummaryDto {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_input_tokens: i64,
    pub total_tokens: i64,
    pub call_count: i64,
    pub has_usage: bool,
    pub is_platform_ai_on: bool,
    pub missing_token_counts_note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UsageDayDto {
    pub day: NaiveDate,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, Serialize)]
pub struct UsageOverTimeDto {
    pub days: Vec<UsageDayDto>,
}

#[derive(Debug, Serialize)]
pub struct UsageUserDto {
    pub user_id: Option<Uuid>,
    pub name: String,
    pub total_tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, Serialize)]
pub struct UsageKindDto {
    pub kind: &'static str,
    pub label: &'static str,
    pub total_tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, Serialize)]
pub struct UsageByKindDto {
    pub kinds: Vec<UsageKindDto>,
}

#[derive(Debug, Serialize)]
pub struct UsageConversationDto {
    pub conversation_id: Uuid,
    pub name: String,
    pub name_kind: ConversationNameKind,
    pub user_name: String,
    pub total_tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, Serialize)]
pub struct UsageQuestionDto {
    pub conversation_message_id: Option<Uuid>,
    pub text: String,
    pub text_kind: QuestionTextKind,
    pub first_call_at: DateTime<FixedOffset>,
    pub total_tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, Serialize)]
pub struct UsageConversationQuestionsDto {
    pub conversation_id: Uuid,
    pub name: String,
    pub name_kind: ConversationNameKind,
    pub total_tokens: i64,
    pub call_count: i64,
    pub question_count: i64,
    pub questions: Vec<UsageQuestionDto>,
}

#[derive(Debug, Serialize)]
pub struct PlatformUsageTenantDto {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub total_tokens: i64,
    pub call_count: i64,
    pub allowance_usage: Option<AllowanceUsageDto>,
}

#[derive(Debug, Serialize)]
pub struct AllowanceUsageDto {
    pub tokens_label: String,
    pub used_percentage: u64,
    pub level: AllowanceUsageLevel,
}

#[derive(Debug, Serialize)]
pub struct PlatformUsageUserDto {
    pub user_id: Option<Uuid>,
    pub name: String,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub total_tokens: i64,
    pub call_count: i64,
}

#[derive(Debug, Serialize)]
pub struct PlatformUsageConversationDto {
    pub conversation_id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub name: String,
    pub name_kind: ConversationNameKind,
    pub user_name: String,
    pub total_tokens: i64,
    pub call_count: i64,
}
