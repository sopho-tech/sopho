use crate::entity;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConversationMessageContentDto {
    pub conversation_message_id: Uuid,
    pub sequence_number: i32,
    pub content_type: String,
    pub content: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationMessageContentDto {
    pub id: Uuid,
    pub conversation_message_id: Uuid,
    pub sequence_number: i32,
    pub content_type: String,
    pub content: String,
    pub status: String,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<entity::conversation_message_content::Model> for ConversationMessageContentDto {
    fn from(model: entity::conversation_message_content::Model) -> Self {
        ConversationMessageContentDto {
            id: model.id,
            conversation_message_id: model.conversation_message_id,
            sequence_number: model.sequence_number,
            content_type: model.content_type,
            content: model.content,
            status: model.status,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
