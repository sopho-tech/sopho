use crate::ai::dto::{
    CanvasCandidate, CanvasCandidateCell, CanvasOp, CanvasPlan, PlannedAxisChart, PlannedChart,
    PlannedChartSpec,
};
use crate::attribution::service::{self as attribution_service, AttributionNames};
use crate::authorization::constants::{Action, ResourceType, Visibility};
use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::authorization::dto::{AuthorizationSubject, ResourceOwnership, TenantContext};
use crate::authorization::service as authorization_service;
use crate::canvas::constants::CanvasStatus;
use crate::canvas::constants::{CANVAS_NOT_FOUND_MESSAGE, DASHBOARD_NOT_FOUND_FOR_CANVAS_MESSAGE};
use crate::canvas::dto;
use crate::canvas::error::CanvasError;
use crate::canvas::repository;
use crate::cell::constants::{CellType, ChartType};
use crate::cell::dto::{
    AxisChartContent, ChartContent, ChartSeries, CreateCellDto, MetricChartContent,
    PieChartContent, series_alias,
};
use crate::cell::service as cell_service;
use crate::common::constants::CreatedByFilter;
use crate::common::time_utils;
use crate::common::{AppState, PaginatedResponse};
use crate::dashboard::service::{self as dashboard_service, DashboardChartRequest};
use crate::entity;
use crate::notebook::service as notebook_service;
use sea_orm::Condition;
use sea_orm::ConnectionTrait;
use sea_orm::DatabaseTransaction;
use sea_orm::TransactionTrait;
use std::collections::HashMap;
use uuid::Uuid;

fn authorization_subject_for(
    canvas: &entity::canvas::Model,
    tenant_id: &TenantId,
) -> Result<AuthorizationSubject, String> {
    authorization_service::subject_from_parts(
        tenant_id,
        ResourceType::Canvas,
        canvas.id,
        canvas.owner_user_id,
        &canvas.visibility,
    )
}

pub(crate) fn reachable_canvas_condition(tenant_context: &TenantContext) -> Condition {
    authorization_service::reachable_resource_condition(
        tenant_context,
        ResourceType::Canvas,
        entity::canvas::Column::Id,
        entity::canvas::Column::OwnerUserId,
        entity::canvas::Column::Visibility,
    )
}

pub fn visible_canvas_ids_subquery(
    tenant_context: &TenantContext,
) -> sea_orm::sea_query::SelectStatement {
    repository::visible_canvas_ids_subquery(
        tenant_context.tenant_id(),
        reachable_canvas_condition(tenant_context),
    )
}

async fn authorize_canvas(
    app_state: &AppState,
    tenant_context: &TenantContext,
    id: Uuid,
    action: Action,
) -> Result<entity::canvas::Model, CanvasError> {
    let canvas = repository::get_canvas(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        id,
    )
    .await?
    .ok_or(CanvasError::NotFound)?;

    let subject = authorization_subject_for(&canvas, tenant_context.tenant_id())
        .map_err(CanvasError::Internal)?;

    authorization_service::require_access(app_state, tenant_context, &subject, action).await?;

    Ok(canvas)
}

pub async fn get_canvas(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
) -> Result<dto::CanvasDto, CanvasError> {
    let canvas = authorize_canvas(&app_state, tenant_context, id, Action::Read).await?;
    canvas_response_dto(&app_state, tenant_context, canvas).await
}

async fn canvas_response_dto(
    app_state: &AppState,
    tenant_context: &TenantContext,
    canvas: entity::canvas::Model,
) -> Result<dto::CanvasDto, CanvasError> {
    let names = attribution_service::load_names(
        app_state,
        [canvas.created_by_user_id, canvas.modified_by_user_id],
    )
    .await?;
    canvas_response_dto_with_names(app_state, tenant_context, canvas, &names).await
}

async fn canvas_response_dto_with_names(
    app_state: &AppState,
    tenant_context: &TenantContext,
    canvas: entity::canvas::Model,
    names: &AttributionNames,
) -> Result<dto::CanvasDto, CanvasError> {
    let subject = authorization_subject_for(&canvas, tenant_context.tenant_id())
        .map_err(CanvasError::Internal)?;
    let created_by = names.describe(&canvas.created_by_actor, canvas.created_by_user_id)?;
    let modified_by = names.describe(&canvas.modified_by_actor, canvas.modified_by_user_id)?;
    let mut response_dto = dto::CanvasDto::from(canvas);
    response_dto.capabilities = Some(
        authorization_service::resolve_capabilities(app_state, tenant_context, &subject).await?,
    );
    response_dto.created_by = Some(created_by);
    response_dto.modified_by = Some(modified_by);
    Ok(response_dto)
}

