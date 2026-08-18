use super::constants::{
    DATA_CATALOG_BATCHES, DATA_PROFILING_CONCURRENCY, HYPOTHESIS_GENERATION_CONCURRENCY,
    LOGICAL_PLAN_HYPOTHESIS_GENERATION_ROUNDS, SCHEMA_LINKING_MAX_REFINE_ROUNDS,
    SEARCH_SPACE_REDUCTION_CONCURRENCY, SQL_GENERATION_ROUNDS,
};
use super::system_prompt::SystemPrompt;
use super::user_prompt::UserPrompt;
use crate::ai::agent_utils::{AgentName, ModelRole};
use crate::ai::dto::{
    DeletionSet, EmpericalObservation, Event, EventChannels, ExplorationQuery,
    ExplorationQueryResult, FunctionalRoleAnalysisResult, LogicalPlanningResponse,
    RefineNextAction, SchemaLinkingFinalSynthesisForSql, SchemaLinkingFinalSynthesisResponse,
    SchemaStatusColumn, SchemaStatusTable, SelectionSet, SqlGenerationAction,
    SqlGenerationActionType, TableFunction,
};
use crate::common::AppState;
use crate::connection::constants::{SourceType, SqlDialect};
use crate::data_catalog::dto::Database;
use crate::data_catalog::{
    get_data_catalog_batches, join_pruned_batches, prune_data_catalog_batch,
};
use crate::database::service::{execute_sql_queries_in_parallel, execute_sql_query};
use crate::entity;
use anyhow::Result;
use futures_util::StreamExt;
use rig::message::Message;
use tracing::info;

pub async fn execute(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    channels: &EventChannels,
) -> Result<String> {
    let (master_plan, linked_schema) =
        execute_schema_linking(app_state, connection, question, channels).await?;
    channels.send(Event::GeneratingSql).await?;
    let generated_sql =
        sql_generation(app_state, connection, question, &master_plan, linked_schema).await?;
    info!("generated_sql: {}", generated_sql);
    channels
        .send(Event::GeneratedSql {
            sql: generated_sql.clone(),
        })
        .await?;
    Ok(generated_sql)
}

pub async fn execute_schema_linking(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    channels: &EventChannels,
) -> Result<(String, SchemaLinkingFinalSynthesisResponse)> {
    let (data_catalog_batches, master_plan) = tokio::try_join!(
        async {
            get_data_catalog_batches(app_state, connection, DATA_CATALOG_BATCHES)
                .await
                .map_err(anyhow::Error::from)
        },
        async {
            let candidate_hypothesis =
                execute_hypothesis_generation(app_state, connection, question, channels).await?;
            channels.send(Event::IntegratingCandidatePlans).await?;
            let master_plan = execute_integrate_candidate_plans(
                app_state,
                connection,
                question,
                candidate_hypothesis,
            )
            .await?;
            channels
                .send(Event::IntegratedCandidatePlans {
                    master_plan: master_plan.clone(),
                })
                .await?;
            Ok(master_plan)
        }
    )?;

    info!("master_plan: {master_plan}");

    channels.send(Event::ExecutingSearchSpaceReduction).await?;
    let pruned_data_catalog = execute_search_space_reduction(
        app_state,
        connection,
        question,
        &master_plan,
        data_catalog_batches,
    )
    .await?;
    channels
        .send(Event::ExecutedSearchSpaceReduction {
            pruned_data_catalog: pruned_data_catalog.clone(),
        })
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
            channels,
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

    channels
        .send(Event::ExecutingSchemaLinkingSynthesis)
        .await?;
    let linked_schema = schema_linking_final_synthesis(
        app_state,
        connection,
        question,
        functional_role_analysis_result,
        &emperical_observations,
        &pruned_data_catalog,
    )
    .await?;
    channels
        .send(Event::ExecutedSchemaLinkingSynthesis {
            linked_schema: linked_schema.clone(),
        })
        .await?;
    info!(
        "linked_schema: {}",
        serde_json::to_string_pretty(&linked_schema).unwrap_or_default()
    );
    Ok((master_plan, linked_schema))
}

