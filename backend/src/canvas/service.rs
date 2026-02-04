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
                page: page,
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
    let txn = match app_state.database_connection.begin().await {
        Ok(txn) => txn,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };

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

    let canvas = match repository::save_canvas_transaction(&txn, canvas_entity).await {
        Ok(canvas) => canvas,
        Err(e) => {
            let _ = txn.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };

    if let Err(e) = notebook_service::create_notebook_transaction(
        &txn,
        canvas_id,
        payload.name.clone(),
        payload.description.clone(),
    )
    .await
    {
        let _ = txn.rollback().await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        );
    }

    if let Err(e) = dashboard_service::create_dashboard_transaction(
        &txn,
        canvas_id,
        payload.name.clone(),
        payload.description.clone().unwrap_or_default(),
        None,
    )
    .await
    {
        let _ = txn.rollback().await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        );
    }

    match txn.commit().await {
        Ok(_) => {
            let response_dto = dto::CanvasDto::from(canvas);
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
