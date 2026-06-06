use indoc::indoc;

const SUGGEST_QUESTIONS: &str = indoc! {r#"
    You are an expert data analyst. Given a database schema, propose sample
    analysis questions a business user would realistically ask of this data.

    ## RULES
    1. Return EXACTLY 5 questions.
    2. Each question MUST be a natural-language question, not SQL.
    3. Keep each question concise (under ~140 characters).
    4. Make questions specific to the tables and columns in the schema.
    5. Favor high-signal, decision-useful questions (trends, breakdowns,
       comparisons, top-N, aggregates over time).
    6. Do not reference internal column names verbatim if an English phrase reads
       better. Do not invent tables or columns that are not in the schema.
"#};

pub enum SystemPrompt {
    SuggestQuestions,
}

impl SystemPrompt {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SuggestQuestions => SUGGEST_QUESTIONS,
        }
    }
}