pub async fn get_all_canvases(
    app_state: AppState,
    tenant_context: &TenantContext,
    page: u64,
    page_size: u64,
    created_by: Option<CreatedByFilter>,
) -> Result<PaginatedResponse<dto::CanvasDto>, CanvasError> {
    authorization_service::require_capability(
        tenant_context,
        ResourceType::Canvas.permission_for(Action::Read),
    )?;

    let created_by_user_id = created_by.map(|filter| match filter {
        CreatedByFilter::Me => tenant_context.user_id().get_uuid(),
    });
    let (canvases, total_items, total_pages) = repository::get_paginated_visible_canvases(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        reachable_canvas_condition(tenant_context),
        created_by_user_id,
        page,
        page_size,
    )
    .await?;

    let names = attribution_service::load_names(
        &app_state,
        canvases
            .iter()
            .flat_map(|canvas| [canvas.created_by_user_id, canvas.modified_by_user_id]),
    )
    .await?;
    let mut response_dto_list: Vec<dto::CanvasDto> = Vec::new();
    for canvas in canvases.iter() {
        let mut canvas_dto =
            canvas_response_dto_with_names(&app_state, tenant_context, canvas.clone(), &names)
                .await?;

        let cell_counts_result = cell_service::get_cell_counts_by_canvas_id(
            &app_state,
            tenant_context.tenant_id(),
            canvas.id,
        )
        .await;
        if let Ok((sql_count, chart_count)) = cell_counts_result {
            canvas_dto.sql_cell_count = sql_count;
            canvas_dto.chart_cell_count = chart_count;
        }

        let dashboard_charts_result = dashboard_service::get_dashboard_charts_count_by_canvas_id(
            &app_state,
            tenant_context.tenant_id(),
            canvas.id,
        )
        .await;
        if let Ok(dashboard_charts_count) = dashboard_charts_result {
            canvas_dto.dashboard_charts_count = dashboard_charts_count;
        }

        response_dto_list.push(canvas_dto);
    }

    Ok(PaginatedResponse {
        data: response_dto_list,
        total_pages,
        page,
        page_size,
        total_items,
    })
}

pub async fn create_canvas(
    app_state: AppState,
    tenant_context: &TenantContext,
    owner_user_id: Uuid,
    payload: dto::CreateCanvasDto,
) -> Result<dto::CanvasDto, CanvasError> {
    crate::authorization::service::require_capability(
        tenant_context,
        crate::authorization::constants::Permission::CanvasCreate,
    )?;

    let created = execute_create_canvas(
        &app_state,
        tenant_context.tenant_id(),
        ResourceOwnership {
            owner_user_id,
            visibility: Visibility::Tenant,
        },
        &tenant_context.actor(),
        payload,
    )
    .await?;

    canvas_response_dto(&app_state, tenant_context, created.canvas).await
}

pub async fn execute_create_canvas(
    app_state: &AppState,
    tenant_id: &TenantId,
    ownership: ResourceOwnership,
    actor: &Actor,
    payload: dto::CreateCanvasDto,
) -> Result<dto::CreateCanvasResult, sea_orm::DbErr> {
    let txn = app_state.database_connection.begin().await?;
    let created = create_canvas_transaction(&txn, tenant_id, ownership, payload, actor).await?;
    txn.commit().await?;
    Ok(created)
}

