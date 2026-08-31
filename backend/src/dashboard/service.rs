use crate::ai_summary::service as ai_summary_service;
use crate::authorization::constants::Action;
use crate::authorization::dto::{TenantContext, TenantId};
use crate::common::errors::UpdateDashboardError;
use crate::common::time_utils;
use crate::common::AppState;
use crate::dashboard::constants;
use crate::dashboard::constants::DashboardStatus;
use crate::dashboard::constants::DASHBOARD_NOT_FOUND_MESSAGE;
use crate::dashboard::dto;
use crate::dashboard::dto::Layout;
use crate::dashboard::error::DashboardError;
use crate::dashboard::repository;
use crate::entity;
use sea_orm::DatabaseTransaction;
use uuid::Uuid;

pub async fn does_dashboard_exist(app_state: &AppState, tenant_id: &TenantId, id: Uuid) -> bool {
    repository::get_dashboard(&app_state.database_connection, tenant_id, id)
        .await
        .is_ok()
}

pub async fn get_dashboard(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
) -> Result<dto::DashboardDto, DashboardError> {
    let dashboard = authorize_dashboard(&app_state, tenant_context, id, Action::Read).await?;
    Ok(dto::DashboardDto::from(dashboard))
}

pub async fn get_dashboard_by_canvas_id(
    app_state: AppState,
    tenant_context: &TenantContext,
    canvas_id: Uuid,
) -> Result<dto::DashboardDto, DashboardError> {
    authorize_canvas_for_children(&app_state, tenant_context, canvas_id).await?;

    let dashboard = repository::get_dashboard_by_canvas_id(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        canvas_id,
    )
    .await?
    .ok_or(DashboardError::NotFound)?;

    Ok(dto::DashboardDto::from(dashboard))
}

pub async fn create_dashboard(
    app_state: AppState,
    tenant_context: &TenantContext,
    payload: dto::CreateDashboardDto,
) -> Result<dto::DashboardDto, DashboardError> {
    crate::authorization::service::require_capability(
        tenant_context,
        crate::authorization::constants::Permission::CanvasWrite,
    )?;
    let tenant_id = tenant_context.tenant_id();
    let dashboard_entity = entity::dashboard::Model {
        id: Uuid::new_v4(),
        tenant_id: tenant_id.get_uuid(),
        canvas_id: Uuid::new_v4(),
        name: payload.name,
        description: payload.description,
        status: DashboardStatus::Active.to_string(),
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
        layout: None,
    };

    let saved_dashboard = repository::save_dashboard_connection(
        &app_state.database_connection,
        tenant_id,
        dashboard_entity,
    )
    .await?;

    Ok(dto::DashboardDto::from(saved_dashboard))
}

pub async fn create_dashboard_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    canvas_id: Uuid,
    name: String,
    description: String,
    layout: Option<Vec<Layout>>,
) -> Result<entity::dashboard::Model, sea_orm::DbErr> {
    let dashboard_entity = entity::dashboard::Model {
        id: Uuid::new_v4(),
        tenant_id: tenant_id.get_uuid(),
        canvas_id,
        name,
        description,
        layout: Layout::to_json(layout),
        status: DashboardStatus::Active.to_string(),
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
    };
    repository::save_dashboard_transaction(txn, tenant_id, dashboard_entity).await
}

pub async fn get_dashboard_entity(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<entity::dashboard::Model, sea_orm::DbErr> {
    repository::get_dashboard(&app_state.database_connection, tenant_id, id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(DASHBOARD_NOT_FOUND_MESSAGE.into()))
}

pub fn dashboard_chart_cell_ids(dashboard: &entity::dashboard::Model) -> Vec<Uuid> {
    Layout::from_json(dashboard.layout.clone())
        .unwrap_or_default()
        .iter()
        .map(Layout::cell_id)
        .collect()
}

pub async fn get_dashboard_by_canvas_id_entity(
    app_state: &AppState,
    tenant_id: &TenantId,
    canvas_id: Uuid,
) -> Result<entity::dashboard::Model, sea_orm::DbErr> {
    repository::get_dashboard_by_canvas_id(&app_state.database_connection, tenant_id, canvas_id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(DASHBOARD_NOT_FOUND_MESSAGE.into()))
}

pub async fn get_dashboard_by_canvas_id_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    canvas_id: Uuid,
) -> Result<entity::dashboard::Model, sea_orm::DbErr> {
    repository::get_dashboard_by_canvas_id_transaction(txn, tenant_id, canvas_id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(DASHBOARD_NOT_FOUND_MESSAGE.into()))
}

pub async fn execute_update_dashboard(
    app_state: &AppState,
    tenant_id: &TenantId,
    dashboard_id: Uuid,
    payload: dto::DashboardDto,
) -> Result<entity::dashboard::Model, UpdateDashboardError> {
    let chart_count = payload.layout.as_ref().map_or(0, Vec::len);
    if chart_count > constants::MAX_CHARTS_PER_DASHBOARD {
        return Err(UpdateDashboardError::TooManyCharts(
            constants::MAX_CHARTS_PER_DASHBOARD,
        ));
    }
    repository::update_dashboard(
        &app_state.database_connection,
        tenant_id,
        dashboard_id,
        payload,
    )
    .await
    .map_err(UpdateDashboardError::Repository)?
    .ok_or(UpdateDashboardError::Repository(
        sea_orm::DbErr::RecordNotFound(DASHBOARD_NOT_FOUND_MESSAGE.into()),
    ))
}

