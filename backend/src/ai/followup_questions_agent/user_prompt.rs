use indoc::indoc;

const SUGGEST_FOLLOWUPS: &str = indoc! {r#"
    Propose {count} follow-up analysis questions.

    ### LATEST QUESTION
    {question}

    ### SQL THAT ANSWERED IT
    {sql}

    ### PRIOR CONVERSATION
    {history}

    ### SCHEMA
    {schema}
"#};

pub enum UserPrompt<'a> {
    SuggestFollowups {
        count: usize,
        question: &'a str,
        sql: &'a str,
        history: &'a str,
        schema: &'a str,
    },
}

impl<'a> UserPrompt<'a> {
    pub fn render(&self) -> String {
        match self {
            UserPrompt::SuggestFollowups {
                count,
                question,
                sql,
                history,
                schema,
            } => SUGGEST_FOLLOWUPS
                .replace("{count}", &count.to_string())
                .replace("{question}", question)
                .replace("{sql}", sql)
                .replace("{history}", history)
                .replace("{schema}", schema),
        }
    }
}
