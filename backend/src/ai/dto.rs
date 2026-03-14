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
