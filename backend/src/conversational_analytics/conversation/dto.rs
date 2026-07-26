use super::constants::ConversationStatus;
use crate::conversational_analytics::message::dto::ConversationMessageWithContentDto;
use crate::conversational_analytics::segment::MessageSegment;
use crate::entity;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConversationDto {
    pub connection_id: Uuid,
    pub segments: Vec<MessageSegment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationDto {
    pub id: Uuid,
    pub connection_id: Uuid,
    pub name: String,
    #[serde(
        deserialize_with = "ConversationStatus::deserialize_from_str",
        serialize_with = "ConversationStatus::serialize_to_str"
    )]
    pub status: ConversationStatus,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl TryFrom<entity::conversation::Model> for ConversationDto {
    type Error = String;

    fn try_from(model: entity::conversation::Model) -> Result<Self, Self::Error> {
        let status = model.status.parse()?;
        Ok(ConversationDto {
            id: model.id,
            connection_id: model.connection_id,
            name: model.name,
            status,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListConversationsQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationListItemDto {
    #[serde(flatten)]
    pub conversation: ConversationDto,
    pub user_message_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedConversationsDto {
    pub items: Vec<ConversationListItemDto>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkDeleteConversationsDto {
    pub conversation_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppendUserMessageDto {
    pub segments: Vec<MessageSegment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationWithMessagesDto {
    pub conversation: ConversationDto,
    pub messages: Vec<ConversationMessageWithContentDto>,
    pub should_execute_completion: bool,
}
