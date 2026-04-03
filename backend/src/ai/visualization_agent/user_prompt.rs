use indoc::indoc;

const RECOMMENDATION: &str = indoc! {r#"
    ### USER QUESTION
    {question}

    ### RESULT COLUMNS
    {columns}

    ### SAMPLE DATA (first rows)
    {sample_data}

    Recommend the best visualization.
"#};

pub enum UserPrompt {
    Recommendation {
        question: String,
        columns: Vec<serde_json::Value>,
        sample_data: Vec<serde_json::Value>,
    },
}

impl UserPrompt {
    pub fn render(&self) -> String {
        match self {
            UserPrompt::Recommendation {
                question,
                columns,
                sample_data,
            } => {
                let columns_str =
                    serde_json::to_string_pretty(columns).unwrap_or_else(|_| "[]".to_string());
                let sample_str =
                    serde_json::to_string_pretty(sample_data).unwrap_or_else(|_| "[]".to_string());
                RECOMMENDATION
                    .replace("{question}", question)
                    .replace("{columns}", &columns_str)
                    .replace("{sample_data}", &sample_str)
            }
        }
    }
}
