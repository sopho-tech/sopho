use crate::canvas::constants::CanvasStatus;
use crate::canvas::dto;
use crate::canvas::repository;
use crate::cell::service as cell_service;
use crate::common::time_utils;
use crate::common::{AppState, PaginatedResponse, Pagination};
use crate::dashboard::service as dashboard_service;
use crate::entity;
use crate::notebook::service as notebook_service;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::TransactionTrait;
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
    let canvas = repository::save_canvas_transaction(&txn, canvas_entity).await?;
    let notebook = notebook_service::create_notebook_transaction(
        &txn,
        canvas_id,
        payload.name.clone(),
        payload.description.clone(),
    )
    .await?;
    let dashboard = dashboard_service::create_dashboard_transaction(
        &txn,
        canvas_id,
        payload.name.clone(),
        payload.description.clone().unwrap_or_default(),
        None,
    )
    .await?;
    txn.commit().await?;
    Ok(dto::CreateCanvasResult {
        canvas,
        notebook,
        dashboard,
    })
}

fn chart_content_json(
    cell_id: Uuid,
    chart: &crate::ai::dto::CanvasPlanChart,
) -> Result<String, sea_orm::DbErr> {
    use crate::cell::constants::{
        AggregateFunction, AxisMinorTickShow, AxisTickShow, ChartOrientation, ChartType,
        MetricFormat, SortOrder,
    };
    use crate::cell::dto::{
        AxisChartContent, ChartContent, MetricChartContent, PieChartContent,
    };

    let aggregate_function = chart
        .aggregate_function
        .clone()
        .unwrap_or(AggregateFunction::Max)
        .as_str()
        .to_string();

    let content = match chart.chart_type {
        ChartType::Bar | ChartType::Line => {
            let axis = AxisChartContent {
                cell_id,
                x_axis: chart.x_axis.clone().unwrap_or_default(),
                y_axis: chart.y_axis.clone().unwrap_or_default(),
                orientation: Some(ChartOrientation::Vertical),
                y_axis_aggregate_function: Some(aggregate_function),
                y_axis_sort_order: Some(SortOrder::None),
                x_axis_tick_show: Some(AxisTickShow::Show),
                y_axis_tick_show: Some(AxisTickShow::Show),
                axis_minor_tick_show: Some(AxisMinorTickShow::Show),
            };
            match chart.chart_type {
                ChartType::Line => ChartContent::Line(axis),
                _ => ChartContent::Bar(axis),
            }
        }
        ChartType::Pie => ChartContent::Pie(PieChartContent {
            cell_id,
            category: chart.category.clone().unwrap_or_default(),
            value: chart.value.clone().unwrap_or_default(),
            aggregate_function: Some(aggregate_function),
        }),
        ChartType::Metric => ChartContent::Metric(MetricChartContent {
            cell_id,
            decimal_precision: Some(2),
            suffix: None,
            format: Some(MetricFormat::Default),
        }),
    };

    serde_json::to_string(&content)
        .map_err(|_| sea_orm::DbErr::Custom("failed to serialize chart content".into()))
}

pub async fn generate_canvas_from_plan(
    app_state: &AppState,
    connection_id: Uuid,
    plan: crate::ai::dto::CanvasPlan,
) -> Result<dto::GeneratedCanvasSummary, sea_orm::DbErr> {
    use crate::cell::constants::CellType;
    use crate::cell::dto::CreateCellDto;
    use crate::dashboard::service::{DashboardChartPlacement, DashboardGridPacker};

    let created = execute_create_canvas(
        app_state,
        dto::CreateCanvasDto {
            name: plan.name.clone(),
            description: plan.description.clone(),
        },
    )
    .await?;
    let notebook_id = created.notebook.id;
    let canvas_id = created.canvas.id;

    let mut sql_cell_count = 0;
    let mut chart_cell_count = 0;
    let mut placements: Vec<DashboardChartPlacement> = Vec::new();
    let mut packer = DashboardGridPacker::default();
    let mut order = 0;

    for plan_cell in plan.cells.iter() {
        let sql_cell = cell_service::execute_create_cell(
            app_state,
            CreateCellDto {
                notebook_id,
                connection_id: Some(connection_id),
                name: Some(plan_cell.title.clone()),
                content: Some(plan_cell.sql.clone()),
                display_order: Some(order),
                cell_type: CellType::Sql,
            },
        )
        .await
        .map_err(|_| sea_orm::DbErr::Custom("failed to create sql cell".into()))?;
        sql_cell_count += 1;
        order += 1;

        if let Some(chart) = &plan_cell.chart {
            let chart_cell = cell_service::execute_create_cell(
                app_state,
                CreateCellDto {
                    notebook_id,
                    connection_id: Some(connection_id),
                    name: Some(plan_cell.title.clone()),
                    content: Some(chart_content_json(sql_cell.id, chart)?),
                    display_order: Some(order),
                    cell_type: CellType::Chart,
                },
            )
            .await
            .map_err(|_| sea_orm::DbErr::Custom("failed to create chart cell".into()))?;
            chart_cell_count += 1;
            order += 1;

            let (x, y, width, height) = packer.place(chart.grid_width, chart.grid_height);
            placements.push(DashboardChartPlacement {
                cell_id: chart_cell.id,
                notebook_id,
                x,
                y,
                width,
                height,
            });
        }
    }

    let dashboard_charts_count = placements.len() as i32;
    if !placements.is_empty() {
        dashboard_service::set_dashboard_layout(
            app_state,
            created.dashboard.id,
            plan.name.clone(),
            plan.description.clone(),
            placements,
        )
        .await?;
    }

    Ok(dto::GeneratedCanvasSummary {
        canvas_id,
        name: plan.name,
        description: plan.description,
        sql_cell_count,
        chart_cell_count,
        dashboard_charts_count,
    })
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
