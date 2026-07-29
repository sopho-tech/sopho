use crate::ai::dto::ConversationHistory;

pub enum UserPrompt<'a> {
    Route {
        question: &'a str,
        conversation_history: &'a ConversationHistory,
    },
}

impl<'a> UserPrompt<'a> {
    pub fn render(&self) -> String {
        match self {
            UserPrompt::Route {
                question,
                conversation_history,
            } => {
                let history_json = if conversation_history.is_empty() {
                    "[]".to_string()
                } else {
                    serde_json::to_string(conversation_history).unwrap_or_else(|_| "[]".to_string())
                };
                format!(
                    "PRIOR_TURNS_OLDEST_FIRST: {history_json}\n\nCURRENT_USER_MESSAGE: {question}\n\nReturn the JSON object now."
                )
            }
        }
    }
}
