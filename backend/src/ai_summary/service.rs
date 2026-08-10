use crate::ai::chart_summary_agent;
use crate::ai::dashboard_summary_agent;
use crate::ai::dto::{ChartSummaryInput, DashboardSummaryInput};
use crate::ai_summary::constants::{self, SummaryEntityType};
use crate::ai_summary::dto::{AiSummaryDto, DashboardSummariesDto};
use crate::ai_summary::repository;
use crate::cell::dto::ChartExecution;
use crate::cell::service as cell_service;
use crate::common::time_utils;
use crate::common::AppState;
use crate::dashboard::service as dashboard_service;
use crate::database::constants::QueryResult;
use sea_orm::prelude::DateTimeWithTimeZone;
use crate::entity;
use futures_util::stream::{self, StreamExt};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr};
use tracing::{error, info};
use uuid::Uuid;

const UNTITLED_CHART: &str = "untitled chart";
const MAX_STORED_ERROR_LENGTH: usize = 500;

#[derive(Debug, thiserror::Error)]
pub enum SummaryError {
    #[error("Not found")]
    NotFound,
    #[error("Dashboard has no charts")]
    NoCharts,
    #[error("AI is not configured")]
    AiNotConfigured,
    #[error("Prompt must be at most {0} characters")]
    PromptTooLong(usize),
    #[error("Database error: {0}")]
    Repository(#[from] DbErr),
}

pub enum GenerateOutcome {
    Generated(AiSummaryDto),
    AlreadyGenerating(AiSummaryDto),
}

enum GenerationClaim {
    Started { user_prompt: Option<String> },
    InProgress(Box<entity::ai_summary::Model>),
}

pub async fn get_dashboard_summaries(
    app_state: &AppState,
    dashboard_id: Uuid,
) -> Result<DashboardSummariesDto, SummaryError> {
    let dashboard = dashboard_service::get_dashboard_entity(app_state, dashboard_id)
        .await
        .map_err(map_not_found)?;
    let cell_ids = dashboard_service::dashboard_chart_cell_ids(&dashboard);
    let rows =
        repository::find_for_dashboard(&app_state.database_connection, dashboard_id, &cell_ids)
            .await?;

    let mut dashboard_summary = None;
    let mut charts = Vec::new();
    for row in rows {
        match SummaryEntityType::from_str(&row.entity_type) {
            Ok(SummaryEntityType::Dashboard) => dashboard_summary = Some(AiSummaryDto::from(row)),
            _ => charts.push(AiSummaryDto::from(row)),
        }
    }

    Ok(DashboardSummariesDto {
        dashboard: dashboard_summary,
        charts,
    })
}

pub async fn generate_dashboard_summary(
    app_state: &AppState,
    dashboard_id: Uuid,
) -> Result<GenerateOutcome, SummaryError> {
    require_ai(app_state).await?;

    let dashboard = dashboard_service::get_dashboard_entity(app_state, dashboard_id)
        .await
        .map_err(map_not_found)?;
    let cell_ids = dashboard_service::dashboard_chart_cell_ids(&dashboard);
    if cell_ids.is_empty() {
        return Err(SummaryError::NoCharts);
    }

    let entity_type = SummaryEntityType::Dashboard;
    let user_prompt = match try_start_generation(app_state, entity_type, dashboard_id).await? {
        GenerationClaim::InProgress(row) => {
            info!("ai_summary: dashboard {dashboard_id} is already generating; ignoring request");
            return Ok(GenerateOutcome::AlreadyGenerating(AiSummaryDto::from(*row)));
        }
        GenerationClaim::Started { user_prompt } => user_prompt,
    };

    info!(
        "ai_summary: dashboard {dashboard_id} starting summary over {} chart(s)",
        cell_ids.len()
    );

    let outcome = match summarize_dashboard(app_state, dashboard_id, &cell_ids, &user_prompt).await
    {
        Ok((summary, count)) => {
            info!("ai_summary: dashboard {dashboard_id} summarised {count} chart(s)");
            Ok((summary, Some(count)))
        }
        Err(e) => {
            error!("ai_summary: dashboard {dashboard_id} generation failed: {e:#}");
            Err(truncate_error(e))
        }
    };

    Ok(GenerateOutcome::Generated(
        record_generation_result(app_state, entity_type, dashboard_id, outcome, user_prompt)
            .await?,
    ))
}

pub async fn generate_all_chart_summaries(
    app_state: &AppState,
    dashboard_id: Uuid,
) -> Result<Vec<AiSummaryDto>, SummaryError> {
    require_ai(app_state).await?;

    let dashboard = dashboard_service::get_dashboard_entity(app_state, dashboard_id)
        .await
        .map_err(map_not_found)?;
    let cell_ids = dashboard_service::dashboard_chart_cell_ids(&dashboard);
    if cell_ids.is_empty() {
        return Err(SummaryError::NoCharts);
    }

    let mut claimed = Vec::new();
    let mut rows = Vec::new();
    for cell_id in cell_ids {
        match try_start_generation(app_state, SummaryEntityType::ChartCell, cell_id).await? {
            GenerationClaim::Started { user_prompt } => claimed.push((cell_id, user_prompt)),
            GenerationClaim::InProgress(row) => rows.push(AiSummaryDto::from(*row)),
        }
    }

    info!(
        "ai_summary: dashboard {dashboard_id} queued {} chart(s), {} already generating",
        claimed.len(),
        rows.len()
    );

    for (cell_id, _) in &claimed {
        if let Some(row) = repository::find(
            &app_state.database_connection,
            SummaryEntityType::ChartCell,
            *cell_id,
        )
        .await?
        {
            rows.push(AiSummaryDto::from(row));
        }
    }

    spawn_chart_summary_batch(app_state.clone(), dashboard_id, claimed);

    Ok(rows)
}

fn spawn_chart_summary_batch(
    app_state: AppState,
    dashboard_id: Uuid,
    claimed: Vec<(Uuid, Option<String>)>,
) {
    if claimed.is_empty() {
        return;
    }
    tokio::spawn(async move {
        stream::iter(claimed.into_iter().map(|(cell_id, user_prompt)| {
            let app_state = &app_state;
            async move {
                let outcome = match summarize_chart(app_state, cell_id, &user_prompt).await {
                    Ok(summary) => Ok((summary, None)),
                    Err(e) => {
                        error!("ai_summary: chart {cell_id} generation failed: {e:#}");
                        Err(truncate_error(e))
                    }
                };
                if let Err(e) = record_generation_result(
                    app_state,
                    SummaryEntityType::ChartCell,
                    cell_id,
                    outcome,
                    user_prompt,
                )
                .await
                {
                    error!("ai_summary: chart {cell_id} result could not be stored: {e}");
                }
            }
        }))
        .buffer_unordered(constants::GENERATION_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

        info!("ai_summary: dashboard {dashboard_id} finished background chart summaries");
    });
}

pub async fn generate_chart_summary(
    app_state: &AppState,
    cell_id: Uuid,
) -> Result<GenerateOutcome, SummaryError> {
    require_ai(app_state).await?;

    if !cell_service::does_cell_exist(app_state, cell_id).await {
        return Err(SummaryError::NotFound);
    }

    let entity_type = SummaryEntityType::ChartCell;
    let user_prompt = match try_start_generation(app_state, entity_type, cell_id).await? {
        GenerationClaim::InProgress(row) => {
            info!("ai_summary: chart {cell_id} is already generating; ignoring request");
            return Ok(GenerateOutcome::AlreadyGenerating(AiSummaryDto::from(*row)));
        }
        GenerationClaim::Started { user_prompt } => user_prompt,
    };

    info!("ai_summary: chart {cell_id} starting summary");

    let outcome = match summarize_chart(app_state, cell_id, &user_prompt).await {
        Ok(summary) => {
            info!("ai_summary: chart {cell_id} summarised");
            Ok((summary, None))
        }
        Err(e) => {
            error!("ai_summary: chart {cell_id} generation failed: {e:#}");
            Err(truncate_error(e))
        }
    };

    Ok(GenerateOutcome::Generated(
        record_generation_result(app_state, entity_type, cell_id, outcome, user_prompt).await?,
    ))
}

pub async fn set_dashboard_prompt(
    app_state: &AppState,
    dashboard_id: Uuid,
    user_prompt: Option<String>,
) -> Result<AiSummaryDto, SummaryError> {
    dashboard_service::get_dashboard_entity(app_state, dashboard_id)
        .await
        .map_err(map_not_found)?;
    set_prompt(app_state, SummaryEntityType::Dashboard, dashboard_id, user_prompt).await
}

pub async fn set_chart_prompt(
    app_state: &AppState,
    cell_id: Uuid,
    user_prompt: Option<String>,
) -> Result<AiSummaryDto, SummaryError> {
    if !cell_service::does_cell_exist(app_state, cell_id).await {
        return Err(SummaryError::NotFound);
    }
    set_prompt(app_state, SummaryEntityType::ChartCell, cell_id, user_prompt).await
}

async fn set_prompt(
    app_state: &AppState,
    entity_type: SummaryEntityType,
    entity_id: Uuid,
    user_prompt: Option<String>,
) -> Result<AiSummaryDto, SummaryError> {
    let user_prompt = normalise_prompt(user_prompt)?;
    let row = repository::set_user_prompt(
        &app_state.database_connection,
        entity_type,
        entity_id,
        user_prompt,
    )
    .await?;
    info!("ai_summary: {entity_type} {entity_id} prompt updated");
    Ok(AiSummaryDto::from(row))
}

fn normalise_prompt(user_prompt: Option<String>) -> Result<Option<String>, SummaryError> {
    let Some(prompt) = user_prompt else {
        return Ok(None);
    };
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > constants::MAX_USER_PROMPT_LENGTH {
        return Err(SummaryError::PromptTooLong(
            constants::MAX_USER_PROMPT_LENGTH,
        ));
    }
    Ok(Some(trimmed.to_string()))
}

pub async fn delete_chart_summary(db: &impl ConnectionTrait, cell_id: Uuid) -> Result<(), DbErr> {
    repository::delete_for_entity(db, SummaryEntityType::ChartCell, cell_id).await
}

pub async fn delete_chart_summaries(
    db: &impl ConnectionTrait,
    cell_ids: &[Uuid],
) -> Result<(), DbErr> {
    repository::delete_for_entities(db, SummaryEntityType::ChartCell, cell_ids).await
}

pub async fn delete_dashboard_summary(
    db: &impl ConnectionTrait,
    dashboard_id: Uuid,
) -> Result<(), DbErr> {
    repository::delete_for_entity(db, SummaryEntityType::Dashboard, dashboard_id).await
}

async fn require_ai(app_state: &AppState) -> Result<(), SummaryError> {
    if app_state.current_model_client().await.is_none() {
        return Err(SummaryError::AiNotConfigured);
    }
    Ok(())
}

fn map_not_found(err: DbErr) -> SummaryError {
    match err {
        DbErr::RecordNotFound(_) => SummaryError::NotFound,
        other => SummaryError::Repository(other),
    }
}

async fn try_start_generation(
    app_state: &AppState,
    entity_type: SummaryEntityType,
    entity_id: Uuid,
) -> Result<GenerationClaim, SummaryError> {
    let db = &app_state.database_connection;
    let stale_before: DateTimeWithTimeZone = (time_utils::now_utc()
        - chrono::Duration::minutes(constants::STALE_GENERATION_TIMEOUT_MINUTES))
    .into();

    // The conditional update is the claim itself: it tests "not already generating" and takes
    // ownership in one statement, so no competing request can slip between the two.
    if repository::try_mark_generating(db, entity_type, entity_id, stale_before).await? > 0 {
        // Snapshot the prompt after the claim commits, so a concurrent edit cannot change what
        // this run is recorded as having used.
        let user_prompt = repository::find(db, entity_type, entity_id)
            .await?
            .and_then(|row| row.user_prompt);
        return Ok(GenerationClaim::Started { user_prompt });
    }

    // Updating nothing means either a row exists and is actively generating, or none exists yet.
    match repository::find(db, entity_type, entity_id).await? {
        Some(row) => Ok(GenerationClaim::InProgress(Box::new(row))),
        None => insert_generation_claim(db, entity_type, entity_id).await,
    }
}

async fn insert_generation_claim(
    db: &DatabaseConnection,
    entity_type: SummaryEntityType,
    entity_id: Uuid,
) -> Result<GenerationClaim, SummaryError> {
    match repository::insert_generating(db, entity_type, entity_id).await {
        Ok(row) => Ok(GenerationClaim::Started {
            user_prompt: row.user_prompt,
        }),
        Err(e) => match repository::find(db, entity_type, entity_id).await? {
            Some(row) => Ok(GenerationClaim::InProgress(Box::new(row))),
            None => Err(SummaryError::Repository(e)),
        },
    }
}

async fn record_generation_result(
    app_state: &AppState,
    entity_type: SummaryEntityType,
    entity_id: Uuid,
    outcome: Result<(String, Option<i32>), String>,
    user_prompt_used: Option<String>,
) -> Result<AiSummaryDto, SummaryError> {
    let db = &app_state.database_connection;
    let row = match outcome {
        Ok((summary_text, summarized_entity_count)) => {
            repository::mark_ready(
                db,
                entity_type,
                entity_id,
                summary_text,
                summarized_entity_count,
                user_prompt_used,
            )
            .await?
        }
        Err(message) => repository::mark_failed(db, entity_type, entity_id, message).await?,
    };
    row.map(AiSummaryDto::from).ok_or(SummaryError::NotFound)
}

async fn summarize_dashboard(
    app_state: &AppState,
    dashboard_id: Uuid,
    cell_ids: &[Uuid],
    user_prompt: &Option<String>,
) -> anyhow::Result<(String, i32)> {
    let outcomes = cell_service::execute_chart_cells_data(
        app_state,
        cell_ids,
        constants::GENERATION_CONCURRENCY,
    )
    .await?;

    let mut executions = Vec::new();
    let mut skipped_chart_names = Vec::new();
    for outcome in outcomes {
        match outcome.result {
            Ok(execution) => executions.push(execution),
            Err(e) => {
                error!(
                    "ai_summary: chart {} could not be loaded: {e}",
                    outcome.cell_id
                );
                skipped_chart_names.push(chart_label(outcome.chart_name));
            }
        }
    }

    if executions.is_empty() {
        anyhow::bail!("no chart on this dashboard could be loaded");
    }

    let row_limit = constants::rows_per_chart(executions.len());
    info!(
        "ai_summary: dashboard {dashboard_id} loaded {} chart(s), skipped {}, sampling {row_limit} row(s) each",
        executions.len(),
        skipped_chart_names.len()
    );

    let charts: Vec<ChartSummaryInput> = executions
        .into_iter()
        .map(|execution| to_summary_input(execution, row_limit))
        .collect();
    let summarized_chart_count = charts.len() as i32;

    let input = DashboardSummaryInput {
        charts,
        skipped_chart_names,
        user_prompt: user_prompt.clone(),
    };

    let summary = dashboard_summary_agent::summarize_dashboard(app_state, &input).await?;
    Ok((summary, summarized_chart_count))
}

async fn summarize_chart(
    app_state: &AppState,
    cell_id: Uuid,
    user_prompt: &Option<String>,
) -> anyhow::Result<String> {
    let execution = cell_service::execute_chart_cell_data(app_state, cell_id).await?;
    let input = to_summary_input(execution, constants::CHART_SUMMARY_ROW_LIMIT);
    chart_summary_agent::summarize_chart(app_state, &input, user_prompt).await
}

fn to_summary_input(execution: ChartExecution, row_limit: usize) -> ChartSummaryInput {
    let ChartExecution {
        chart_name,
        chart_type,
        field_names,
        result,
        ..
    } = execution;
    let QueryResult { columns, data } = result;
    let total_row_count = data.len();

    ChartSummaryInput {
        chart_name: chart_label(chart_name),
        chart_type,
        field_names,
        columns,
        rows: data.into_iter().take(row_limit).collect(),
        total_row_count,
    }
}

fn truncate_error(err: anyhow::Error) -> String {
    let message = err.to_string();
    match message.char_indices().nth(MAX_STORED_ERROR_LENGTH) {
        Some((index, _)) => format!("{}…", &message[..index]),
        None => message,
    }
}

fn chart_label(chart_name: String) -> String {
    if chart_name.trim().is_empty() {
        UNTITLED_CHART.to_string()
    } else {
        chart_name
    }
}
