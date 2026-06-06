use indoc::indoc;

const SUGGEST_FOLLOWUPS: &str = indoc! {r#"
    You are an expert data analyst. You are given the database schema, the most
    recent question a user asked, the SQL that answered it, and the prior
    conversation. Propose natural-language follow-up questions the user would
    realistically ask next to dig deeper into the same data.

    ## RULES
    1. Return EXACTLY {count} questions.
    2. Each question MUST be a natural-language question, not SQL.
    3. Keep each question concise (under ~140 characters).
    4. Each question MUST be answerable from the tables and columns in the schema.
    5. Build on the latest question and result: drill-downs, breakdowns,
       comparisons, trends over time, or related top-N analyses.
    6. Do not repeat questions already asked in the conversation. Do not invent
       tables or columns that are not in the schema.
"#};

pub enum SystemPrompt {
    SuggestFollowups,
}

impl SystemPrompt {
    pub fn render(&self, count: usize) -> String {
        match self {
            Self::SuggestFollowups => SUGGEST_FOLLOWUPS.replace("{count}", &count.to_string()),
        }
    }
}
