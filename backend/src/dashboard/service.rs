use crate::common::AppState;
use crate::dashboard::repository;
use crate::dashboard::dto;
use crate::dashboard::constants::DashboardStatus;
use uuid::Uuid;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use chrono::Utc;
use crate::entity;

pub async fn does_dashboard_exist(
    app_state: &AppState,
    id: Uuid,
) -> bool {
    repository::get_dashboard(&app_state.database_connection, id).await.is_ok()
}

pub async fn get_dashboard(
    app_state: AppState,
    id: Uuid,
) -> impl IntoResponse {
    let dashboard = repository::get_dashboard(&app_state.database_connection, id).await;
    match dashboard {
        Ok(dashboard) => {
            let response_dto = dto::DashboardDto::from(dashboard);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
        }
    }
}

pub async fn create_dashboard(
    app_state: AppState,
    payload: dto::CreateDashboardDto,
) -> impl IntoResponse {
    // Convert DTO to entity
    let dashboard_entity = entity::dashboard::Model {
        id: Uuid::new_v4(),
        name: payload.name,
        description: payload.description,
        status: DashboardStatus::Active.to_string(),
        title: payload.title,
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };
    
    // Save the dashboard (assuming you have a repository or service)
    match repository::save_dashboard(&app_state.database_connection, dashboard_entity).await {
        Ok(saved_dashboard) => {
            // Convert entity back to DTO for response
            let response_dto = dto::DashboardDto::from(saved_dashboard);
            (StatusCode::CREATED, axum::Json(serde_json::json!(response_dto)))
        }
        Err(e) => {
            match e {
                sea_orm::DbErr::RecordNotFound(_) => {
                    (StatusCode::NOT_FOUND, axum::Json(serde_json::json!({ "error": "Dashboard not found" })))
                },
                _ => {
                    (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() })))
                }
            }
        }
    }
}
