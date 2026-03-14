use crate::ai::constants::{self, DATA_CATALOG_BATCHES, SCHEMA_LINKING_MAX_REFINE_ROUNDS};
use crate::ai::dto::{
    DeletionSet, EmpericalObservation, ExplorationQuery, ExplorationQueryResult,
    FunctionalRoleAnalysisResult, RefineNextAction, SchemaLinkingFinalSynthesisForSql,
    SchemaLinkingFinalSynthesisResponse, SchemaStatusColumn, SchemaStatusTable, SelectionSet,
    SqlGenerationAction, SqlGenerationActionType, TableFunction,
};
use crate::ai::prompts::{SystemPrompt, UserPrompt};
use crate::cell::service::{execute_sql_queries_in_parallel, execute_sql_query};
use crate::common::AppState;
use crate::connection::constants::{SourceType, SqlDialect};
use crate::data_catalog::dto::Database;
use crate::data_catalog::{
    get_data_catalog_batches, join_pruned_batches, prune_data_catalog_batch,
};
use crate::entity;
use anyhow::Result;
use rig::completion::Prompt;
use rig::message::Message;
use rig::prelude::TypedPrompt;
use rig::{
    client::{Client, CompletionClient},
    providers::anthropic::client::AnthropicExt,
};
use tracing::info;

pub async fn execute(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    client: &Client<AnthropicExt>,
) -> Result<String> {
    let (master_plan, linked_schema) =
        execute_schema_linking(app_state, connection, question, client).await?;
    let generated_sql =
        sql_generation(connection, &client, question, &master_plan, linked_schema).await?;
    info!("generated_sql: {}", generated_sql);
    Ok(generated_sql)
}

pub async fn execute_schema_linking(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    client: &Client<AnthropicExt>,
) -> Result<(String, SchemaLinkingFinalSynthesisResponse)> {
    let candidate_hypothesis =
        execute_hypothesis_generation(app_state, connection, question, client).await?;

    let master_plan = execute_integrate_candidate_plans(
        app_state,
        connection,
        question,
        client,
        candidate_hypothesis,
    )
    .await?;

    info!("master_plan: {master_plan}");

    let pruned_data_catalog =
        execute_search_space_reduction(app_state, connection, question, &master_plan, &client)
            .await?;
    info!(
        "pruned_data_catalog: {}",
        serde_json::to_string_pretty(&pruned_data_catalog).unwrap_or_default()
    );
    let (emperical_observations, functional_role_analysis_result) =
        execute_hypothesis_verification(
            app_state,
            connection,
            question,
            &master_plan,
            &pruned_data_catalog,
            &client,
        )
        .await?;
    info!(
        "emperical_observations: {}",
        serde_json::to_string_pretty(&emperical_observations).unwrap_or_default()
    );
    info!(
        "functional_role_analysis_result: {}",
        serde_json::to_string_pretty(&functional_role_analysis_result).unwrap_or_default()
    );

    let linked_schema = schema_linking_final_synthesis(
        connection,
        question,
        functional_role_analysis_result,
        &emperical_observations,
        &pruned_data_catalog,
        &client,
    )
    .await?;
    info!(
        "linked_schema: {}",
        serde_json::to_string_pretty(&linked_schema).unwrap_or_default()
    );
    Ok((master_plan, linked_schema))
}

async fn execute_hypothesis_generation(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    client: &Client<AnthropicExt>,
) -> Result<Vec<String>> {
    let hypothesis_generation_agent = client
        .agent("claude-haiku-4-5")
        .name("hypothesis_generation_agent")
        .preamble(SystemPrompt::LogicalPlanning.as_str())
        .temperature(0.8)
        .max_tokens(2048)
        .build();
    let mut candidate_hypothesis = Vec::new();
    for _rounds in 0..constants::LOGICAL_PLAN_HYPOTHESIS_GENERATION_ROUNDS {
        let prompt = UserPrompt::LogicalPlanningHypothesisGeneration {
            question: question.to_string(),
        }
        .render();
        let result = hypothesis_generation_agent.prompt(prompt).await?;
        candidate_hypothesis.push(result);
    }
    Ok(candidate_hypothesis)
}

async fn execute_integrate_candidate_plans(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    client: &Client<AnthropicExt>,
    candidate_hypothesis: Vec<String>,
) -> Result<String> {
    let integrate_candidate_plan_agent = client
        .agent("claude-haiku-4-5")
        .name("integrate_candidate_plan_agent")
        .preamble(SystemPrompt::AggregatingPlanCandidates.as_str())
        .temperature(0.2)
        .max_tokens(2048)
        .build();
    let candidate_plans_str = format_candidate_plans_for_prompt(candidate_hypothesis);
    let prompt = UserPrompt::AggregatingPlanCandidates {
        question: question.to_string(),
        candidate_plans: candidate_plans_str,
    }
    .render();
    let master_plan = integrate_candidate_plan_agent.prompt(prompt).await?;
    Ok(master_plan)
}

