use crate::ai_usage::constants::{ConversationNameKind, KeySource, QuestionTextKind};
use crate::authorization::dto::{TenantId, UserId};
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiUsageMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_message_id: Option<Uuid>,
}

#[derive(Clone, Debug)]
pub struct AiCallContext {
    tenant_id: TenantId,
    user_id: Option<UserId>,
    metadata: AiUsageMetadata,
}

impl AiCallContext {
    pub fn for_user(tenant_id: TenantId, user_id: UserId) -> Self {
        Self {
            tenant_id,
            user_id: Some(user_id),
            metadata: AiUsageMetadata::default(),
        }
    }

    pub fn for_background_work(tenant_id: TenantId) -> Self {
        Self {
            tenant_id,
            user_id: None,
            metadata: AiUsageMetadata::default(),
        }
    }

    pub fn in_conversation(mut self, conversation_id: Uuid) -> Self {
        self.metadata = AiUsageMetadata {
            conversation_id: Some(conversation_id),
            conversation_message_id: None,
        };
        self
    }

    pub fn answering_message(
        mut self,
        conversation_id: Uuid,
        conversation_message_id: Uuid,
    ) -> Self {
        self.metadata = AiUsageMetadata {
            conversation_id: Some(conversation_id),
            conversation_message_id: Some(conversation_message_id),
        };
        self
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    pub fn user_id(&self) -> Option<UserId> {
        self.user_id
    }

    pub fn metadata(&self) -> &AiUsageMetadata {
        &self.metadata
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_input_tokens: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct ModelCallRecord {
    pub agent_name: &'static str,
    pub provider: &'static str,
    pub model: &'static str,
    pub key_source: KeySource,
    pub token_usage: TokenUsage,
}

#[derive(Debug, Deserialize)]
pub struct UsageRangeQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub last_day_count: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UsageRange {
    pub first_day: NaiveDate,
    pub last_day: NaiveDate,
    pub starts_at: DateTime<Utc>,
    pub ends_before: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationPageQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
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
pub struct UsageByUserDto {
    pub users: Vec<UsageUserDto>,
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
