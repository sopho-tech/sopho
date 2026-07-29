use crate::ai::dto::ConversationHistory;

pub enum UserPrompt<'a> {
    Generate {
        conversation_history: &'a ConversationHistory,
    },
}

impl<'a> UserPrompt<'a> {
    pub fn render(&self) -> String {
        match self {
            UserPrompt::Generate {
                conversation_history,
            } => {
                let mut out = String::from(
                    "Design a Canvas from the conversation so far.\n\nConversation history:\n",
                );
                for (i, turn) in conversation_history.into_iter().enumerate() {
                    out.push_str(&format!("\nTurn {}:\n", i + 1));
                    out.push_str(&format!("  User: {}\n", turn.user_question));
                    if let Some(sql) = &turn.generated_sql {
                        out.push_str(&format!("  SQL: {}\n", sql));
                    }
                }
                out
            }
        }
    }
}