pub(crate) async fn execute_search_space_reduction(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    master_plan: &str,
    client: &Client<AnthropicExt>,
) -> Result<Database> {
    let data_catalog_batches = get_data_catalog_batches(&connection, DATA_CATALOG_BATCHES)
        .await
        .map_err(anyhow::Error::from)?;
    info!("initial data_catalog_batches: {:?}", data_catalog_batches);
    let mut pruned_batches = Vec::new();
    for mut data_catalog_batch in data_catalog_batches {
        let data_catalog_deletion_agent = client
            .agent("claude-haiku-4-5")
            .name("data_catalog_deletion_agent")
            .preamble(SystemPrompt::IdentifyingDeletionSet.as_str())
            .temperature(0.2)
            .max_tokens(2048)
            .build();
        let data_catalog_selection_agent = client
            .agent("claude-haiku-4-5")
            .name("data_catalog_selection_agent")
            .preamble(SystemPrompt::IdentifyingSelectionSet.as_str())
            .temperature(0.2)
            .max_tokens(2048)
            .build();
        let data_catalog_json = data_catalog_batch.to_display_string().unwrap_or_default();
        let prompt = format!(
            "USER QUESTION {}\n MASTER LOGICAL PLAN: {}\n DATABASE SCHEMA: {}",
            question, master_plan, data_catalog_json
        );
        let (deletion_result, selection_result) = tokio::join!(
            data_catalog_deletion_agent.prompt_typed::<DeletionSet>(&prompt),
            data_catalog_selection_agent.prompt_typed::<SelectionSet>(&prompt)
        );
        let deletion_set = deletion_result?;
        let selection_set = selection_result?;
        info!("Deletion: {:?}", deletion_set);
        info!("Selection: {:?}", selection_set);
        prune_data_catalog_batch(&mut data_catalog_batch, deletion_set, selection_set);
        pruned_batches.push(data_catalog_batch);
    }
    let pruned_data_catalog = join_pruned_batches(pruned_batches);
    Ok(pruned_data_catalog)
}

pub(crate) async fn execute_hypothesis_verification(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    master_plan: &str,
    pruned_data_catalog: &Database,
    client: &Client<AnthropicExt>,
) -> Result<(Vec<EmpericalObservation>, FunctionalRoleAnalysisResult)> {
    let functional_role_analysis_result = execute_functional_role_analysis(
        app_state,
        connection,
        question,
        master_plan,
        pruned_data_catalog,
        client,
    )
    .await?;
    let emperical_observations = execute_data_profiling(
        app_state,
        connection,
        question,
        pruned_data_catalog,
        client,
        &functional_role_analysis_result,
    )
    .await?;
    Ok((emperical_observations, functional_role_analysis_result))
}

/// Functional role analysis does not return tables irrelevant for further analysis.
async fn execute_functional_role_analysis(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    master_plan: &str,
    pruned_data_catalog: &Database,
    client: &Client<AnthropicExt>,
) -> Result<FunctionalRoleAnalysisResult> {
    let functional_role_analysis_agent = client
        .agent("claude-haiku-4-5")
        .name("functional_role_analysis_agent")
        .preamble(SystemPrompt::SemanticLinking.as_str())
        .temperature(0.2)
        .max_tokens(2048)
        .build();
    let prompt = UserPrompt::FunctionalRoleAnalysis {
        question: question.to_string(),
        logical_plan: master_plan.to_string(),
        database_schema: pruned_data_catalog.to_display_string()?,
    }
    .render();
    let functional_role_analysis_result: FunctionalRoleAnalysisResult =
        functional_role_analysis_agent.prompt_typed(prompt).await?;
    info!(
        "functional_role_analysis_result: {:?}",
        functional_role_analysis_result
    );
    Ok(functional_role_analysis_result)
}

