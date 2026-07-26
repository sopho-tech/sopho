use crate::common::time_utils;
use crate::common::AppState;
use crate::dashboard::constants;
use crate::dashboard::constants::DashboardStatus;
use crate::dashboard::dto;
use crate::dashboard::dto::Layout;
use crate::dashboard::repository;
use crate::entity;
use axum::http::StatusCode;
use axum::response::IntoResponse;
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

pub async fn get_dashboard_by_canvas_id(app_state: AppState, canvas_id: Uuid) -> impl IntoResponse {
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
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
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
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
    };
    repository::save_dashboard_transaction(txn, dashboard_entity).await
}

pub async fn get_dashboard_by_canvas_id_entity(
    app_state: &AppState,
    canvas_id: Uuid,
) -> Result<entity::dashboard::Model, sea_orm::DbErr> {
    repository::get_dashboard_by_canvas_id(&app_state.database_connection, canvas_id).await
}

pub async fn get_dashboard_by_canvas_id_transaction(
    txn: &DatabaseTransaction,
    canvas_id: Uuid,
) -> Result<entity::dashboard::Model, sea_orm::DbErr> {
    repository::get_dashboard_by_canvas_id_transaction(txn, canvas_id).await
}

pub async fn execute_update_dashboard(
    app_state: &AppState,
    dashboard_id: Uuid,
    payload: dto::DashboardDto,
) -> Result<entity::dashboard::Model, sea_orm::DbErr> {
    repository::update_dashboard(&app_state.database_connection, dashboard_id, payload).await
}

pub struct DashboardChartPlacement {
    pub cell_id: Uuid,
    pub notebook_id: Uuid,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Default)]
pub struct DashboardGridPacker {
    x: u16,
    y: u16,
    row_height: u16,
}

impl DashboardGridPacker {
    pub fn place(&mut self, width: Option<u16>, height: Option<u16>) -> (u16, u16, u16, u16) {
        let width = width
            .unwrap_or(constants::DEFAULT_CHART_WIDTH)
            .clamp(constants::MIN_CHART_WIDTH, constants::GRID_COLUMN_COUNT);
        let height = height
            .unwrap_or(constants::DEFAULT_CHART_HEIGHT)
            .clamp(constants::MIN_CHART_HEIGHT, constants::MAX_CHART_HEIGHT);

        if self.x + width > constants::GRID_COLUMN_COUNT {
            self.y += self.row_height;
            self.x = 0;
            self.row_height = 0;
        }

        let placement = (self.x, self.y, width, height);
        self.x += width;
        self.row_height = self.row_height.max(height);
        placement
    }
}

pub async fn set_dashboard_layout(
    app_state: &AppState,
    dashboard_id: Uuid,
    name: String,
    description: Option<String>,
    charts: Vec<DashboardChartPlacement>,
) -> Result<entity::dashboard::Model, sea_orm::DbErr> {
    let layout: Vec<Layout> = charts
        .into_iter()
        .map(|c| Layout::new(c.cell_id, c.notebook_id, c.x, c.y, c.width, c.height))
        .collect();
    let payload = dto::DashboardDto {
        id: dashboard_id,
        name: Some(name),
        description,
        layout: Some(layout),
        status: DashboardStatus::Active,
    };
    repository::update_dashboard(&app_state.database_connection, dashboard_id, payload).await
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
    let dashboard = execute_update_dashboard(&app_state, dashboard_id, payload).await;
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

pub async fn remove_cell_from_layout_transaction(
    txn: &DatabaseTransaction,
    canvas_id: Uuid,
    cell_id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    let dashboard = match repository::get_dashboard_by_canvas_id_transaction(txn, canvas_id).await {
        Ok(d) => d,
        Err(_) => return Ok(()),
    };
    let mut layout = match dto::Layout::from_json(dashboard.layout.clone()) {
        Some(l) => l,
        None => return Ok(()),
    };
    layout.retain(|l| l.cell_id() != cell_id);
    repository::update_dashboard_layout_transaction(
        txn,
        dashboard,
        if layout.is_empty() {
            None
        } else {
            Some(layout)
        },
    )
    .await?;
    Ok(())
}

pub async fn get_dashboard_charts_count_by_canvas_id(
    app_state: &AppState,
    canvas_id: Uuid,
) -> Result<i32, sea_orm::DbErr> {
    let dashboard = get_dashboard_by_canvas_id_entity(app_state, canvas_id).await?;

    if let Some(layout_json) = dashboard.layout {
        let layout_value: serde_json::Value = layout_json;
        if let Ok(layout_items) = serde_json::from_value::<Vec<serde_json::Value>>(layout_value) {
            return Ok(layout_items.len() as i32);
        }
    }

    Ok(0)
}
