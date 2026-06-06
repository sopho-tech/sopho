use crate::entity;
use uuid::Uuid;

#[derive(Debug, serde::Serialize)]
pub struct SuggestedQuestionDto {
    pub id: Uuid,
    pub question_text: String,
    pub generated_at: String,
}

impl From<entity::suggested_question::Model> for SuggestedQuestionDto {
    fn from(m: entity::suggested_question::Model) -> Self {
        Self {
            id: m.id,
            question_text: m.question_text,
            generated_at: m.generated_at.to_rfc3339(),
        }
    }
}