async fn execute_data_profiling(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    pruned_data_catalog: &Database,
    client: &Client<AnthropicExt>,
    functional_role_analysis_result: &FunctionalRoleAnalysisResult,
) -> Result<Vec<EmpericalObservation>> {
    let mut emperical_observations: Vec<EmpericalObservation> = Vec::new();
    for table_function in &functional_role_analysis_result.table_functions {
        let table =
            match pruned_data_catalog.get_table(&table_function.schema, &table_function.table) {
                Some(t) => t,
                None => continue,
            };
        let mut chat_history: Vec<Message> = Vec::new();

        let data_profiling_agent = client
            .agent("claude-haiku-4-5")
            .name("data_profiling_before_agent")
            .preamble(SystemPrompt::DataProfiling.as_str())
            .temperature(0.2)
            .max_tokens(2048)
            .build();
        let sql_dialect = sql_dialect_from_source_type(&connection.source_type);
        let prompt = UserPrompt::DataProfilingBefore {
            table_name: table_function.table.clone(),
            columns: table.columns_as_vec(),
            question: question.to_string(),
            semantic_role: table_function.table_function.clone(),
            sql_dialect,
        }
        .render();
        let exploration_queries: Vec<ExplorationQuery> = data_profiling_agent
            .prompt_typed(prompt)
            .with_history(&mut chat_history)
            .await?;

        let queries: Vec<&str> = exploration_queries.iter().map(|q| q.sql.as_str()).collect();
        let query_results = execute_sql_queries_in_parallel(connection, &queries).await;
        let observations: Vec<ExplorationQueryResult> = exploration_queries
            .iter()
            .zip(query_results)
            .map(|(eq, res)| {
                let sql_result = match res {
                    Ok(r) => serde_json::to_string(&r.data).unwrap_or_else(|_| "{}".to_string()),
                    Err(e) => e.to_string(),
                };
                ExplorationQueryResult {
                    sql: eq.sql.clone(),
                    sql_result,
                }
            })
            .collect();
        info!(
            "observations: {}",
            serde_json::to_string_pretty(&observations).unwrap_or_default()
        );

        let prompt = UserPrompt::DataProfilingAfter {
            table_name: table_function.table.clone(),
            observations: observations,
        }
        .render();

        let table_emperical_observation: EmpericalObservation = data_profiling_agent
            .prompt_typed(prompt)
            .with_history(&mut chat_history)
            .await?;

        info!(
            "table_emperical_observation: {}",
            serde_json::to_string_pretty(&table_emperical_observation).unwrap_or_default()
        );

        emperical_observations.push(table_emperical_observation);
    }
    Ok(emperical_observations)
}