async fn execute_hypothesis_generation(
    app_state: &AppState,
    _connection: &entity::connection::Model,
    question: &str,
    channels: &EventChannels,
) -> Result<Vec<LogicalPlanningResponse>> {
    channels.send(Event::GeneratingCandidateHypothesis).await?;
    let model_client = app_state.require_model_client().await?;

    let generation_results: Vec<Result<LogicalPlanningResponse, _>> =
        futures_util::stream::iter((0..LOGICAL_PLAN_HYPOTHESIS_GENERATION_ROUNDS).map(|_round| {
            let model_client = model_client.clone();
            let question = question.to_string();
            async move {
                let hypothesis_generation_agent = model_client.build_agent(
                    ModelRole::Default,
                    AgentName::HypothesisGenerationAgent,
                    SystemPrompt::LogicalPlanning.as_str(),
                    0.8,
                    2048,
                );
                let prompt = UserPrompt::LogicalPlanningHypothesisGeneration { question }.render();
                hypothesis_generation_agent
                    .prompt_typed::<LogicalPlanningResponse>(prompt)
                    .await
            }
        }))
        .buffered(HYPOTHESIS_GENERATION_CONCURRENCY)
        .collect()
        .await;

    let candidate_hypothesis: Vec<LogicalPlanningResponse> = generation_results
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    let hypothesis_for_event: Vec<String> = candidate_hypothesis
        .iter()
        .map(|p| p.logical_steps.join("\n"))
        .collect();
    channels
        .send(Event::GeneratedCandidateHypothesis(hypothesis_for_event))
        .await?;
    Ok(candidate_hypothesis)
}

async fn execute_integrate_candidate_plans(
    app_state: &AppState,
    _connection: &entity::connection::Model,
    question: &str,
    candidate_hypothesis: Vec<LogicalPlanningResponse>,
) -> Result<String> {
    let model_client = app_state.require_model_client().await?;
    let integrate_candidate_plan_agent = model_client.build_agent(
        ModelRole::Default,
        AgentName::IntegrateCandidatePlanAgent,
        SystemPrompt::AggregatingPlanCandidates.as_str(),
        0.2,
        2048,
    );
    let candidate_plans_str = format_candidate_plans_for_prompt(candidate_hypothesis);
    let prompt = UserPrompt::AggregatingPlanCandidates {
        question: question.to_string(),
        candidate_plans: candidate_plans_str,
    }
    .render();
    let master_plan = integrate_candidate_plan_agent.prompt(&prompt).await?;
    Ok(master_plan)
}

pub(crate) async fn execute_search_space_reduction(
    app_state: &AppState,
    _connection: &entity::connection::Model,
    question: &str,
    master_plan: &str,
    data_catalog_batches: Vec<Database>,
) -> Result<Database> {
    info!("initial data_catalog_batches: {:?}", data_catalog_batches);
    let pruning_results: Vec<Result<Database>> =
        futures_util::stream::iter(data_catalog_batches.into_iter().map(
            |mut data_catalog_batch| {
                let model_client = app_state.require_model_client();
                let question = question.to_string();
                let master_plan = master_plan.to_string();
                async move {
                    let model_client = model_client.await?;
                    let data_catalog_deletion_agent = model_client.build_agent(
                        ModelRole::Default,
                        AgentName::DataCatalogDeletionAgent,
                        SystemPrompt::IdentifyingDeletionSet.as_str(),
                        0.2,
                        2048,
                    );
                    let data_catalog_selection_agent = model_client.build_agent(
                        ModelRole::Default,
                        AgentName::DataCatalogSelectionAgent,
                        SystemPrompt::IdentifyingSelectionSet.as_str(),
                        0.2,
                        2048,
                    );
                    let data_catalog_json =
                        data_catalog_batch.to_display_string().unwrap_or_default();
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
                    Ok(data_catalog_batch)
                }
            },
        ))
        .buffered(SEARCH_SPACE_REDUCTION_CONCURRENCY)
        .collect()
        .await;

    let pruned_batches: Vec<Database> =
        pruning_results.into_iter().collect::<Result<Vec<_>>>()?;
    let pruned_data_catalog = join_pruned_batches(pruned_batches);
    Ok(pruned_data_catalog)
}

pub(crate) async fn execute_hypothesis_verification(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    master_plan: &str,
    pruned_data_catalog: &Database,
    channels: &EventChannels,
) -> Result<(Vec<(TableFunction, EmpericalObservation)>, FunctionalRoleAnalysisResult)> {
    channels
        .send(Event::ExecutingFunctionalRoleAnalysis)
        .await?;
    let functional_role_analysis_result = execute_functional_role_analysis(
        app_state,
        connection,
        question,
        master_plan,
        pruned_data_catalog,
    )
    .await?;
    channels
        .send(Event::ExecutedFunctionalRoleAnalysis {
            functional_role_analysis_result: functional_role_analysis_result.clone(),
        })
        .await?;
    channels.send(Event::ExecutingDataProfiling).await?;
    let emperical_observations = execute_data_profiling(
        app_state,
        connection,
        question,
        pruned_data_catalog,
        &functional_role_analysis_result,
    )
    .await?;
    channels
        .send(Event::ExecutedDataProfiling {
            emperical_observations: emperical_observations
                .iter()
                .map(|(_, observation)| observation.clone())
                .collect(),
        })
        .await?;
    Ok((emperical_observations, functional_role_analysis_result))
}