pub struct DashboardChartRequest {
    pub cell_id: Uuid,
    pub notebook_id: Uuid,
    pub width: Option<u16>,
    pub height: Option<u16>,
}

#[derive(Default)]
pub struct DashboardGridPacker {
    x: u16,
    y: u16,
    row_height: u16,
}

impl DashboardGridPacker {
    pub fn below(layout: &[Layout]) -> Self {
        Self {
            x: 0,
            y: layout.iter().map(Layout::bottom).max().unwrap_or(0),
            row_height: 0,
        }
    }

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

pub async fn append_dashboard_charts_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    canvas_id: Uuid,
    removed_cell_ids: &[Uuid],
    charts: Vec<DashboardChartRequest>,
) -> Result<i32, sea_orm::DbErr> {
    let dashboard = repository::get_dashboard_by_canvas_id_transaction(txn, tenant_id, canvas_id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(DASHBOARD_NOT_FOUND_MESSAGE.into()))?;
    let mut layout = Layout::from_json(dashboard.layout.clone()).unwrap_or_default();
    layout.retain(|l| !removed_cell_ids.contains(&l.cell_id()));

    let mut packer = DashboardGridPacker::below(&layout);
    for chart in charts {
        let (x, y, width, height) = packer.place(chart.width, chart.height);
        layout.push(Layout::new(
            chart.cell_id,
            chart.notebook_id,
            x,
            y,
            width,
            height,
        ));
    }

    if layout.len() > constants::MAX_CHARTS_PER_DASHBOARD {
        return Err(sea_orm::DbErr::Custom(format!(
            "dashboard can hold at most {} charts",
            constants::MAX_CHARTS_PER_DASHBOARD
        )));
    }

    let count = layout.len() as i32;
    let layout = (!layout.is_empty()).then_some(layout);
    repository::update_dashboard_layout_transaction(txn, dashboard, layout).await?;
    Ok(count)
}

pub async fn delete_dashboard_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    let chart_cell_ids = match repository::get_dashboard_transaction(txn, tenant_id, id).await {
        Ok(Some(dashboard)) => dashboard_chart_cell_ids(&dashboard),
        Ok(None) | Err(_) => Vec::new(),
    };
    repository::delete_dashboard_transaction(txn, tenant_id, id).await?;
    ai_summary_service::delete_dashboard_summary(txn, tenant_id, id).await?;
    ai_summary_service::delete_chart_summaries(txn, tenant_id, &chart_cell_ids).await
}

pub async fn update_dashboard(
    app_state: AppState,
    tenant_context: &TenantContext,
    dashboard_id: Uuid,
    payload: dto::DashboardDto,
) -> Result<dto::DashboardDto, DashboardError> {
    authorize_dashboard(&app_state, tenant_context, dashboard_id, Action::Write).await?;

    let dashboard = execute_update_dashboard(
        &app_state,
        tenant_context.tenant_id(),
        dashboard_id,
        payload,
    )
    .await?;

    Ok(dto::DashboardDto::from(dashboard))
}

pub async fn remove_cell_from_layout_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    canvas_id: Uuid,
    cell_id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    let dashboard =
        match repository::get_dashboard_by_canvas_id_transaction(txn, tenant_id, canvas_id).await {
            Ok(Some(dashboard)) => dashboard,
            Ok(None) | Err(_) => return Ok(()),
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
    tenant_id: &TenantId,
    canvas_id: Uuid,
) -> Result<i32, sea_orm::DbErr> {
    let dashboard = get_dashboard_by_canvas_id_entity(app_state, tenant_id, canvas_id).await?;

    if let Some(layout_json) = dashboard.layout {
        let layout_value: serde_json::Value = layout_json;
        if let Ok(layout_items) = serde_json::from_value::<Vec<serde_json::Value>>(layout_value) {
            return Ok(layout_items.len() as i32);
        }
    }

    Ok(0)
}

pub async fn load_authorization_subject(
    app_state: &AppState,
    tenant_id: &TenantId,
    dashboard_id: Uuid,
) -> Result<crate::authorization::dto::AuthorizationSubject, sea_orm::DbErr> {
    let dashboard =
        repository::get_dashboard(&app_state.database_connection, tenant_id, dashboard_id)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound(DASHBOARD_NOT_FOUND_MESSAGE.into()))?;
    crate::canvas::service::load_authorization_subject(app_state, tenant_id, dashboard.canvas_id)
        .await
}

pub async fn authorize_dashboard(
    app_state: &AppState,
    tenant_context: &TenantContext,
    id: Uuid,
    action: Action,
) -> Result<entity::dashboard::Model, DashboardError> {
    let record = repository::get_dashboard(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        id,
    )
    .await
    .map_err(|_| DashboardError::NotFound)?
    .ok_or(DashboardError::NotFound)?;

    let subject = load_authorization_subject(app_state, tenant_context.tenant_id(), id)
        .await
        .map_err(|_| DashboardError::NotFound)?;

    crate::authorization::service::require_access(app_state, tenant_context, &subject, action)
        .await?;

    Ok(record)
}

async fn authorize_canvas_for_children(
    app_state: &AppState,
    tenant_context: &TenantContext,
    canvas_id: Uuid,
) -> Result<(), DashboardError> {
    let subject = crate::canvas::service::load_authorization_subject(
        app_state,
        tenant_context.tenant_id(),
        canvas_id,
    )
    .await
    .map_err(|_| DashboardError::CanvasNotFound)?;

    crate::authorization::service::require_access(
        app_state,
        tenant_context,
        &subject,
        crate::authorization::constants::Action::Read,
    )
    .await?;

    Ok(())
}