pub(crate) async fn schema_linking_final_synthesis(
    connection: &entity::connection::Model,
    question: &str,
    functional_role_analysis_result: FunctionalRoleAnalysisResult,
    emperical_observations: &[EmpericalObservation],
    pruned_data_catalog: &Database,
    client: &Client<AnthropicExt>,
) -> Result<SchemaLinkingFinalSynthesisResponse> {
    let agent = client
        .agent("claude-haiku-4-5")
        .name("schema_linking_final_synthesis_agent")
        .preamble(SystemPrompt::SchemaLinkingFinalSynthesis.as_str())
        .temperature(0.2)
        .max_tokens(2048)
        .build();

    let schema_status = generate_schema_status(
        &functional_role_analysis_result.table_functions,
        emperical_observations,
        pruned_data_catalog,
    );

    let mut chat_history: Vec<Message> = Vec::new();
    let sql_dialect = sql_dialect_from_source_type(&connection.source_type);
    let prompt = UserPrompt::SchemaLinkingFinalSynthesisFirst {
        question: question.to_string(),
        semantic_analysis: functional_role_analysis_result,
        schema_status,
        max_refine_rounds: SCHEMA_LINKING_MAX_REFINE_ROUNDS,
        sql_dialect,
    }
    .render();
    let mut result = agent
        .prompt_typed::<SchemaLinkingFinalSynthesisResponse>(prompt)
        .with_history(&mut chat_history)
        .await?;

    for _round in 0..SCHEMA_LINKING_MAX_REFINE_ROUNDS {
        if result.status == "CONFIRM" {
            break;
        }
        let queries: Vec<&str> = result
            .exploration_queries
            .iter()
            .map(|s| s.as_str())
            .collect();
        let query_results = execute_sql_queries_in_parallel(connection, &queries).await;

        let results: String = result
            .exploration_queries
            .iter()
            .zip(query_results)
            .map(|(sql, res)| {
                let sql_result = match res {
                    Ok(r) => serde_json::to_string(&r.data).unwrap_or_else(|_| "{}".to_string()),
                    Err(e) => e.to_string(),
                };
                format!("SQL: {}\nResult: {}", sql, sql_result)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = UserPrompt::SchemaLinkingFinalSynthesisExplorationResults { results }.render();
        result = agent
            .prompt_typed::<SchemaLinkingFinalSynthesisResponse>(prompt)
            .with_history(&mut chat_history)
            .await?;
    }

    Ok(result)
}

pub(crate) async fn sql_generation(
    connection: &entity::connection::Model,
    client: &Client<AnthropicExt>,
    question: &str,
    master_plan: &str,
    linked_schema: SchemaLinkingFinalSynthesisResponse,
) -> Result<String> {
    let mut generated_sql = String::new();
    let agent = client
        .agent("claude-haiku-4-5")
        .name("sql_generation_agent")
        .preamble(SystemPrompt::SqlGeneration.as_str())
        .temperature(0.2)
        .max_tokens(2048)
        .build();
    let mut chat_history: Vec<Message> = Vec::new();
    let sql_dialect = sql_dialect_from_source_type(&connection.source_type);
    let mut prompt = UserPrompt::SqlGenerationFirst {
        question: question.to_string(),
        schema: serde_json::to_string(&SchemaLinkingFinalSynthesisForSql::from(&linked_schema))
            .unwrap_or_else(|_| "{}".to_string()),
        current_plan: master_plan.to_string(),
        sql_dialect,
    }
    .render();

    for round in 0..constants::SQL_GENERATION_ROUNDS {
        let result: SqlGenerationAction = agent
            .prompt_typed(&prompt)
            .with_history(&mut chat_history)
            .await?;
        match result.action {
            SqlGenerationActionType::Explore => {
                let explore_queries = result.explore_queries.unwrap_or_default();
                let queries: Vec<&str> =
                    explore_queries.iter().map(|res| res.sql.as_str()).collect();
                let exploration_results =
                    execute_sql_queries_in_parallel(connection, &queries).await;
                let results: String = queries
                    .iter()
                    .zip(exploration_results)
                    .map(|(sql, res)| {
                        let sql_result = match res {
                            Ok(r) => {
                                serde_json::to_string(&r.data).unwrap_or_else(|_| "{}".to_string())
                            }
                            Err(e) => e.to_string(),
                        };
                        format!("SQL: {}\nResult: {}", sql, sql_result)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                prompt = UserPrompt::SqlGenerationExplorationResult { results }.render();
            }
            SqlGenerationActionType::Refine => {
                let next_action = result
                    .next_action
                    .as_deref()
                    .map(|s| match s {
                        "GENERATE_SQL" => RefineNextAction::GenerateSql,
                        _ => RefineNextAction::Explore,
                    })
                    .unwrap_or(RefineNextAction::Explore);
                prompt = UserPrompt::SqlGenerationRefineResult { next_action }.render();
            }
            SqlGenerationActionType::GenerateSql => {
                let sql = result.sql.unwrap_or_default();
                let result = execute_sql_query(connection, sql.as_str()).await;
                let result = match result {
                    Ok(r) => serde_json::to_string(&r.data).unwrap_or_else(|_| "{}".to_string()),
                    Err(e) => e.to_string(),
                };

                prompt = UserPrompt::SqlGenerationQueryResult { result }.render();
                generated_sql = sql;
            }
            SqlGenerationActionType::Confirm => {
                break;
            }
        }
    }
    Ok(generated_sql)
}

fn format_candidate_plans_for_prompt(candidate_plans: Vec<String>) -> String {
    candidate_plans
        .into_iter()
        .enumerate()
        .map(|(i, r)| format!("plan{}: {}", i + 1, r))
        .collect::<Vec<_>>()
        .join("\n")
}

fn generate_schema_status(
    table_functions: &[TableFunction],
    emperical_observations: &[EmpericalObservation],
    pruned_data_catalog: &Database,
) -> String {
    let tables_status: Vec<SchemaStatusTable> = table_functions
        .iter()
        .zip(emperical_observations.iter())
        .filter_map(|(tf, obs)| {
            let table = pruned_data_catalog.get_table(&tf.schema, &tf.table)?;
            let observation_by_column: std::collections::HashMap<&str, _> = obs
                .relevant_columns
                .iter()
                .map(|c| (c.column_name.as_str(), c))
                .collect();
            let columns: Vec<SchemaStatusColumn> = table
                .columns_as_vec()
                .into_iter()
                .map(|col| {
                    let obs_col = observation_by_column.get(col.name.as_str());
                    SchemaStatusColumn {
                        name: col.name,
                        data_type: col.data_type,
                        description: col.description,
                        observations: obs_col.map(|c| c.observations.clone()).unwrap_or_default(),
                        relevance_reason: obs_col
                            .map(|c| c.relevance_reason.clone())
                            .unwrap_or_default(),
                    }
                })
                .collect();
            Some(SchemaStatusTable {
                table_name: tf.table.clone(),
                status: if obs.relevant {
                    "MARKED_RELEVANT".to_string()
                } else {
                    "MARKED_IRRELEVANT".to_string()
                },
                columns,
            })
        })
        .collect();

    tables_status
        .iter()
        .map(|t| serde_json::to_string(t).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

fn sql_dialect_from_source_type(source_type: &str) -> SqlDialect {
    SourceType::from_str(source_type)
        .ok()
        .and_then(|st| st.to_sql_dialect())
        .unwrap()
}
