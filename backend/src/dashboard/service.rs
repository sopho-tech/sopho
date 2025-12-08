use crate::common::AppState;
use crate::dashboard::constants::DashboardStatus;
use crate::dashboard::dto;
use crate::dashboard::dto::Layout;
use crate::dashboard::repository;
use crate::entity;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use sea_orm::DatabaseTransaction;
use uuid::Uuid;

pub async fn does_dashboard_exist(app_state: &AppState, id: Uuid) -> bool {
    repository::get_dashboard(&app_state.database_connection, id)
        .await
        .is_ok()
}

pub async fn get_dashboard(app_state: AppState, id: Uuid) -> impl IntoResponse {
    let dashboard = repository::get_dashboard(&app_state.database_connection, id).await;
    match dashboard {
        Ok(dashboard) => {
            let response_dto = dto::DashboardDto::from(dashboard);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_dashboard_by_canvas_id(
    app_state: AppState,
    canvas_id: Uuid,
) -> impl IntoResponse {
    let dashboard =
        repository::get_dashboard_by_canvas_id(&app_state.database_connection, canvas_id).await;
    match dashboard {
        Ok(dashboard) => {
            let response_dto = dto::DashboardDto::from(dashboard);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        }
        Err(e) => match e {
            sea_orm::DbErr::RecordNotFound(_) => (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({ "error": "Dashboard not found" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            ),
        },
    }
}

pub async fn create_dashboard(
    app_state: AppState,
    payload: dto::CreateDashboardDto,
) -> impl IntoResponse {
    let dashboard_entity = entity::dashboard::Model {
        id: Uuid::new_v4(),
        canvas_id: Uuid::new_v4(),
        name: payload.name,
        description: payload.description,
        status: DashboardStatus::Active.to_string(),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
        layout: None,
    };

    match repository::save_dashboard_connection(&app_state.database_connection, dashboard_entity)
        .await
    {
        Ok(saved_dashboard) => {
            let response_dto = dto::DashboardDto::from(saved_dashboard);
            (
                StatusCode::CREATED,
                axum::Json(serde_json::json!(response_dto)),
            )
        }
        Err(e) => match e {
            sea_orm::DbErr::RecordNotFound(_) => (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({ "error": "Dashboard not found" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            ),
        },
    }
}

pub async fn create_dashboard_transaction(
    txn: &DatabaseTransaction,
    canvas_id: Uuid,
    name: String,
    description: String,
    layout: Option<Vec<Layout>>,
) -> Result<entity::dashboard::Model, sea_orm::DbErr> {
    let dashboard_entity = entity::dashboard::Model {
        id: Uuid::new_v4(),
        canvas_id,
        name,
        description,
        layout: Layout::to_json(layout),
        status: DashboardStatus::Active.to_string(),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    };
    repository::save_dashboard_transaction(txn, dashboard_entity).await
}

pub async fn get_dashboard_by_canvas_id_transaction(
    txn: &DatabaseTransaction,
    canvas_id: Uuid,
) -> Result<entity::dashboard::Model, sea_orm::DbErr> {
    repository::get_dashboard_by_canvas_id_transaction(txn, canvas_id).await
}

pub async fn delete_dashboard_transaction(
    txn: &DatabaseTransaction,
    id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    repository::delete_dashboard_transaction(txn, id).await
}

pub async fn update_dashboard(
    app_state: AppState,
    dashboard_id: Uuid,
    payload: dto::DashboardDto,
) -> impl IntoResponse {
    let dashboard =
        repository::update_dashboard(&app_state.database_connection, dashboard_id, payload).await;
    match dashboard {
        Ok(dashboard) => {
            let response_dto = dto::DashboardDto::from(dashboard);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        }
        Err(e) => match e {
            sea_orm::DbErr::RecordNotFound(_) => (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({ "error": "Dashboard not found" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            ),
        },
    }
}
