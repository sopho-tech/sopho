use crate::cell::service as cell_service;
use crate::common::{AppState, PaginatedResponse, Pagination};
use crate::entity;
use crate::notebook::constants::NotebookStatus;
use crate::notebook::dto;
use crate::notebook::repository;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use uuid::Uuid;

pub async fn does_notebook_exist(app_state: &AppState, id: Uuid) -> bool {
    repository::get_notebook(&app_state.database_connection, id)
        .await
        .is_ok()
}

pub async fn get_notebook(app_state: AppState, id: Uuid) -> impl IntoResponse {
    let notebook = repository::get_notebook(&app_state.database_connection, id).await;
    match notebook {
        Ok(notebook) => {
            let mut response_dto = dto::NotebookDto::from(notebook);
            let cells = cell_service::get_cells_by_notebook_id(&app_state, id).await;
            match cells {
                Ok(cells) => {
                    response_dto.cells = cells;
                    (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                ),
            }
        }
        Err(e) => match e {
            sea_orm::DbErr::RecordNotFound(_) => (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({ "error": "Connection not found" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            ),
        },
    }
}

pub async fn get_all_notebooks(
    app_state: AppState,
    pagination: Query<Pagination>,
) -> impl IntoResponse {
    let page = pagination.page();
    let page_size = pagination.page_size();
    let result =
        repository::get_paginated_notebooks(&app_state.database_connection, page, page_size).await;

    match result {
        Ok((notebooks, total_items, total_pages)) => {
            let response_dto_list: Vec<dto::NotebookDto> = notebooks
                .iter()
                .map(|notebook| dto::NotebookDto::from(notebook.clone()))
                .collect();

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

pub async fn create_notebook(
    app_state: AppState,
    payload: dto::CreateNotebookDto,
) -> impl IntoResponse {
    let notebook = entity::notebook::Model {
        id: Uuid::new_v4(),
        name: payload.name,
        description: payload.description,
        status: NotebookStatus::Active.to_string(),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };
    let notebook = repository::save_notebook(&app_state.database_connection, notebook).await;
    match notebook {
        Ok(notebook) => {
            let response_dto = dto::NotebookDto::from(notebook);
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
