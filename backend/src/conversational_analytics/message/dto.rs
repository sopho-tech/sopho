use crate::conversational_analytics::message::constants::Sender;
use crate::conversational_analytics::{
    conversation::constants::MessageStatus, message_content::dto::ConversationMessageContentDto,
};
use crate::entity;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConversationMessageDto {
    pub conversation_id: Uuid,
    pub sequence_number: i32,
    pub sender: String,
    pub content: String,
    pub status: String,
}

impl CreateConversationMessageDto {
    pub fn initial_user_message(conversation_id: Uuid, user_message: &str) -> Self {
        Self {
            conversation_id,
            sequence_number: 1,
            sender: Sender::Human.to_string(),
            content: user_message.to_string(),
            status: MessageStatus::Processed.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationMessageDto {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sequence_number: i32,
    pub sender: String,
    pub status: String,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationMessageWithContentDto {
    #[serde(flatten)]
    pub message: ConversationMessageDto,
    pub content: Vec<ConversationMessageContentDto>,
}

impl ConversationMessageWithContentDto {
    pub fn new(
        message: ConversationMessageDto,
        content: Vec<ConversationMessageContentDto>,
    ) -> Self {
        Self { message, content }
    }
}

impl From<entity::conversation_message::Model> for ConversationMessageDto {
    fn from(model: entity::conversation_message::Model) -> Self {
        ConversationMessageDto {
            id: model.id,
            conversation_id: model.conversation_id,
            sequence_number: model.sequence_number,
            sender: model.sender,
            status: model.status,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
