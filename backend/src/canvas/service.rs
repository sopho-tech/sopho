use crate::ai::dto::{
    CanvasCandidate, CanvasCandidateCell, CanvasOp, CanvasPlan, PlannedAxisChart, PlannedChart,
    PlannedChartSpec,
};
use crate::canvas::constants::CanvasStatus;
use crate::canvas::dto;
use crate::canvas::repository;
use crate::cell::constants::{CellType, ChartType};
use crate::cell::dto::{
    series_alias, AxisChartContent, ChartContent, ChartSeries, CreateCellDto, MetricChartContent,
    PieChartContent,
};
use crate::cell::service as cell_service;
use crate::common::time_utils;
use crate::common::{AppState, PaginatedResponse, Pagination};
use crate::dashboard::service::{self as dashboard_service, DashboardChartRequest};
use crate::entity;
use crate::notebook::service as notebook_service;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::TransactionTrait;
use std::collections::HashMap;
use uuid::Uuid;

pub async fn get_canvas(app_state: AppState, id: Uuid) -> impl IntoResponse {
    let canvas = repository::get_canvas(&app_state.database_connection, id).await;
    match canvas {
        Ok(canvas) => {
            let response_dto = dto::CanvasDto::from(canvas);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        }
        Err(e) => match e {
            sea_orm::DbErr::RecordNotFound(_) => (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({ "error": "Canvas not found" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            ),
        },
    }
}

pub async fn get_all_canvases(
    app_state: AppState,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    let page = pagination.page();
    let page_size = pagination.page_size();
    let result =
        repository::get_paginated_canvases(&app_state.database_connection, page, page_size).await;

    match result {
        Ok((canvases, total_items, total_pages)) => {
            let mut response_dto_list: Vec<dto::CanvasDto> = Vec::new();

            for canvas in canvases.iter() {
                let mut canvas_dto = dto::CanvasDto::from(canvas.clone());

                let cell_counts_result =
                    cell_service::get_cell_counts_by_canvas_id(&app_state, canvas.id).await;
                if let Ok((sql_count, chart_count)) = cell_counts_result {
                    canvas_dto.sql_cell_count = sql_count;
                    canvas_dto.chart_cell_count = chart_count;
                }

                let dashboard_charts_result =
                    dashboard_service::get_dashboard_charts_count_by_canvas_id(
                        &app_state, canvas.id,
                    )
                    .await;
                if let Ok(dashboard_charts_count) = dashboard_charts_result {
                    canvas_dto.dashboard_charts_count = dashboard_charts_count;
                }

                response_dto_list.push(canvas_dto);
            }

            let paginated_response = PaginatedResponse {
                data: response_dto_list,
                total_pages,
                page,
                page_size,
                total_items,
            };

            (
                StatusCode::OK,
                axum::Json(serde_json::json!(paginated_response)),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn create_canvas(
    app_state: AppState,
    payload: dto::CreateCanvasDto,
) -> impl IntoResponse {
    match execute_create_canvas(&app_state, payload).await {
        Ok(result) => {
            let response_dto = dto::CanvasDto::from(result.canvas);
            (
                StatusCode::CREATED,
                axum::Json(serde_json::json!(response_dto)),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn execute_create_canvas(
    app_state: &AppState,
    payload: dto::CreateCanvasDto,
) -> Result<dto::CreateCanvasResult, sea_orm::DbErr> {
    let txn = app_state.database_connection.begin().await?;
    let created = create_canvas_transaction(&txn, payload).await?;
    txn.commit().await?;
    Ok(created)
}

async fn create_canvas_transaction(
    txn: &sea_orm::DatabaseTransaction,
    payload: dto::CreateCanvasDto,
) -> Result<dto::CreateCanvasResult, sea_orm::DbErr> {
    let canvas_id = Uuid::new_v4();
    let now = time_utils::now_utc_into();
    let canvas_entity = entity::canvas::Model {
        id: canvas_id,
        name: payload.name.clone(),
        description: payload.description.clone(),
        status: CanvasStatus::Active.to_string(),
        created_at: now,
        updated_at: now,
    };
    let canvas = repository::save_canvas_transaction(txn, canvas_entity).await?;
    let notebook = notebook_service::create_notebook_transaction(
        txn,
        canvas_id,
        payload.name.clone(),
        payload.description.clone(),
    )
    .await?;
    let dashboard = dashboard_service::create_dashboard_transaction(
        txn,
        canvas_id,
        payload.name.clone(),
        payload.description.clone().unwrap_or_default(),
        None,
    )
    .await?;
    Ok(dto::CreateCanvasResult {
        canvas,
        notebook,
        dashboard,
    })
}

fn axis_chart_content(cell_id: Uuid, axis: &PlannedAxisChart) -> AxisChartContent {
    let series: Vec<ChartSeries> = axis
        .series
        .iter()
        .enumerate()
        .map(|(index, entry)| ChartSeries {
            alias: series_alias(&entry.column, &entry.aggregate_function),
            column: entry.column.clone(),
            aggregate_function: Some(entry.aggregate_function.clone()),
            label: entry.label.clone(),
            color_index: Some(index as u32),
        })
        .collect();

    AxisChartContent {
        cell_id,
        x_axis: axis.x_axis.clone(),
        x_axis_alias: axis.x_axis.clone(),
        x_axis_title: axis.x_axis_title.clone(),
        y_axis_title: axis.y_axis_title.clone(),
        y_axis_sort_by: axis
            .sort_by_series_column
            .as_ref()
            .and_then(|column| series.iter().find(|entry| &entry.column == column))
            .map(|entry| entry.alias.clone()),
        series,
        bar_layout: Some(axis.bar_layout.clone()),
        orientation: Some(axis.orientation.clone()),
        y_axis_sort_order: Some(axis.sort_order.clone()),
        x_axis_tick_show: Some(axis.x_axis_tick_show.clone()),
        y_axis_tick_show: Some(axis.y_axis_tick_show.clone()),
        axis_minor_tick_show: Some(axis.axis_minor_tick_show.clone()),
        show_dots: Some(axis.show_dots.clone()),
    }
}

fn chart_content_json(cell_id: Uuid, chart: &PlannedChart) -> Result<String, sea_orm::DbErr> {
    let content = match &chart.spec {
        PlannedChartSpec::Bar(axis) => ChartContent::Bar(axis_chart_content(cell_id, axis)),
        PlannedChartSpec::Line(axis) => ChartContent::Line(axis_chart_content(cell_id, axis)),
        PlannedChartSpec::Pie(pie) => ChartContent::Pie(PieChartContent {
            cell_id,
            category: pie.category.clone(),
            value: pie.value.clone(),
            aggregate_function: Some(pie.aggregate_function.as_str().to_string()),
        }),
        PlannedChartSpec::Metric(metric) => ChartContent::Metric(MetricChartContent {
            cell_id,
            decimal_precision: Some(metric.decimal_precision),
            suffix: metric.suffix.clone(),
            format: Some(metric.format.clone()),
        }),
    };

    serde_json::to_string(&content)
        .map_err(|_| sea_orm::DbErr::Custom("failed to serialize chart content".into()))
}

struct CellPair {
    sql_cell_id: Uuid,
    chart_cell_id: Option<Uuid>,
    title: String,
    sql: String,
    chart_type: Option<ChartType>,
}

fn pair_cells(cells: Vec<entity::cell::Model>) -> Vec<CellPair> {
    let mut charts: HashMap<Uuid, (Uuid, ChartType)> = HashMap::new();
    for cell in cells.iter() {
        if cell.cell_type != CellType::Chart.to_string() {
            continue;
        }
        let Some(content) = cell.content.as_deref() else {
            continue;
        };
        if let Ok(chart) = serde_json::from_str::<ChartContent>(content) {
            charts.insert(chart.cell_id(), (cell.id, chart.chart_type()));
        }
    }

    cells
        .iter()
        .filter(|cell| cell.cell_type == CellType::Sql.to_string())
        .map(|cell| {
            let chart = charts.get(&cell.id);
            CellPair {
                sql_cell_id: cell.id,
                chart_cell_id: chart.map(|(id, _)| *id),
                title: cell.name.clone().unwrap_or_default(),
                sql: cell.content.clone().unwrap_or_default(),
                chart_type: chart.map(|(_, chart_type)| chart_type.clone()),
            }
        })
        .collect()
}

pub async fn list_canvas_candidates(
    app_state: &AppState,
    canvas_ids: &[Uuid],
) -> Vec<CanvasCandidate> {
    let mut candidates = Vec::new();
    for canvas_id in canvas_ids {
        let Ok(canvas) = repository::get_canvas(&app_state.database_connection, *canvas_id).await
        else {
            continue;
        };
        let Ok(notebook) = notebook_service::get_notebook_by_canvas_id(app_state, *canvas_id).await
        else {
            continue;
        };
        let Ok(cells) =
            cell_service::get_cells_by_notebook_id_entities(app_state, notebook.id).await
        else {
            continue;
        };
        candidates.push(CanvasCandidate {
            id: canvas.id,
            name: canvas.name,
            description: canvas.description,
            cells: pair_cells(cells)
                .into_iter()
                .map(|pair| CanvasCandidateCell {
                    title: pair.title,
                    sql: pair.sql,
                    chart_type: pair.chart_type,
                })
                .collect(),
        });
    }
    candidates
}

async fn resolve_existing_canvas(
    txn: &sea_orm::DatabaseTransaction,
    canvas_id: Uuid,
) -> Option<(entity::canvas::Model, Uuid)> {
    let canvas = repository::get_canvas_transaction(txn, canvas_id)
        .await
        .ok()?;
    let notebook = notebook_service::get_notebook_by_canvas_id_transaction(txn, canvas_id)
        .await
        .ok()?;
    Some((canvas, notebook.id))
}

fn non_blank(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

struct AppliedChanges {
    added: i32,
    updated: i32,
    removed: i32,
}

pub async fn apply_canvas_plan(
    app_state: &AppState,
    connection_id: Uuid,
    plan: CanvasPlan,
    target_canvas_id: Option<Uuid>,
    ops: Vec<CanvasOp>,
) -> Result<dto::CanvasChangeSummary, sea_orm::DbErr> {
    let txn = app_state.database_connection.begin().await?;

    let existing = match target_canvas_id {
        Some(canvas_id) => resolve_existing_canvas(&txn, canvas_id).await,
        None => None,
    };
    let (canvas, notebook_id, reused) = match existing {
        Some((canvas, notebook_id)) => (canvas, notebook_id, true),
        None => {
            let created = create_canvas_transaction(
                &txn,
                dto::CreateCanvasDto {
                    name: plan.name.clone(),
                    description: non_blank(&plan.description),
                },
            )
            .await?;
            (created.canvas, created.notebook.id, false)
        }
    };

    let changes = apply_ops(&txn, connection_id, notebook_id, canvas.id, ops).await?;
    if !reused && changes.added == 0 {
        return Err(sea_orm::DbErr::Custom(
            "the canvas plan produced no cells".into(),
        ));
    }
    let name = non_blank(&plan.name).unwrap_or_else(|| canvas.name.clone());
    let description = non_blank(&plan.description).or_else(|| canvas.description.clone());
    let canvas = repository::update_canvas_transaction(&txn, canvas, name, description).await?;

    txn.commit().await?;

    let (sql_cell_count, chart_cell_count) =
        cell_service::get_cell_counts_by_canvas_id(app_state, canvas.id)
            .await
            .unwrap_or((0, 0));
    let dashboard_charts_count =
        dashboard_service::get_dashboard_charts_count_by_canvas_id(app_state, canvas.id)
            .await
            .unwrap_or(0);

    Ok(dto::CanvasChangeSummary {
        canvas_id: canvas.id,
        name: canvas.name,
        description: canvas.description,
        reused,
        reasoning: plan.reasoning,
        cells_added: changes.added,
        cells_updated: changes.updated,
        cells_removed: changes.removed,
        sql_cell_count,
        chart_cell_count,
        dashboard_charts_count,
    })
}

async fn apply_ops(
    txn: &sea_orm::DatabaseTransaction,
    connection_id: Uuid,
    notebook_id: Uuid,
    canvas_id: Uuid,
    ops: Vec<CanvasOp>,
) -> Result<AppliedChanges, sea_orm::DbErr> {
    let cells = cell_service::get_cells_by_notebook_id_transaction(txn, notebook_id).await?;
    let mut order = cells.iter().map(|c| c.display_order).max().unwrap_or(-1) + 1;
    let pairs = pair_cells(cells);

    let mut changes = AppliedChanges {
        added: 0,
        updated: 0,
        removed: 0,
    };
    let mut queued: Vec<DashboardChartRequest> = Vec::new();
    let mut removed_cell_ids: Vec<Uuid> = Vec::new();
    let mut deleted_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for op in ops {
        if let CanvasOp::Update { index, .. } | CanvasOp::Delete { index } = &op {
            if deleted_indices.contains(index) {
                continue;
            }
        }
        match op {
            CanvasOp::Create { title, sql, chart } => {
                let created = create_pair(
                    txn,
                    connection_id,
                    notebook_id,
                    &mut order,
                    title,
                    sql,
                    chart.as_ref(),
                )
                .await?;
                if let Some(request) = created {
                    queued.push(request);
                }
                changes.added += 1;
            }
            CanvasOp::Update {
                index,
                title,
                sql,
                chart,
            } => {
                let Some(pair) = pairs.get(index) else {
                    continue;
                };
                if title.is_some() || sql.is_some() {
                    cell_service::update_cell_content_transaction(
                        txn,
                        pair.sql_cell_id,
                        title.clone(),
                        sql,
                    )
                    .await?;
                }
                if let Some(chart) = chart.as_ref() {
                    let content = chart_content_json(pair.sql_cell_id, chart)?;
                    match pair.chart_cell_id {
                        Some(chart_cell_id) => {
                            cell_service::update_cell_content_transaction(
                                txn,
                                chart_cell_id,
                                title,
                                Some(content),
                            )
                            .await?;
                        }
                        None => {
                            let chart_cell = create_chart_cell(
                                txn,
                                connection_id,
                                notebook_id,
                                &mut order,
                                title.or_else(|| Some(pair.title.clone())),
                                content,
                            )
                            .await?;
                            queued.push(DashboardChartRequest {
                                cell_id: chart_cell.id,
                                notebook_id,
                                width: chart.grid_width_units(),
                                height: chart.grid_height_units(),
                            });
                        }
                    }
                }
                changes.updated += 1;
            }
            CanvasOp::Delete { index } => {
                let Some(pair) = pairs.get(index) else {
                    continue;
                };
                if let Some(chart_cell_id) = pair.chart_cell_id {
                    cell_service::delete_cell_transaction(txn, chart_cell_id).await?;
                    removed_cell_ids.push(chart_cell_id);
                }
                cell_service::delete_cell_transaction(txn, pair.sql_cell_id).await?;
                deleted_indices.insert(index);
                changes.removed += 1;
            }
        }
    }

    if !queued.is_empty() || !removed_cell_ids.is_empty() {
        dashboard_service::append_dashboard_charts_transaction(
            txn,
            canvas_id,
            &removed_cell_ids,
            queued,
        )
        .await?;
    }

    Ok(changes)
}

async fn create_pair(
    txn: &sea_orm::DatabaseTransaction,
    connection_id: Uuid,
    notebook_id: Uuid,
    order: &mut i32,
    title: Option<String>,
    sql: String,
    chart: Option<&PlannedChart>,
) -> Result<Option<DashboardChartRequest>, sea_orm::DbErr> {
    let sql_cell = cell_service::execute_create_cell_transaction(
        txn,
        CreateCellDto {
            notebook_id,
            connection_id: Some(connection_id),
            name: title.clone(),
            content: Some(sql),
            display_order: None,
            cell_type: CellType::Sql,
        },
        *order,
    )
    .await?;
    *order += 1;

    let Some(chart) = chart else {
        return Ok(None);
    };
    let content = chart_content_json(sql_cell.id, chart)?;
    let chart_cell =
        create_chart_cell(txn, connection_id, notebook_id, order, title, content).await?;
    Ok(Some(DashboardChartRequest {
        cell_id: chart_cell.id,
        notebook_id,
        width: chart.grid_width_units(),
        height: chart.grid_height_units(),
    }))
}

async fn create_chart_cell(
    txn: &sea_orm::DatabaseTransaction,
    connection_id: Uuid,
    notebook_id: Uuid,
    order: &mut i32,
    title: Option<String>,
    content: String,
) -> Result<entity::cell::Model, sea_orm::DbErr> {
    let cell = cell_service::execute_create_cell_transaction(
        txn,
        CreateCellDto {
            notebook_id,
            connection_id: Some(connection_id),
            name: title,
            content: Some(content),
            display_order: None,
            cell_type: CellType::Chart,
        },
        *order,
    )
    .await?;
    *order += 1;
    Ok(cell)
}

pub async fn get_last_modified_canvases(
    app_state: &AppState,
    limit: u64,
) -> Result<Vec<entity::canvas::Model>, sea_orm::DbErr> {
    repository::get_paginated_canvases(&app_state.database_connection, 0, limit)
        .await
        .map(|(canvases, _, _)| canvases)
}

pub async fn search_canvases_by_name(
    app_state: &AppState,
    search_query: &str,
    limit: u64,
) -> Result<Vec<entity::canvas::Model>, sea_orm::DbErr> {
    repository::search_canvases_by_name(&app_state.database_connection, search_query, limit).await
}

pub async fn update_canvas(
    app_state: AppState,
    id: Uuid,
    payload: dto::CreateCanvasDto,
) -> impl IntoResponse {
    match repository::update_canvas(&app_state.database_connection, id, payload).await {
        Ok(canvas) => {
            let response_dto = dto::CanvasDto::from(canvas);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        }
        Err(e) => match e {
            sea_orm::DbErr::RecordNotFound(_) => (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({ "error": "Canvas not found" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            ),
        },
    }
}

pub async fn delete_canvas(app_state: AppState, id: Uuid) -> impl IntoResponse {
    // Start a transaction
    let txn = match app_state.database_connection.begin().await {
        Ok(txn) => txn,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };

    let notebook = match notebook_service::get_notebook_by_canvas_id_transaction(&txn, id).await {
        Ok(notebook) => notebook,
        Err(e) => {
            let _ = txn.rollback().await;
            return match e {
                sea_orm::DbErr::RecordNotFound(_) => (
                    StatusCode::NOT_FOUND,
                    axum::Json(serde_json::json!({ "error": "Canvas not found" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                ),
            };
        }
    };

    if let Err(e) = cell_service::delete_cells_by_notebook_id_transaction(&txn, notebook.id).await {
        let _ = txn.rollback().await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        );
    }

    if let Err(e) = notebook_service::delete_notebook_transaction(&txn, notebook.id).await {
        let _ = txn.rollback().await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        );
    }

    let dashboard = match dashboard_service::get_dashboard_by_canvas_id_transaction(&txn, id).await
    {
        Ok(dashboard) => dashboard,
        Err(e) => {
            let _ = txn.rollback().await;
            return match e {
                sea_orm::DbErr::RecordNotFound(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": "Dashboard not found for canvas" })),
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                ),
            };
        }
    };

    if let Err(e) = dashboard_service::delete_dashboard_transaction(&txn, dashboard.id).await {
        let _ = txn.rollback().await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        );
    }

    // Finally, delete the canvas itself
    if let Err(e) = repository::delete_canvas_transaction(&txn, id).await {
        let _ = txn.rollback().await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        );
    }

    // Commit the transaction
    match txn.commit().await {
        Ok(_) => (
            StatusCode::NO_CONTENT,
            axum::Json(serde_json::json!({ "message": "Canvas deleted successfully" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}
