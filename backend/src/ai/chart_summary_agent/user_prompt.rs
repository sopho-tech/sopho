use crate::ai::dto::ChartSummaryInput;
use indoc::indoc;

const SUMMARIZE_CHART: &str = indoc! {r#"
    ### CHART TITLE
    {chart_name}

    ### CHART TYPE
    {chart_type}

    ### PLOTTED FIELDS
    {field_names}

    ### RESULT COLUMNS
    {columns}

    ### RESULT ROWS
    {rows}

    ### ROW COUNT
    {row_count_note}
    {user_guidance}
    Describe what this chart shows.
"#};

const USER_GUIDANCE: &str = indoc! {r#"

    ### USER GUIDANCE
    {user_prompt}
"#};

pub fn render_user_guidance(user_prompt: &Option<String>) -> String {
    match user_prompt {
        Some(prompt) => USER_GUIDANCE.replace("{user_prompt}", prompt),
        None => String::new(),
    }
}

pub enum UserPrompt<'a> {
    SummarizeChart {
        chart: &'a ChartSummaryInput,
        user_prompt: &'a Option<String>,
    },
}

impl UserPrompt<'_> {
    pub fn render(&self) -> String {
        match self {
            UserPrompt::SummarizeChart { chart, user_prompt } => {
                let field_names = if chart.field_names.is_empty() {
                    "not applicable".to_string()
                } else {
                    chart.field_names.join(", ")
                };
                SUMMARIZE_CHART
                    .replace("{chart_name}", &chart.chart_name)
                    .replace("{chart_type}", &chart.chart_type)
                    .replace("{field_names}", &field_names)
                    .replace("{columns}", &chart.columns_json())
                    .replace("{rows}", &chart.rows_json())
                    .replace("{row_count_note}", &chart.row_count_note())
                    .replace("{user_guidance}", &render_user_guidance(user_prompt))
            }
        }
    }
}
