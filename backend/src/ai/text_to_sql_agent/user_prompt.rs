use indoc::indoc;

use crate::ai::dto::{ExplorationQueryResult, FunctionalRoleAnalysisResult, RefineNextAction};
use crate::connection::constants::SqlDialect;
use crate::data_catalog::dto::Column;

const LOGICAL_PLANNING_HYPOTHESIS_GENERATION: &str = indoc! {r#"
    ### USER QUESTION
    {question}
"#};

const AGGREGATING_PLAN_CANDIDATES: &str = indoc! {r#"
    ### USER QUESTION
    {question}

    ### CANDIDATE PLANS
    {candidate_plans}
"#};

const FUNCTIONAL_ROLE_ANALYSIS: &str = indoc! {r#"
    ### USER QUESTION
    {question}

    ### LOGICAL PLAN
    {logical_plan}

    ### DATABASE SCHEMA
    {database_schema}
"#};

const DATA_PROFILING_BEFORE: &str = indoc! {r#"
    ## TARGET TABLE: {table_name}
    
    ### Columns
    {columns}
    
    ### USER QUESTION
    {question}

    ### SQL_DIALECT: {sql_dialect}
    
    ### ANTICIPATED ROLE
    This table was identified as: {semantic_role}. Use this to guide your exploration.

    ## MISSION
    - Generate 3-8 SQL queries to investigate.
    - Focus on understanding the table's semantics and utility.
    
    ## Motivation for Exploration
    
    ### Semantic Alignment
    - Check distinct values to understand what the column *means* versus what the query *needs*. 
    - Ex: If column is "type", does it contain the specific categories?
    - Ex: If column is "status", does it contain values like "Active" or code "1"?

    ### Granularity & Scope
    - Verify the table"s grain.
    - Ex: is it one row per Order or per Item?.
    - This determines if it supports the required aggregations.

    ### Bridge/Connectivity
    - If this looks like a linking table, verify the Foreign Keys are populated (not all NULL) to ensure it can actually serve as a bridge.

    ### Data Quality
    - Are critical columns (targets for filters or answers) usable, or are they mostly NULL?.

    ## Output Format
    Always respond in this exact JSON format:
    [
        {
            "motivation": "Checking distinct values in "status" to see if it aligns with the query"s filter requirement",
            "sql": "SELECT DISTINCT status FROM table_name LIMIT 10"
        }
    ]
    
    Generate your exploration queries.
"#};

const DATA_PROFILING_AFTER: &str = indoc! {r#"
    Based on the exploration history and results below, determine if table "{table_name}" is RELEVANT to the User Question.

    ## EXPLORATION EVIDENCE
    {observations}

    ## DECISION GUIDELINES
    - **Direct Match**: Contains the specific answer data.
    - **Bridge Table**: Contains IDs needed to join other relevant tables (CRITICAL: Keep even if no other useful data).
    - **Filter Source**: Contains columns needed to restrict the result.
    - **Calculation Support**: Contains numerical columns needed for aggregation (e.g., "score" for AVG, "price" for SUM).

    ## OUTPUT GUIDELINES
    - "relevance_reason": Explain the LOGICAL role of the column (e.g., "Provides the Join Key for X and Y", "Contains the target column Z").
    - "observations": Summarize FACTUAL findings from exploration for the column (e.g., "Column A contains integer codes 1-5", "Table is empty").
    - "table_summary": A concise summary of what this table represents in the context of the query.
    - "relevant": Whether the table is relevant to the user question after analysis of all its columns.

    ## Output Format
    Always respond in this exact JSON format:
    {
        "relevant": true/false,
        "relevant_columns: [
            {
                "column_name": "col1",
                "relevance_reason": "...",
                observations: "...",
            }
        ],
        "table_summary": "..."
    }
"#};

const SCHEMA_LINKING_FINAL_SYNTHESIS_FIRST: &str = indoc! {r#"
    ### USER QUESTION
    {question}

    ### SEMANTIC ANALYSIS
    {semantic_analysis}

    ### SCHEMA STATUS
    {schema_status}

    ### MAX_REFINE_ROUNDS
    {max_refine_rounds}

    ### SQL_DIALECT: {sql_dialect}

    Begin refinement
"#};

const SCHEMA_LINKING_FINAL_SYNTHESIS_EXPLORATION_RESULTS: &str = indoc! {r#"
    ### EXPLORATION QUERY RESULTS
    {results}
"#};

const SQL_GENERATION_FIRST: &str = indoc! {r#"
    ### USER QUESTION
    {question}

    ### SCHEMA
    {schema}

    ### CURRENT PLAN
    {current_plan}

    ### SQL_DIALECT: {sql_dialect}

    Now begin your exploration to generate the correct SQL
"#};

const SQL_GENERATION_EXPLORATION_RESULT: &str = indoc! {r#"
    ### EXPLORATION RESULTS
    {results}
"#};

const SQL_GENERATION_REFINE_RESULT: &str = indoc! {r#"
    Perform {next_action}
"#};

const SQL_GENERATION_QUERY_RESULT: &str = indoc! {r#"
    Results of executing the generated SQL is: {result}
"#};

pub enum UserPrompt {
    LogicalPlanningHypothesisGeneration {
        question: String,
    },
    AggregatingPlanCandidates {
        question: String,
        candidate_plans: String,
    },
    FunctionalRoleAnalysis {
        question: String,
        logical_plan: String,
        database_schema: String,
    },
    DataProfilingBefore {
        table_name: String,
        columns: Vec<Column>,
        question: String,
        semantic_role: String,
        sql_dialect: SqlDialect,
    },
    DataProfilingAfter {
        table_name: String,
        observations: Vec<ExplorationQueryResult>,
    },
    SchemaLinkingFinalSynthesisFirst {
        question: String,
        semantic_analysis: FunctionalRoleAnalysisResult,
        schema_status: String,
        max_refine_rounds: u32,
        sql_dialect: SqlDialect,
    },
    SchemaLinkingFinalSynthesisExplorationResults {
        results: String,
    },
    SqlGenerationFirst {
        question: String,
        schema: String,
        current_plan: String,
        sql_dialect: SqlDialect,
    },
    SqlGenerationExplorationResult {
        results: String,
    },
    SqlGenerationRefineResult {
        next_action: RefineNextAction,
    },
    SqlGenerationQueryResult {
        result: String,
    },
}

impl UserPrompt {
    pub fn render(&self) -> String {
        match self {
            UserPrompt::LogicalPlanningHypothesisGeneration { question } => {
                LOGICAL_PLANNING_HYPOTHESIS_GENERATION.replace("{question}", question)
            }
            UserPrompt::AggregatingPlanCandidates {
                question,
                candidate_plans,
            } => AGGREGATING_PLAN_CANDIDATES
                .replace("{question}", question)
                .replace("{candidate_plans}", candidate_plans),
            UserPrompt::FunctionalRoleAnalysis {
                question,
                logical_plan,
                database_schema,
            } => FUNCTIONAL_ROLE_ANALYSIS
                .replace("{question}", question)
                .replace("{logical_plan}", logical_plan)
                .replace("{database_schema}", database_schema),
            UserPrompt::DataProfilingBefore {
                table_name,
                columns,
                question,
                semantic_role,
                sql_dialect,
            } => {
                let formatted_columns = columns
                    .iter()
                    .map(|c| format!("{} ({}): {}", c.name, c.data_type, c.description))
                    .collect::<Vec<_>>()
                    .join("\n");
                DATA_PROFILING_BEFORE
                    .replace("{table_name}", table_name)
                    .replace("{columns}", formatted_columns.as_str())
                    .replace("{question}", question)
                    .replace("{semantic_role}", semantic_role)
                    .replace("{sql_dialect}", &sql_dialect.to_string())
            }
            UserPrompt::DataProfilingAfter {
                table_name,
                observations,
            } => {
                let formatted_observations = observations
                    .iter()
                    .map(|o| format!("SQL: {}\nResult: {}", o.sql, o.sql_result))
                    .collect::<Vec<_>>()
                    .join("\n\n");
                DATA_PROFILING_AFTER
                    .replace("{table_name}", table_name)
                    .replace("{observations}", &formatted_observations)
            }
            UserPrompt::SchemaLinkingFinalSynthesisFirst {
                question,
                semantic_analysis,
                schema_status,
                max_refine_rounds,
                sql_dialect,
            } => {
                let semantic_analysis_str =
                    serde_json::to_string_pretty(semantic_analysis).unwrap_or_default();
                SCHEMA_LINKING_FINAL_SYNTHESIS_FIRST
                    .replace("{question}", question)
                    .replace("{semantic_analysis}", &semantic_analysis_str)
                    .replace("{schema_status}", schema_status)
                    .replace("{max_refine_rounds}", &max_refine_rounds.to_string())
                    .replace("{sql_dialect}", &sql_dialect.to_string())
            }
            UserPrompt::SchemaLinkingFinalSynthesisExplorationResults { results } => {
                SCHEMA_LINKING_FINAL_SYNTHESIS_EXPLORATION_RESULTS.replace("{results}", results)
            }
            UserPrompt::SqlGenerationFirst {
                question,
                schema,
                current_plan,
                sql_dialect,
            } => SQL_GENERATION_FIRST
                .replace("{question}", question)
                .replace("{schema}", schema)
                .replace("{current_plan}", current_plan)
                .replace("{sql_dialect}", &sql_dialect.to_string()),
            UserPrompt::SqlGenerationExplorationResult { results } => {
                SQL_GENERATION_EXPLORATION_RESULT.replace("{results}", results)
            }
            UserPrompt::SqlGenerationRefineResult { next_action } => {
                let next_action_str = match next_action {
                    RefineNextAction::Explore => "EXPLORE",
                    RefineNextAction::GenerateSql => "GENERATE_SQL",
                };
                SQL_GENERATION_REFINE_RESULT.replace("{next_action}", next_action_str)
            }
            UserPrompt::SqlGenerationQueryResult { result } => {
                SQL_GENERATION_QUERY_RESULT.replace("{result}", result)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::dto::TableFunction;

    fn column(name: &str, data_type: &str, description: &str) -> crate::data_catalog::dto::Column {
        crate::data_catalog::dto::Column {
            name: name.to_string(),
            data_type: data_type.to_string(),
            description: description.to_string(),
            sample_values: vec![],
            should_delete: false,
            should_select: false,
        }
    }

    #[test]
    fn parallel_data_profiling_before_renders_empty_columns() {
        let prompt = UserPrompt::DataProfilingBefore {
            table_name: "orders".to_string(),
            columns: vec![],
            question: "How many orders per month?".to_string(),
            semantic_role: "Target Table for order counts".to_string(),
            sql_dialect: SqlDialect::Postgresql,
        };
        let rendered = prompt.render();
        assert!(rendered.contains("## TARGET TABLE: orders"));
        assert!(rendered.contains("### Columns\n\n"));
        assert!(rendered.contains("How many orders per month?"));
        assert!(rendered.contains("Target Table for order counts"));
    }

    #[test]
    fn parallel_data_profiling_before_renders_single_column() {
        let prompt = UserPrompt::DataProfilingBefore {
            table_name: "users".to_string(),
            columns: vec![column("id", "integer", "Primary key identifier")],
            question: "How many users?".to_string(),
            semantic_role: "Filtering Table".to_string(),
            sql_dialect: SqlDialect::Postgresql,
        };
        let rendered = prompt.render();
        assert!(rendered.contains("## TARGET TABLE: users"));
        assert!(rendered.contains("id (integer): Primary key identifier"));
        assert!(rendered.contains("How many users?"));
        assert!(rendered.contains("Filtering Table"));
    }

    #[test]
    fn parallel_data_profiling_before_renders_multiple_columns() {
        let prompt = UserPrompt::DataProfilingBefore {
            table_name: "patents".to_string(),
            columns: vec![
                column("grant_date", "date", "Date the patent was granted"),
                column(
                    "status",
                    "varchar",
                    "Current status: Active, Expired, Pending",
                ),
            ],
            question: "Patents granted in 2020?".to_string(),
            semantic_role: "Bridge table connecting Inventors and Applications".to_string(),
            sql_dialect: SqlDialect::Postgresql,
        };
        let rendered = prompt.render();
        assert!(rendered.contains("## TARGET TABLE: patents"));
        assert!(rendered.contains("grant_date (date): Date the patent was granted"));
        assert!(rendered.contains("status (varchar): Current status: Active, Expired, Pending"));
        assert!(rendered.contains("Patents granted in 2020?"));
        assert!(rendered.contains("Bridge table connecting Inventors and Applications"));
    }

    fn ecommerce_semantic_analysis() -> FunctionalRoleAnalysisResult {
        FunctionalRoleAnalysisResult {
            database_structure: "One ecommerce schema with orders and customers".to_string(),
            query_specific_content_analysis: "Orders carry the purchase timestamp".to_string(),
            table_functions: vec![TableFunction {
                database: "ecommerce".to_string(),
                schema: "main".to_string(),
                table: "orders".to_string(),
                table_function: "Target Table for order counts".to_string(),
            }],
        }
    }

    #[test]
    fn snapshot_logical_planning_hypothesis_generation() {
        insta::assert_snapshot!(
            UserPrompt::LogicalPlanningHypothesisGeneration {
                question: "How many orders were placed per month?".to_string(),
            }
            .render()
        );
    }

    #[test]
    fn snapshot_aggregating_plan_candidates() {
        insta::assert_snapshot!(
            UserPrompt::AggregatingPlanCandidates {
                question: "How many orders were placed per month?".to_string(),
                candidate_plans: "1. Count orders grouped by month\n2. Count order items"
                    .to_string(),
            }
            .render()
        );
    }

    #[test]
    fn snapshot_functional_role_analysis() {
        insta::assert_snapshot!(
            UserPrompt::FunctionalRoleAnalysis {
                question: "How many orders were placed per month?".to_string(),
                logical_plan: "Count orders grouped by purchase month".to_string(),
                database_schema: "orders(order_id, order_purchase_timestamp)".to_string(),
            }
            .render()
        );
    }

    #[test]
    fn snapshot_data_profiling_before() {
        insta::assert_snapshot!(
            UserPrompt::DataProfilingBefore {
                table_name: "orders".to_string(),
                columns: vec![
                    column("order_id", "TEXT", "Order identifier"),
                    column(
                        "order_purchase_timestamp",
                        "TEXT",
                        "When the order was placed"
                    ),
                ],
                question: "How many orders were placed per month?".to_string(),
                semantic_role: "Target Table for order counts".to_string(),
                sql_dialect: SqlDialect::Sqlite,
            }
            .render()
        );
    }

    #[test]
    fn snapshot_data_profiling_after() {
        insta::assert_snapshot!(
            UserPrompt::DataProfilingAfter {
                table_name: "orders".to_string(),
                observations: vec![ExplorationQueryResult {
                    sql: "SELECT MIN(order_purchase_timestamp) FROM orders".to_string(),
                    sql_result: "2016-09-04 21:15:19".to_string(),
                }],
            }
            .render()
        );
    }

    #[test]
    fn snapshot_schema_linking_final_synthesis_first() {
        insta::assert_snapshot!(
            UserPrompt::SchemaLinkingFinalSynthesisFirst {
                question: "How many orders were placed per month?".to_string(),
                semantic_analysis: ecommerce_semantic_analysis(),
                schema_status: "orders: order_id, order_purchase_timestamp".to_string(),
                max_refine_rounds: 3,
                sql_dialect: SqlDialect::Sqlite,
            }
            .render()
        );
    }

    #[test]
    fn snapshot_schema_linking_final_synthesis_exploration_results() {
        insta::assert_snapshot!(
            UserPrompt::SchemaLinkingFinalSynthesisExplorationResults {
                results: "order_purchase_timestamp spans 2016 to 2018".to_string(),
            }
            .render()
        );
    }

    #[test]
    fn snapshot_sql_generation_first() {
        insta::assert_snapshot!(
            UserPrompt::SqlGenerationFirst {
                question: "How many orders were placed per month?".to_string(),
                schema: "orders(order_id, order_purchase_timestamp)".to_string(),
                current_plan: "Count orders grouped by purchase month".to_string(),
                sql_dialect: SqlDialect::Sqlite,
            }
            .render()
        );
    }

    #[test]
    fn snapshot_sql_generation_exploration_result() {
        insta::assert_snapshot!(
            UserPrompt::SqlGenerationExplorationResult {
                results: "12 distinct months".to_string(),
            }
            .render()
        );
    }

    #[test]
    fn snapshot_sql_generation_refine_result() {
        insta::assert_snapshot!(
            UserPrompt::SqlGenerationRefineResult {
                next_action: RefineNextAction::GenerateSql,
            }
            .render()
        );
    }

    #[test]
    fn snapshot_sql_generation_query_result() {
        insta::assert_snapshot!(
            UserPrompt::SqlGenerationQueryResult {
                result: "month | order_count\n2017-01 | 800".to_string(),
            }
            .render()
        );
    }
}
