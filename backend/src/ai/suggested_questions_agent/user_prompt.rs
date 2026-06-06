use indoc::indoc;

const SUGGEST_QUESTIONS: &str = indoc! {r#"
    Propose 5 sample analysis questions for this connection.

    ### CONNECTION
    name: {name}
    type: {source_type}

    ### SCHEMA
    {schema}
"#};

pub enum UserPrompt<'a> {
    SuggestQuestions {
        name: &'a str,
        source_type: &'a str,
        schema: &'a str,
    },
}

impl<'a> UserPrompt<'a> {
    pub fn render(&self) -> String {
        match self {
            UserPrompt::SuggestQuestions {
                name,
                source_type,
                schema,
            } => SUGGEST_QUESTIONS
                .replace("{name}", name)
                .replace("{source_type}", source_type)
                .replace("{schema}", schema),
        }
    }
}