async fn create_canvas_transaction(
    txn: &sea_orm::DatabaseTransaction,
    tenant_id: &TenantId,
    ownership: ResourceOwnership,
    payload: dto::CreateCanvasDto,
    actor: &Actor,
) -> Result<dto::CreateCanvasResult, sea_orm::DbErr> {
    let canvas_id = Uuid::new_v4();
    let now = time_utils::now_utc_into();
    let canvas_entity = entity::canvas::Model {
        id: canvas_id,
        tenant_id: tenant_id.get_uuid(),
        owner_user_id: ownership.owner_user_id,
        visibility: ownership.visibility.as_str().to_string(),
        name: payload.name.clone(),
        description: payload.description.clone(),
        status: CanvasStatus::Active.to_string(),
        created_by_user_id: actor.user_id(),
        created_by_actor: actor.kind_name(),
        modified_by_user_id: actor.user_id(),
        modified_by_actor: actor.kind_name(),
        created_at: now,
        updated_at: now,
    };
    let canvas = repository::save_canvas_transaction(txn, tenant_id, canvas_entity).await?;
    let notebook = notebook_service::create_notebook_transaction(
        txn,
        tenant_id,
        canvas_id,
        payload.name.clone(),
        payload.description.clone(),
        actor,
    )
    .await?;
    dashboard_service::create_dashboard_transaction(
        txn,
        tenant_id,
        canvas_id,
        payload.name.clone(),
        payload.description.clone().unwrap_or_default(),
        None,
        actor,
    )
    .await?;
    Ok(dto::CreateCanvasResult { canvas, notebook })
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
    tenant_id: &TenantId,
    canvas_ids: &[Uuid],
) -> Vec<CanvasCandidate> {
    let mut candidates = Vec::new();
    for canvas_id in canvas_ids {
        let Ok(canvas) =
            repository::get_canvas(&app_state.database_connection, tenant_id, *canvas_id).await
        else {
            continue;
        };
        let Some(canvas) = canvas else {
            continue;
        };
        let Ok(notebook) =
            notebook_service::get_notebook_by_canvas_id(app_state, tenant_id, *canvas_id).await
        else {
            continue;
        };
        let Ok(cells) =
            cell_service::get_cells_by_notebook_id_entities(app_state, tenant_id, notebook.id)
                .await
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
    tenant_id: &TenantId,
    canvas_id: Uuid,
) -> Option<(entity::canvas::Model, Uuid)> {
    let canvas = repository::get_canvas_transaction(txn, tenant_id, canvas_id)
        .await
        .ok()
        .flatten()?;
    let notebook =
        notebook_service::get_notebook_by_canvas_id_transaction(txn, tenant_id, canvas_id)
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

struct PlannedCellScope<'a> {
    txn: &'a DatabaseTransaction,
    tenant_id: &'a TenantId,
    actor: &'a Actor,
    connection_id: Uuid,
    notebook_id: Uuid,
}

pub async fn apply_canvas_plan(
    app_state: &AppState,
    tenant_id: &TenantId,
    authorship: dto::CanvasAuthorship,
    connection_id: Uuid,
    plan: CanvasPlan,
    target_canvas_id: Option<Uuid>,
    ops: Vec<CanvasOp>,
) -> Result<dto::CanvasChangeSummary, sea_orm::DbErr> {
    let actor = &authorship.actor;
    let txn = app_state.database_connection.begin().await?;

    let existing = match target_canvas_id {
        Some(canvas_id) => resolve_existing_canvas(&txn, tenant_id, canvas_id).await,
        None => None,
    };
    let (canvas, notebook_id, reused) = match existing {
        Some((canvas, notebook_id)) => (canvas, notebook_id, true),
        None => {
            let created = create_canvas_transaction(
                &txn,
                tenant_id,
                authorship.ownership,
                dto::CreateCanvasDto {
                    name: plan.name.clone(),
                    description: non_blank(&plan.description),
                },
                actor,
            )
            .await?;
            (created.canvas, created.notebook.id, false)
        }
    };

    if reused {
        stamp_modified(&txn, tenant_id, canvas.id, actor).await?;
    }
    let scope = PlannedCellScope {
        txn: &txn,
        tenant_id,
        actor,
        connection_id,
        notebook_id,
    };
    let changes = apply_ops(&scope, canvas.id, ops).await?;
    if !reused && changes.added == 0 {
        return Err(sea_orm::DbErr::Custom(
            "the canvas plan produced no cells".into(),
        ));
    }
    let name = non_blank(&plan.name).unwrap_or_else(|| canvas.name.clone());
    let description = non_blank(&plan.description).or_else(|| canvas.description.clone());
    let canvas =
        repository::update_canvas_transaction(&txn, canvas, name, description, actor).await?;

    txn.commit().await?;

    let (sql_cell_count, chart_cell_count) =
        cell_service::get_cell_counts_by_canvas_id(app_state, tenant_id, canvas.id)
            .await
            .unwrap_or((0, 0));
    let dashboard_charts_count =
        dashboard_service::get_dashboard_charts_count_by_canvas_id(app_state, tenant_id, canvas.id)
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
    scope: &PlannedCellScope<'_>,
    canvas_id: Uuid,
    ops: Vec<CanvasOp>,
) -> Result<AppliedChanges, sea_orm::DbErr> {
    let &PlannedCellScope {
        txn,
        tenant_id,
        actor,
        notebook_id,
        ..
    } = scope;
    let cells =
        cell_service::get_cells_by_notebook_id_transaction(txn, tenant_id, notebook_id).await?;
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
        if let CanvasOp::Update { index, .. } | CanvasOp::Delete { index } = &op
            && deleted_indices.contains(index)
        {
            continue;
        }
        match op {
            CanvasOp::Create { title, sql, chart } => {
                let created = create_pair(scope, &mut order, title, sql, chart.as_ref()).await?;
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
                        tenant_id,
                        pair.sql_cell_id,
                        title.clone(),
                        sql,
                        actor,
                    )
                    .await?;
                }
                if let Some(chart) = chart.as_ref() {
                    let content = chart_content_json(pair.sql_cell_id, chart)?;
                    match pair.chart_cell_id {
                        Some(chart_cell_id) => {
                            cell_service::update_cell_content_transaction(
                                txn,
                                tenant_id,
                                chart_cell_id,
                                title,
                                Some(content),
                                actor,
                            )
                            .await?;
                        }
                        None => {
                            let chart_cell = create_chart_cell(
                                scope,
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
                    cell_service::delete_cell_transaction(txn, tenant_id, chart_cell_id).await?;
                    removed_cell_ids.push(chart_cell_id);
                }
                cell_service::delete_cell_transaction(txn, tenant_id, pair.sql_cell_id).await?;
                deleted_indices.insert(index);
                changes.removed += 1;
            }
        }
    }

    if !queued.is_empty() || !removed_cell_ids.is_empty() {
        dashboard_service::append_dashboard_charts_transaction(
            txn,
            tenant_id,
            canvas_id,
            &removed_cell_ids,
            queued,
            actor,
        )
        .await?;
    }

    Ok(changes)
}

async fn create_pair(
    scope: &PlannedCellScope<'_>,
    order: &mut i32,
    title: Option<String>,
    sql: String,
    chart: Option<&PlannedChart>,
) -> Result<Option<DashboardChartRequest>, sea_orm::DbErr> {
    let &PlannedCellScope {
        txn,
        tenant_id,
        actor,
        connection_id,
        notebook_id,
    } = scope;
    let sql_cell = cell_service::execute_create_cell_transaction(
        txn,
        tenant_id,
        CreateCellDto {
            notebook_id,
            connection_id: Some(connection_id),
            name: title.clone(),
            content: Some(sql),
            display_order: None,
            cell_type: CellType::Sql,
        },
        *order,
        actor,
    )
    .await?;
    *order += 1;

    let Some(chart) = chart else {
        return Ok(None);
    };
    let content = chart_content_json(sql_cell.id, chart)?;
    let chart_cell = create_chart_cell(scope, order, title, content).await?;
    Ok(Some(DashboardChartRequest {
        cell_id: chart_cell.id,
        notebook_id,
        width: chart.grid_width_units(),
        height: chart.grid_height_units(),
    }))
}

async fn create_chart_cell(
    scope: &PlannedCellScope<'_>,
    order: &mut i32,
    title: Option<String>,
    content: String,
) -> Result<entity::cell::Model, sea_orm::DbErr> {
    let &PlannedCellScope {
        txn,
        tenant_id,
        actor,
        connection_id,
        notebook_id,
    } = scope;
    let cell = cell_service::execute_create_cell_transaction(
        txn,
        tenant_id,
        CreateCellDto {
            notebook_id,
            connection_id: Some(connection_id),
            name: title,
            content: Some(content),
            display_order: None,
            cell_type: CellType::Chart,
        },
        *order,
        actor,
    )
    .await?;
    *order += 1;
    Ok(cell)
}

pub async fn get_last_modified_canvases(
    app_state: &AppState,
    tenant_context: &TenantContext,
    limit: u64,
) -> Result<Vec<entity::canvas::Model>, sea_orm::DbErr> {
    repository::get_paginated_visible_canvases(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        reachable_canvas_condition(tenant_context),
        None,
        0,
        limit,
    )
    .await
    .map(|(canvases, _, _)| canvases)
}

pub async fn search_canvases_by_name(
    app_state: &AppState,
    tenant_context: &TenantContext,
    search_query: &str,
    limit: u64,
) -> Result<Vec<entity::canvas::Model>, sea_orm::DbErr> {
    repository::search_canvases_by_name(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        reachable_canvas_condition(tenant_context),
        search_query,
        limit,
    )
    .await
}

pub async fn update_canvas(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
    payload: dto::CreateCanvasDto,
) -> Result<dto::CanvasDto, CanvasError> {
    authorize_canvas(&app_state, tenant_context, id, Action::Write).await?;

    let canvas = repository::update_canvas(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        id,
        payload,
        &tenant_context.actor(),
    )
    .await?
    .ok_or(CanvasError::NotFound)?;

    canvas_response_dto(&app_state, tenant_context, canvas).await
}

pub async fn delete_canvas(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
) -> Result<(), CanvasError> {
    authorize_canvas(&app_state, tenant_context, id, Action::Delete).await?;

    let transaction = app_state.database_connection.begin().await?;
    match delete_canvas_in_transaction(&transaction, tenant_context.tenant_id(), id).await {
        Ok(()) => {
            transaction.commit().await?;
            Ok(())
        }
        Err(error) => {
            let _ = transaction.rollback().await;
            Err(error)
        }
    }
}

async fn delete_canvas_in_transaction(
    transaction: &DatabaseTransaction,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<(), CanvasError> {
    repository::delete_canvas_transaction(transaction, tenant_id, id).await?;
    let notebook =
        notebook_service::get_notebook_by_canvas_id_transaction(transaction, tenant_id, id)
            .await
            .map_err(|_| CanvasError::NotFound)?;

    cell_service::delete_cells_by_notebook_id_transaction(transaction, tenant_id, notebook.id)
        .await?;
    notebook_service::delete_notebook_transaction(transaction, tenant_id, notebook.id).await?;

    let dashboard =
        dashboard_service::get_dashboard_by_canvas_id_transaction(transaction, tenant_id, id)
            .await
            .map_err(|error| match error {
                sea_orm::DbErr::RecordNotFound(_) => {
                    CanvasError::Internal(DASHBOARD_NOT_FOUND_FOR_CANVAS_MESSAGE.to_string())
                }
                other => CanvasError::Database(other),
            })?;

    dashboard_service::delete_dashboard_transaction(transaction, tenant_id, dashboard.id).await?;

    Ok(())
}

pub async fn load_authorization_subject(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<AuthorizationSubject, sea_orm::DbErr> {
    let canvas = repository::get_canvas(&app_state.database_connection, tenant_id, id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(CANVAS_NOT_FOUND_MESSAGE.into()))?;
    authorization_subject_for(&canvas, tenant_id).map_err(sea_orm::DbErr::Custom)
}

pub async fn set_visibility(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
    visibility: Visibility,
    actor: &Actor,
) -> Result<(), sea_orm::DbErr> {
    let updated_rows = repository::update_canvas_visibility(
        &app_state.database_connection,
        tenant_id,
        id,
        visibility.as_str(),
        actor,
    )
    .await?;
    if updated_rows == 0 {
        return Err(sea_orm::DbErr::RecordNotFound(
            CANVAS_NOT_FOUND_MESSAGE.into(),
        ));
    }
    Ok(())
}

pub async fn stamp_modified(
    connection: &impl ConnectionTrait,
    tenant_id: &TenantId,
    id: Uuid,
    actor: &Actor,
) -> Result<(), sea_orm::DbErr> {
    let stamped_rows = repository::stamp_modified(connection, tenant_id, id, actor).await?;
    if stamped_rows == 0 {
        return Err(sea_orm::DbErr::RecordNotFound(
            CANVAS_NOT_FOUND_MESSAGE.into(),
        ));
    }
    Ok(())
}
