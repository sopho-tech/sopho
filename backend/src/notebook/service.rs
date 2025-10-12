use crate::common::AppState;
use crate::notebook::constants::NotebookStatus;
use crate::notebook::dto;
use crate::notebook::repository;
use crate::entity;
use axum::response::IntoResponse;
use uuid::Uuid;
use axum::http::StatusCode;
use chrono::Utc;
use crate::cell::service as cell_service;


pub async fn does_notebook_exist(
    app_state: &AppState,
    id: Uuid,
) -> bool {
    repository::get_notebook(&app_state.database_connection, id).await.is_ok()
}

pub async fn get_notebook(
    app_state: AppState,
    id: Uuid,
) -> impl IntoResponse {
    let notebook = repository::get_notebook(&app_state.database_connection, id).await;
    match notebook {
        Ok(notebook) => {
            let mut response_dto = dto::NotebookDto::from(notebook);
            let cells = cell_service::get_cells_by_notebook_id(&app_state, id).await;
            match cells {
                Ok(cells) => {
                    response_dto.cells = cells;
                    (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
                },
                Err(e) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
                }
            }
        },
        Err(e) => {
            match e {
                sea_orm::DbErr::RecordNotFound(_) => {
                    (StatusCode::NOT_FOUND, axum::Json(serde_json::json!({ "error": "Connection not found" })))
                },
                _ => {
                    (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
                }
            }
        }
    }
}


pub async fn get_all_notebooks(app_state: AppState) -> impl IntoResponse {
    let notebooks = repository::get_all_notebooks(&app_state.database_connection).await;
    match notebooks {
        Ok(notebooks) => {
            let response_dto_list: Vec<dto::NotebookDto> = notebooks.iter()
                .map(|notebook| dto::NotebookDto::from(notebook.clone()))
                .collect();
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto_list)))
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

pub async fn create_notebook(app_state: AppState, payload: dto::CreateNotebookDto) -> impl IntoResponse {
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
            (StatusCode::CREATED, axum::Json(serde_json::json!(response_dto)))
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}
