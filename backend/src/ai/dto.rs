use crate::data_catalog::dto::Database;

#[derive(Clone, Debug)]
pub struct ChartSummaryInput {
    pub chart_name: String,
    pub chart_type: String,
    pub field_names: Vec<String>,
    pub columns: Vec<serde_json::Value>,
    pub rows: Vec<serde_json::Value>,
    pub total_row_count: usize,
}

impl ChartSummaryInput {
    pub fn columns_json(&self) -> String {
        serde_json::to_string_pretty(&self.columns).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn rows_json(&self) -> String {
        serde_json::to_string_pretty(&self.rows).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn row_count_note(&self) -> String {
        if self.rows.len() < self.total_row_count {
            format!(
                "Showing the first {} of {} rows.",
                self.rows.len(),
                self.total_row_count
            )
        } else {
            format!(
                "{} rows in total. This is the complete result.",
                self.total_row_count
            )
        }
    }
}

#[derive(Clone, Debug)]
pub struct DashboardSummaryInput {
    pub charts: Vec<ChartSummaryInput>,
    pub skipped_chart_names: Vec<String>,
    pub user_prompt: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TableRef {
    pub database: String,
    pub schema: String,
    pub table: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DatabaseSchemas {
    pub database: String,
    pub schemas: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TableColumns {
    pub database: String,
    pub schema: String,
    pub table: String,
    pub columns: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SchemaTables {
    pub database: String,
    pub schema: String,
    pub tables: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DeletionSet {
    pub obviously_irrelevant_tables: Vec<SchemaTables>,
    pub obviously_irrelevant_columns: Vec<TableColumns>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SelectionSet {
    pub relevant_tables: Vec<SchemaTables>,
    pub relevant_columns: Vec<TableColumns>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct LogicalPlanningResponse {
    pub logical_steps: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TableFunction {
    pub database: String,
    pub schema: String,
    pub table: String,
    pub table_function: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct FunctionalRoleAnalysisResult {
    pub database_structure: String,
    pub query_specific_content_analysis: String,
    pub table_functions: Vec<TableFunction>,
}

impl FunctionalRoleAnalysisResult {
    pub fn get_table_function(&self, table_name: &str) -> Option<&TableFunction> {
        self.table_functions
            .iter()
            .find(|tf| tf.table == table_name)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ExplorationQuery {
    pub motivation: String,
    pub sql: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ExplorationQueryResult {
    pub sql: String,
    pub sql_result: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct EmpericalObservationColumn {
    pub column_name: String,
    pub relevance_reason: String,
    pub observations: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct EmpericalObservation {
    pub relevant: bool,
    pub relevant_columns: Vec<EmpericalObservationColumn>,
    pub table_summary: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SchemaStatusColumn {
    pub name: String,
    pub data_type: String,
    pub description: String,
    pub observations: String,
    pub relevance_reason: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SchemaStatusTable {
    pub table_name: String,
    pub status: String,
    pub columns: Vec<SchemaStatusColumn>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SchemaLinkingRelevantColumn {
    pub column_name: String,
    pub relevance_reason: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SchemaLinkingRefinedTable {
    pub table_name: String,
    pub relevant_columns: Vec<SchemaLinkingRelevantColumn>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SchemaLinkingRejectedCandidate {
    pub table: String,
    pub column: String,
    pub reject_reason: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SchemaLinkingFinalSynthesisResponse {
    pub refined_schema: Vec<SchemaLinkingRefinedTable>,
    pub rejected_candidates: Vec<SchemaLinkingRejectedCandidate>,
    pub exploration_queries: Vec<String>,
    pub status: String,
}

#[derive(serde::Serialize)]
pub struct SchemaLinkingFinalSynthesisForSql {
    pub refined_schema: Vec<SchemaLinkingRefinedTable>,
    pub rejected_candidates: Vec<SchemaLinkingRejectedCandidate>,
    pub exploration_queries: Vec<String>,
}

impl From<&SchemaLinkingFinalSynthesisResponse> for SchemaLinkingFinalSynthesisForSql {
    fn from(r: &SchemaLinkingFinalSynthesisResponse) -> Self {
        Self {
            refined_schema: r.refined_schema.clone(),
            rejected_candidates: r.rejected_candidates.clone(),
            exploration_queries: r.exploration_queries.clone(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SqlGenerationExploreQuery {
    pub purpose: String,
    pub sql: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RefineNextAction {
    Explore,
    GenerateSql,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SqlGenerationActionType {
    Explore,
    Refine,
    GenerateSql,
    Confirm,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SqlGenerationAction {
    pub action: SqlGenerationActionType,
    pub explore_queries: Option<Vec<SqlGenerationExploreQuery>>,
    pub exploration_findings: Option<Vec<String>>,
    pub updated_understanding: Option<Vec<String>>,
    pub query_plan: Option<Vec<String>>,
    pub next_action: Option<String>,
    pub sql: Option<String>,
    pub query_description: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct VisualizationRecommendation {
    pub chart_type: crate::cell::constants::ChartType,
    pub x_axis: Option<String>,
    pub y_axis: Option<String>,
    pub series: Option<Vec<String>>,
    pub category: Option<String>,
    pub value: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VisualizationSeries {
    pub data_key: String,
    pub name: String,
    pub color_index: usize,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "chart_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResolvedVisualization {
    Bar {
        x_axis: String,
        series: Vec<VisualizationSeries>,
    },
    Line {
        x_axis: String,
        series: Vec<VisualizationSeries>,
    },
    Pie {
        category: String,
        value: String,
    },
    Metric,
}

impl VisualizationRecommendation {
    pub fn resolved(&self) -> anyhow::Result<ResolvedVisualization> {
        use crate::cell::constants::ChartType;

        Ok(match self.chart_type {
            ChartType::Bar => {
                let (x_axis, series) = self.resolved_axis_chart()?;
                ResolvedVisualization::Bar { x_axis, series }
            }
            ChartType::Line => {
                let (x_axis, series) = self.resolved_axis_chart()?;
                ResolvedVisualization::Line { x_axis, series }
            }
            ChartType::Pie => ResolvedVisualization::Pie {
                category: non_empty(&self.category).ok_or_else(|| {
                    anyhow::anyhow!("visualization recommendation for PIE is missing category")
                })?,
                value: non_empty(&self.value).ok_or_else(|| {
                    anyhow::anyhow!("visualization recommendation for PIE is missing value")
                })?,
            },
            ChartType::Metric => ResolvedVisualization::Metric,
        })
    }

    fn resolved_axis_chart(&self) -> anyhow::Result<(String, Vec<VisualizationSeries>)> {
        let x_axis = non_empty(&self.x_axis).ok_or_else(|| {
            anyhow::anyhow!(
                "visualization recommendation for {} is missing x_axis",
                self.chart_type.as_str()
            )
        })?;

        let mut seen = std::collections::HashSet::new();
        let mut columns: Vec<String> = self
            .series
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|column| non_empty(&Some(column)))
            .filter(|column| seen.insert(column.clone()))
            .collect();
        if columns.is_empty() {
            columns.extend(non_empty(&self.y_axis));
        }
        if columns.is_empty() {
            anyhow::bail!(
                "visualization recommendation for {} has neither series nor y_axis",
                self.chart_type.as_str()
            );
        }
        columns.truncate(crate::cell::constants::MAX_CHART_SERIES);

        let series = columns
            .into_iter()
            .enumerate()
            .map(|(index, column)| VisualizationSeries {
                data_key: column.clone(),
                name: column,
                color_index: index,
            })
            .collect();

        Ok((x_axis, series))
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RouterCode {
    TextToSql,
    Followup,
    Clarify,
    RejectOffTopic,
    RejectUnsafe,
    GenerateCanvas,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RouterDecision {
    pub code: RouterCode,
    pub message: String,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationHistoryTerminalStatus {
    Completed,
    AwaitingClarification,
    Rejected,
    Failed,
}

impl ConversationHistoryTerminalStatus {
    pub fn is_relevant(self, is_latest_turn: bool) -> bool {
        match self {
            Self::Completed => true,
            Self::AwaitingClarification => is_latest_turn,
            Self::Rejected | Self::Failed => false,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConversationHistoryTurn {
    pub user_question: String,
    pub terminal_status: ConversationHistoryTerminalStatus,
    pub assistant_message: Option<String>,
    pub generated_sql: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ConversationHistory {
    turns: Vec<ConversationHistoryTurn>,
}

impl ConversationHistory {
    pub fn select_relevant<T>(
        turns: Vec<T>,
        limit: usize,
        status_of: impl Fn(&T) -> ConversationHistoryTerminalStatus,
    ) -> Vec<T> {
        let total = turns.len();
        let relevant: Vec<T> = turns
            .into_iter()
            .enumerate()
            .filter(|(index, turn)| status_of(turn).is_relevant(index + 1 == total))
            .map(|(_, turn)| turn)
            .collect();
        let skip = relevant.len().saturating_sub(limit);
        relevant.into_iter().skip(skip).collect()
    }

    pub fn from_turns(turns: Vec<ConversationHistoryTurn>, limit: usize) -> Self {
        Self {
            turns: Self::select_relevant(turns, limit, |turn| turn.terminal_status),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ConversationHistoryTurn> {
        self.turns.iter()
    }

    pub fn len(&self) -> usize {
        self.turns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }

    pub fn has_completed_query(&self) -> bool {
        self.turns.iter().any(|turn| {
            matches!(
                turn.terminal_status,
                ConversationHistoryTerminalStatus::Completed
            ) && turn.generated_sql.is_some()
        })
    }
}

impl<'a> IntoIterator for &'a ConversationHistory {
    type Item = &'a ConversationHistoryTurn;
    type IntoIter = std::slice::Iter<'a, ConversationHistoryTurn>;

    fn into_iter(self) -> Self::IntoIter {
        self.turns.iter()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "event_name", content = "data", rename_all = "snake_case")]
pub enum Event {
    Starting,
    Error(String),
    Completed,
    SuggestedFollowups {
        questions: Vec<String>,
    },
    GeneratingCandidateHypothesis,
    GeneratedCandidateHypothesis(Vec<String>),
    IntegratingCandidatePlans,
    IntegratedCandidatePlans {
        master_plan: String,
    },
    ExecutingSearchSpaceReduction,
    ExecutedSearchSpaceReduction {
        pruned_data_catalog: Database,
    },
    ExecutingFunctionalRoleAnalysis,
    ExecutedFunctionalRoleAnalysis {
        functional_role_analysis_result: FunctionalRoleAnalysisResult,
    },
    ExecutingDataProfiling,
    ExecutedDataProfiling {
        emperical_observations: Vec<EmpericalObservation>,
    },
    ExecutingSchemaLinkingSynthesis,
    ExecutedSchemaLinkingSynthesis {
        linked_schema: SchemaLinkingFinalSynthesisResponse,
    },
    GeneratingSql,
    GeneratedSql {
        sql: String,
    },
    ExecutingQuery,
    ExecutedQuery {
        columns: Vec<serde_json::Value>,
        data: Vec<serde_json::Value>,
    },
    RecommendingVisualization,
    RecommendedVisualization {
        visualization: ResolvedVisualization,
    },
    Narrating,
    Narrated {
        narration: String,
    },
    Routing,
    Routed {
        decision: RouterDecision,
    },
    AwaitingClarification,
    Rejected,
    GeneratingCanvas,
    CanvasGenerated {
        canvas_id: uuid::Uuid,
        name: String,
        description: Option<String>,
        reused: bool,
        reasoning: String,
        cells_added: i32,
        cells_updated: i32,
        cells_removed: i32,
        sql_cell_count: i32,
        chart_cell_count: i32,
        dashboard_charts_count: i32,
    },
}

pub struct EventChannels {
    pub sse_tx: tokio::sync::mpsc::Sender<Event>,
    pub persist_tx: tokio::sync::mpsc::Sender<Event>,
}

impl EventChannels {
    pub async fn send(&self, event: Event) -> anyhow::Result<()> {
        if let Err(e) = self.sse_tx.send(event.clone()).await {
            tracing::error!("Failed to send event to SSE channel: {e}");
        }
        self.persist_tx.send(event).await?;
        Ok(())
    }

    pub async fn send_sse_only(&self, event: Event) {
        let _ = self.sse_tx.send(event).await;
    }
}

impl Event {
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).expect("Event JSON serialization")
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AiChartSeries {
    pub column: String,
    pub aggregate_function: Option<crate::cell::constants::AggregateFunction>,
    pub label: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CanvasPlanChart {
    pub chart_type: crate::cell::constants::ChartType,
    pub x_axis: Option<String>,
    pub y_axis: Option<String>,
    pub series: Option<Vec<AiChartSeries>>,
    pub category: Option<String>,
    pub value: Option<String>,
    /// PIE only — BAR and LINE carry an aggregate function per series.
    pub aggregate_function: Option<crate::cell::constants::AggregateFunction>,
    pub grid_width: Option<i32>,
    pub grid_height: Option<i32>,
}

#[derive(Clone, Debug)]
pub struct PlannedSeries {
    pub column: String,
    pub aggregate_function: crate::cell::constants::AggregateFunction,
    pub label: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PlannedAxisChart {
    pub x_axis: String,
    pub series: Vec<PlannedSeries>,
}

#[derive(Clone, Debug)]
pub struct PlannedPieChart {
    pub category: String,
    pub value: String,
    pub aggregate_function: crate::cell::constants::AggregateFunction,
}

#[derive(Clone, Debug)]
pub enum PlannedChartSpec {
    Bar(PlannedAxisChart),
    Line(PlannedAxisChart),
    Pie(PlannedPieChart),
    Metric,
}

#[derive(Clone, Debug)]
pub struct PlannedChart {
    pub spec: PlannedChartSpec,
    pub grid_width: Option<i32>,
    pub grid_height: Option<i32>,
}

impl PlannedChart {
    pub fn grid_width_units(&self) -> Option<u16> {
        to_grid_units(self.grid_width)
    }

    pub fn grid_height_units(&self) -> Option<u16> {
        to_grid_units(self.grid_height)
    }
}

impl CanvasPlanChart {
    fn planned_axis_chart(&self) -> Option<PlannedAxisChart> {
        let x_axis = non_empty(&self.x_axis).or_else(|| non_empty(&self.category))?;

        let mut seen = std::collections::HashSet::new();
        let mut series: Vec<PlannedSeries> = self
            .series
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter(|entry| !entry.column.trim().is_empty())
            .filter(|entry| seen.insert(entry.column.clone()))
            .map(|entry| {
                Some(PlannedSeries {
                    column: entry.column,
                    aggregate_function: entry.aggregate_function?,
                    label: entry.label,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        if series.is_empty() {
            return None;
        }
        series.truncate(crate::cell::constants::MAX_CHART_SERIES);
        Some(PlannedAxisChart { x_axis, series })
    }

    fn planned_pie_chart(&self) -> Option<PlannedPieChart> {
        Some(PlannedPieChart {
            category: non_empty(&self.category).or_else(|| non_empty(&self.x_axis))?,
            value: non_empty(&self.value).or_else(|| non_empty(&self.y_axis))?,
            aggregate_function: self.aggregate_function.clone()?,
        })
    }

    /// Accepts axis/category as synonyms, since agents mix the two vocabularies.
    fn planned(&self) -> Option<PlannedChart> {
        use crate::cell::constants::ChartType;

        let spec = match self.chart_type {
            ChartType::Bar => PlannedChartSpec::Bar(self.planned_axis_chart()?),
            ChartType::Line => PlannedChartSpec::Line(self.planned_axis_chart()?),
            ChartType::Pie => PlannedChartSpec::Pie(self.planned_pie_chart()?),
            ChartType::Metric => PlannedChartSpec::Metric,
        };
        Some(PlannedChart {
            spec,
            grid_width: self.grid_width,
            grid_height: self.grid_height,
        })
    }
}

fn resolve_chart(chart: Option<&CanvasPlanChart>, title: &Option<String>) -> Option<PlannedChart> {
    let chart = chart?;
    let planned = chart.planned();
    if planned.is_none() {
        tracing::warn!(
            "canvas plan: dropped {} chart for {:?} because its axis, series or category fields were missing",
            chart.chart_type.as_str(),
            title.as_deref().unwrap_or("<untitled>")
        );
    }
    planned
}

fn to_grid_units(value: Option<i32>) -> Option<u16> {
    value.map(|v| v.clamp(0, u16::MAX as i32) as u16)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CanvasCandidateCell {
    pub title: String,
    pub sql: String,
    pub chart_type: Option<crate::cell::constants::ChartType>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CanvasCandidate {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub cells: Vec<CanvasCandidateCell>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CanvasCellAction {
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CanvasPlanCell {
    pub action: CanvasCellAction,
    pub target_cell_index: Option<i32>,
    pub title: Option<String>,
    pub sql: Option<String>,
    pub chart: Option<CanvasPlanChart>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CanvasPlan {
    pub target_canvas_index: Option<i32>,
    pub reasoning: String,
    pub name: String,
    pub description: String,
    pub cells: Vec<CanvasPlanCell>,
}

#[derive(Clone, Debug)]
pub enum CanvasOp {
    Create {
        title: Option<String>,
        sql: String,
        chart: Option<PlannedChart>,
    },
    Update {
        index: usize,
        title: Option<String>,
        sql: Option<String>,
        chart: Option<PlannedChart>,
    },
    Delete {
        index: usize,
    },
}

fn to_zero_based(index: Option<i32>, len: usize) -> Option<usize> {
    let index = usize::try_from(index?).ok()?.checked_sub(1)?;
    (index < len).then_some(index)
}

fn non_empty(value: &Option<String>) -> Option<String> {
    let trimmed = value.as_deref()?.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn normalize_sql(sql: &str) -> String {
    sql.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

impl CanvasPlan {
    pub fn resolve_target_canvas(&self, candidates: &[CanvasCandidate]) -> Option<uuid::Uuid> {
        to_zero_based(self.target_canvas_index, candidates.len()).map(|i| candidates[i].id)
    }

    pub fn resolve_ops(&self, target: Option<&CanvasCandidate>) -> Vec<CanvasOp> {
        let cell_count = target.map(|c| c.cells.len()).unwrap_or(0);
        let mut present: std::collections::HashSet<String> = target
            .map(|c| {
                c.cells
                    .iter()
                    .map(|cell| normalize_sql(&cell.sql))
                    .collect()
            })
            .unwrap_or_default();

        let mut ops = Vec::new();
        for cell in self.cells.iter() {
            let Some(op) = Self::resolve_op(cell, cell_count) else {
                continue;
            };
            match &op {
                CanvasOp::Create { title, sql, .. } => {
                    if !present.insert(normalize_sql(sql)) {
                        tracing::warn!(
                            "canvas plan: dropped create of {:?} because its query is already in the target canvas",
                            title.as_deref().unwrap_or("<untitled>")
                        );
                        continue;
                    }
                }
                CanvasOp::Delete { index } => {
                    if let Some(cell) = target.and_then(|c| c.cells.get(*index)) {
                        present.remove(&normalize_sql(&cell.sql));
                    }
                }
                CanvasOp::Update { .. } => {}
            }
            ops.push(op);
        }
        ops
    }

    fn resolve_op(cell: &CanvasPlanCell, cell_count: usize) -> Option<CanvasOp> {
        match cell.action {
            CanvasCellAction::Create => Some(CanvasOp::Create {
                title: non_empty(&cell.title),
                sql: non_empty(&cell.sql)?,
                chart: resolve_chart(cell.chart.as_ref(), &cell.title),
            }),
            CanvasCellAction::Update => {
                let index = to_zero_based(cell.target_cell_index, cell_count)?;
                let title = non_empty(&cell.title);
                let sql = non_empty(&cell.sql);
                let chart = resolve_chart(cell.chart.as_ref(), &cell.title);
                if title.is_none() && sql.is_none() && chart.is_none() {
                    return None;
                }
                Some(CanvasOp::Update {
                    index,
                    title,
                    sql,
                    chart,
                })
            }
            CanvasCellAction::Delete => Some(CanvasOp::Delete {
                index: to_zero_based(cell.target_cell_index, cell_count)?,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Event;

    #[test]
    fn event_json_event_name_and_data_shape() {
        let s = Event::Starting.to_json_string();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["event_name"], "starting");
        assert!(v["data"].is_null());

        let s = Event::Error("oops".into()).to_json_string();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["event_name"], "error");
        assert_eq!(v["data"], "oops");

        let s = Event::GeneratedCandidateHypothesis(vec!["a".into()]).to_json_string();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["event_name"], "generated_candidate_hypothesis");
        assert_eq!(v["data"], serde_json::json!(["a"]));
    }
}
