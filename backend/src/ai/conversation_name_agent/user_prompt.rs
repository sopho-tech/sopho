use indoc::indoc;

const SUGGEST_NAME: &str = indoc! {r#"
    Generate a concise title for this data analysis conversation.

    ### USER QUESTION
    {question}
"#};

pub enum UserPrompt {
    SuggestName { question: String },
}

impl UserPrompt {
    pub fn render(&self) -> String {
        match self {
            UserPrompt::SuggestName { question } => {
                SUGGEST_NAME.replace("{question}", question)
            }
        }
    }
}
