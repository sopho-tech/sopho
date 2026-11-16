use std::collections::HashMap;
use uuid::Uuid;

pub struct ConversationDescriptionForUsage {
    pub conversation_id: Uuid,
    pub name_when_readable: Option<String>,
}

pub enum QuestionForUsage {
    Text { text: String },
    Command,
}

pub struct ConversationQuestionsForUsage {
    pub is_readable: bool,
    pub questions_by_answer_message_id: HashMap<Uuid, QuestionForUsage>,
}