/// Functional role analysis does not return tables irrelevant for further analysis.
async fn execute_functional_role_analysis(
    app_state: &AppState,
    _connection: &entity::connection::Model,
    question: &str,
    master_plan: &str,
    pruned_data_catalog: &Database,
) -> Result<FunctionalRoleAnalysisResult> {
    let model_client = app_state.require_model_client().await?;
    let functional_role_analysis_agent = model_client.build_agent(
        ModelRole::Default,
        AgentName::FunctionalRoleAnalysisAgent,
        SystemPrompt::SemanticLinking.as_str(),
        0.2,
        2048,
    );
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
    functional_role_analysis_result: &FunctionalRoleAnalysisResult,
) -> Result<Vec<(TableFunction, EmpericalObservation)>> {
    let profiling_targets: Vec<(TableFunction, Vec<crate::data_catalog::dto::Column>)> =
        functional_role_analysis_result
            .table_functions
            .iter()
            .filter_map(|table_function| {
                match pruned_data_catalog.get_table(&table_function.schema, &table_function.table) {
                    Some(table) => Some((table_function.clone(), table.columns_as_vec())),
                    None => {
                        tracing::warn!(
                            "data profiling: table {}.{} from functional role analysis is absent from the pruned data catalog; skipping",
                            table_function.schema,
                            table_function.table
                        );
                        None
                    }
                }
            })
            .collect();

    let profiling_results: Vec<Result<(TableFunction, EmpericalObservation)>> =
        futures_util::stream::iter(profiling_targets.into_iter().map(
            |(table_function, table_columns)| async move {
                let mut chat_history: Vec<Message> = Vec::new();

                let model_client = app_state.require_model_client().await?;
                let data_profiling_agent = model_client.build_agent(
                    ModelRole::Default,
                    AgentName::DataProfilingBeforeAgent,
                    SystemPrompt::DataProfiling.as_str(),
                    0.2,
                    2048,
                );
                let sql_dialect = sql_dialect_from_source_type(&connection.source_type);
                let prompt = UserPrompt::DataProfilingBefore {
                    table_name: table_function.table.clone(),
                    columns: table_columns,
                    question: question.to_string(),
                    semantic_role: table_function.table_function.clone(),
                    sql_dialect,
                }
                .render();
                let exploration_queries: Vec<ExplorationQuery> = data_profiling_agent
                    .prompt_typed_with_history(prompt, &mut chat_history)
                    .await?;

                let queries: Vec<&str> =
                    exploration_queries.iter().map(|q| q.sql.as_str()).collect();
                let query_results =
                    execute_sql_queries_in_parallel(app_state, connection, &queries).await;
                let observations: Vec<ExplorationQueryResult> = exploration_queries
                    .iter()
                    .zip(query_results)
                    .map(|(exploration_query, query_result)| {
                        let sql_result = match query_result {
                            Ok(result) => serde_json::to_string(&result.data)
                                .unwrap_or_else(|_| "{}".to_string()),
                            Err(error) => error.to_string(),
                        };
                        ExplorationQueryResult {
                            sql: exploration_query.sql.clone(),
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
                    observations,
                }
                .render();

                let table_emperical_observation: EmpericalObservation = data_profiling_agent
                    .prompt_typed_with_history(prompt, &mut chat_history)
                    .await?;

                info!(
                    "table_emperical_observation: {}",
                    serde_json::to_string_pretty(&table_emperical_observation).unwrap_or_default()
                );

                Ok((table_function.clone(), table_emperical_observation))
            },
        ))
        .buffered(DATA_PROFILING_CONCURRENCY)
        .collect()
        .await;

    profiling_results.into_iter().collect::<Result<Vec<_>>>()
}

pub(crate) async fn schema_linking_final_synthesis(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    functional_role_analysis_result: FunctionalRoleAnalysisResult,
    emperical_observations: &[(TableFunction, EmpericalObservation)],
    pruned_data_catalog: &Database,
) -> Result<SchemaLinkingFinalSynthesisResponse> {
    info!(
        "schema_linking_final_synthesis: starting (tables={}, empirical_observations={})",
        functional_role_analysis_result.table_functions.len(),
        emperical_observations.len()
    );

    let model_client = app_state.require_model_client().await?;
    let agent = model_client.build_agent(
        ModelRole::Default,
        AgentName::SchemaLinkingFinalSynthesisAgent,
        SystemPrompt::SchemaLinkingFinalSynthesis.as_str(),
        0.2,
        16384,
    );

    let schema_status = generate_schema_status(emperical_observations, pruned_data_catalog);
    info!(
        "schema_linking_final_synthesis: schema_status ready (chars={})",
        schema_status.len()
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
    info!("schema_linking_final_synthesis: calling model for initial synthesis");
    let mut result = agent
        .prompt_typed_with_history::<SchemaLinkingFinalSynthesisResponse>(prompt, &mut chat_history)
        .await?;
    info!(
        "schema_linking_final_synthesis: initial response status={}, exploration_queries={}, refined_schema_tables={}, rejected_candidates={}",
        result.status,
        result.exploration_queries.len(),
        result.refined_schema.len(),
        result.rejected_candidates.len()
    );

    for round in 0..SCHEMA_LINKING_MAX_REFINE_ROUNDS {
        if result.status == "CONFIRM" {
            info!("schema_linking_final_synthesis: CONFIRM, stopping refinement");
            break;
        }
        let queries: Vec<&str> = result
            .exploration_queries
            .iter()
            .map(|s| s.as_str())
            .collect();
        info!(
            "schema_linking_final_synthesis: refine round {}/{} executing {} exploration queries",
            round + 1,
            SCHEMA_LINKING_MAX_REFINE_ROUNDS,
            queries.len()
        );
        let query_results =
            execute_sql_queries_in_parallel(app_state, connection, &queries).await;

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
        info!(
            "schema_linking_final_synthesis: calling model after refine round {}",
            round + 1
        );
        result = agent
            .prompt_typed_with_history::<SchemaLinkingFinalSynthesisResponse>(
                prompt,
                &mut chat_history,
            )
            .await?;
        info!(
            "schema_linking_final_synthesis: after refine round {} status={}, exploration_queries={}",
            round + 1,
            result.status,
            result.exploration_queries.len()
        );
    }

    if result.status == "CONFIRM" {
        info!("schema_linking_final_synthesis: finished with CONFIRM");
    } else {
        info!(
            "schema_linking_final_synthesis: finished after {} refine rounds without CONFIRM (status={})",
            SCHEMA_LINKING_MAX_REFINE_ROUNDS,
            result.status
        );
    }

    Ok(result)
}

pub(crate) async fn sql_generation(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    master_plan: &str,
    linked_schema: SchemaLinkingFinalSynthesisResponse,
) -> Result<String> {
    let mut generated_sql = String::new();
    let model_client = app_state.require_model_client().await?;
    let agent = model_client.build_agent(
        ModelRole::Default,
        AgentName::SqlGenerationAgent,
        SystemPrompt::SqlGeneration.as_str(),
        0.2,
        2048,
    );
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

    for _round in 0..SQL_GENERATION_ROUNDS {
        let result: SqlGenerationAction = agent
            .prompt_typed_with_history(&prompt, &mut chat_history)
            .await?;
        match result.action {
            SqlGenerationActionType::Explore => {
                let explore_queries = result.explore_queries.unwrap_or_default();
                let queries: Vec<&str> =
                    explore_queries.iter().map(|res| res.sql.as_str()).collect();
                let exploration_results =
                    execute_sql_queries_in_parallel(app_state, connection, &queries).await;
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
                let result = execute_sql_query(app_state, connection, sql.as_str()).await;
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

fn format_candidate_plans_for_prompt(candidate_plans: Vec<LogicalPlanningResponse>) -> String {
    candidate_plans
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let body = r.logical_steps.join("\n");
            format!("plan{}:\n{}", i + 1, body)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn generate_schema_status(
    emperical_observations: &[(TableFunction, EmpericalObservation)],
    pruned_data_catalog: &Database,
) -> String {
    let tables_status: Vec<SchemaStatusTable> = emperical_observations
        .iter()
        .filter_map(|(table_function, observation)| {
            let table =
                pruned_data_catalog.get_table(&table_function.schema, &table_function.table)?;
            let observation_by_column: std::collections::HashMap<&str, _> = observation
                .relevant_columns
                .iter()
                .map(|column| (column.column_name.as_str(), column))
                .collect();
            let columns: Vec<SchemaStatusColumn> = table
                .columns_as_vec()
                .into_iter()
                .map(|column| {
                    let observed_column = observation_by_column.get(column.name.as_str());
                    SchemaStatusColumn {
                        name: column.name,
                        data_type: column.data_type,
                        description: column.description,
                        observations: observed_column
                            .map(|c| c.observations.clone())
                            .unwrap_or_default(),
                        relevance_reason: observed_column
                            .map(|c| c.relevance_reason.clone())
                            .unwrap_or_default(),
                    }
                })
                .collect();
            Some(SchemaStatusTable {
                table_name: table_function.table.clone(),
                status: if observation.relevant {
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
        .map(|table_status| serde_json::to_string(table_status).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

fn sql_dialect_from_source_type(source_type: &str) -> SqlDialect {
    SourceType::from_str(source_type)
        .ok()
        .and_then(|st| st.to_sql_dialect())
        .unwrap()
}
