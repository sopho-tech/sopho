use crate::ai::agent_utils::ModelClient;
use crate::ai_usage::constants::{
    ConversationNameKind, KeySource, QuestionTextKind, SortDirection, UsageSortColumn,
};
use crate::authorization::dto::{TenantId, UserId};
use chrono::{DateTime, NaiveDate, Utc};
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

#[derive(Clone)]
pub struct AiAccess {
    pub model_client: ModelClient,
    pub key_source: KeySource,
}

impl std::fmt::Debug for AiAccess {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AiAccess")
            .field("key_source", &self.key_source)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
pub struct AiCallContext {
    tenant_id: TenantId,
    user_id: Option<UserId>,
    ai_access: AiAccess,
    metadata: AiUsageMetadata,
}

impl AiCallContext {
    pub fn for_user(tenant_id: TenantId, user_id: UserId, ai_access: AiAccess) -> Self {
        Self {
            tenant_id,
            user_id: Some(user_id),
            ai_access,
            metadata: AiUsageMetadata::default(),
        }
    }

    pub fn for_background_work(tenant_id: TenantId, ai_access: AiAccess) -> Self {
        Self {
            tenant_id,
            user_id: None,
            ai_access,
            metadata: AiUsageMetadata::default(),
        }
    }

    pub fn model_client(&self) -> &ModelClient {
        &self.ai_access.model_client
    }

    pub fn key_source(&self) -> KeySource {
        self.ai_access.key_source
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
    pub token_usage: TokenUsage,
}

#[derive(Clone, Copy, Debug)]
pub enum UsageScope<'a> {
    Tenant { tenant_id: &'a TenantId },
    AllTenants,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UsageRange {
    pub key_source: Option<KeySource>,
    pub first_day: NaiveDate,
    pub last_day: NaiveDate,
    pub starts_at: DateTime<Utc>,
    pub ends_before: DateTime<Utc>,
}

pub(super) struct UsageOrdering {
    pub(super) lowercase_search_text: Option<String>,
    pub(super) sort_column: UsageSortColumn,
    pub(super) sort_direction: SortDirection,
}

pub(super) struct UsageListing {
    pub(super) usage_page: UsagePage,
    pub(super) usage_ordering: UsageOrdering,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum UsageSortValue {
    Text { lowercase_text: String },
    Number { number: i64 },
}

pub(super) trait ListedUsageRow {
    const SORT_COLUMNS: &'static [UsageSortColumn];

    fn searchable_texts(&self) -> Vec<&str>;
    fn sort_value(&self, sort_column: UsageSortColumn) -> Option<UsageSortValue>;
    fn total_tokens(&self) -> i64;
    fn stable_row_id(&self) -> String;
}

pub(super) struct UsagePage {
    pub(super) page: u64,
    pub(super) page_size: u64,
    pub(super) first_row_offset: u64,
}

pub(super) struct LargestUser {
    pub(super) total_tokens: i64,
    pub(super) user_id: Option<Uuid>,
}

impl LargestUser {
    pub(super) fn is_larger_than(&self, other_user: &LargestUser) -> bool {
        (self.total_tokens, other_user.user_id) > (other_user.total_tokens, self.user_id)
    }
}

pub(super) struct ConversationName {
    pub(super) name: String,
    pub(super) name_kind: ConversationNameKind,
}

pub(super) struct QuestionText {
    pub(super) text: String,
    pub(super) text_kind: QuestionTextKind,
}
