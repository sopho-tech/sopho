use crate::ai::dto::{ChartSummaryInput, DashboardSummaryInput};
use indoc::indoc;

const SUMMARIZE_DASHBOARD: &str = indoc! {r#"
    ### CHARTS ON THIS DASHBOARD
    {charts}

    ### COVERAGE
    {coverage_note}
    {user_guidance}
    Summarise what this dashboard shows.
"#};

const USER_GUIDANCE: &str = indoc! {r#"

    ### USER GUIDANCE
    {user_prompt}
"#};

const CHART_BLOCK: &str = indoc! {r#"
    --- CHART {position}: {chart_name} ---
    TYPE: {chart_type}
    PLOTTED FIELDS: {field_names}
    RESULT COLUMNS: {columns}
    RESULT ROWS: {rows}
    ROW COUNT: {row_count_note}
"#};

pub enum UserPrompt<'a> {
    SummarizeDashboard { dashboard: &'a DashboardSummaryInput },
}

impl UserPrompt<'_> {
    pub fn render(&self) -> String {
        match self {
            UserPrompt::SummarizeDashboard { dashboard } => {
                let charts = dashboard
                    .charts
                    .iter()
                    .enumerate()
                    .map(|(index, chart)| render_chart_block(index + 1, chart))
                    .collect::<Vec<_>>()
                    .join("\n");
                SUMMARIZE_DASHBOARD
                    .replace("{charts}", &charts)
                    .replace("{coverage_note}", &render_coverage_note(dashboard))
                    .replace(
                        "{user_guidance}",
                        &render_user_guidance(&dashboard.user_prompt),
                    )
            }
        }
    }
}

fn render_user_guidance(user_prompt: &Option<String>) -> String {
    match user_prompt {
        Some(prompt) => USER_GUIDANCE.replace("{user_prompt}", prompt),
        None => String::new(),
    }
}

fn render_chart_block(position: usize, chart: &ChartSummaryInput) -> String {
    let field_names = if chart.field_names.is_empty() {
        "not applicable".to_string()
    } else {
        chart.field_names.join(", ")
    };
    CHART_BLOCK
        .replace("{position}", &position.to_string())
        .replace("{chart_name}", &chart.chart_name)
        .replace("{chart_type}", &chart.chart_type)
        .replace("{field_names}", &field_names)
        .replace("{columns}", &chart.columns_json())
        .replace("{rows}", &chart.rows_json())
        .replace("{row_count_note}", &chart.row_count_note())
}

fn render_coverage_note(dashboard: &DashboardSummaryInput) -> String {
    let mut notes = vec![format!(
        "{} chart(s) shown above.",
        dashboard.charts.len()
    )];

    if !dashboard.skipped_chart_names.is_empty() {
        notes.push(format!(
            "{} chart(s) could not be loaded and are missing from this summary: {}.",
            dashboard.skipped_chart_names.len(),
            dashboard.skipped_chart_names.join(", ")
        ));
    }

    if notes.len() == 1 {
        notes.push("Every chart on this dashboard is included.".to_string());
    }

    notes.join(" ")
}
