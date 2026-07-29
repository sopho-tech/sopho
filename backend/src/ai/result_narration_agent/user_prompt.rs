use indoc::indoc;

const NARRATE_RESULT: &str = indoc! {r#"
    ### USER QUESTION
    {question}

    ### SQL THAT ANSWERED IT
    {sql}

    ### RESULT COLUMNS
    {columns}

    ### RESULT ROWS
    {rows}

    ### ROW COUNT
    {row_count_note}

    Answer the question.
"#};

pub enum UserPrompt<'a> {
    NarrateResult {
        question: &'a str,
        sql: &'a str,
        columns: &'a [serde_json::Value],
        rows: &'a [serde_json::Value],
        total_row_count: usize,
    },
}

impl<'a> UserPrompt<'a> {
    pub fn render(&self) -> String {
        match self {
            UserPrompt::NarrateResult {
                question,
                sql,
                columns,
                rows,
                total_row_count,
            } => {
                let columns_str =
                    serde_json::to_string_pretty(columns).unwrap_or_else(|_| "[]".to_string());
                let rows_str =
                    serde_json::to_string_pretty(rows).unwrap_or_else(|_| "[]".to_string());
                let row_count_note = if rows.len() < *total_row_count {
                    format!(
                        "Showing the first {} of {} rows.",
                        rows.len(),
                        total_row_count
                    )
                } else {
                    format!(
                        "{} rows in total. This is the complete result.",
                        total_row_count
                    )
                };
                NARRATE_RESULT
                    .replace("{question}", question)
                    .replace("{sql}", sql)
                    .replace("{columns}", &columns_str)
                    .replace("{rows}", &rows_str)
                    .replace("{row_count_note}", &row_count_note)
            }
        }
    }
}
