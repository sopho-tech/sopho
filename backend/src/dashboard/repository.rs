use crate::authorization::dto::TenantId;
use crate::common::time_utils;
use crate::dashboard::dto;
use crate::entity::dashboard;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, Set,
};
use uuid::Uuid;

pub async fn save_dashboard_connection(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    dashboard: dashboard::Model,
) -> Result<dashboard::Model, DbErr> {
    let mut dashboard_active_model: dashboard::ActiveModel = dashboard.into();
    dashboard_active_model.tenant_id = Set(tenant_id.get_uuid());
    dashboard_active_model.insert(db).await
}

pub async fn save_dashboard_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    dashboard: dashboard::Model,
) -> Result<dashboard::Model, DbErr> {
    let mut dashboard_active_model: dashboard::ActiveModel = dashboard.into();
    dashboard_active_model.tenant_id = Set(tenant_id.get_uuid());
    dashboard_active_model.insert(txn).await
}

pub async fn get_dashboard(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<Option<dashboard::Model>, DbErr> {
    dashboard::Entity::find()
        .filter(dashboard::Column::Id.eq(id))
        .filter(dashboard::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(db)
        .await
}

pub async fn get_dashboard_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<Option<dashboard::Model>, DbErr> {
    dashboard::Entity::find()
        .filter(dashboard::Column::Id.eq(id))
        .filter(dashboard::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(txn)
        .await
}

pub async fn get_dashboard_by_canvas_id(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    canvas_id: Uuid,
) -> Result<Option<dashboard::Model>, DbErr> {
    dashboard::Entity::find()
        .filter(dashboard::Column::CanvasId.eq(canvas_id))
        .filter(dashboard::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(db)
        .await
}

pub async fn get_dashboard_by_canvas_id_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    canvas_id: Uuid,
) -> Result<Option<dashboard::Model>, DbErr> {
    dashboard::Entity::find()
        .filter(dashboard::Column::CanvasId.eq(canvas_id))
        .filter(dashboard::Column::TenantId.eq(tenant_id.get_uuid()))
        .one(txn)
        .await
}

pub async fn delete_dashboard_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<(), DbErr> {
    dashboard::Entity::delete_many()
        .filter(dashboard::Column::Id.eq(id))
        .filter(dashboard::Column::TenantId.eq(tenant_id.get_uuid()))
        .exec(txn)
        .await?;
    Ok(())
}

pub async fn update_dashboard_layout_transaction(
    txn: &DatabaseTransaction,
    dashboard: dashboard::Model,
    layout: Option<Vec<dto::Layout>>,
) -> Result<dashboard::Model, DbErr> {
    let mut dashboard_active: dashboard::ActiveModel = dashboard.into();
    dashboard_active.layout = Set(dto::Layout::to_json(layout));
    dashboard_active.updated_at = Set(time_utils::now_utc_into());
    let result = dashboard_active.update(txn).await?;
    Ok(result)
}

pub async fn update_dashboard(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    dashboard_id: Uuid,
    payload: dto::DashboardDto,
) -> Result<Option<dashboard::Model>, DbErr> {
    let Some(dashboard) = get_dashboard(db, tenant_id, dashboard_id).await? else {
        return Ok(None);
    };
    let mut dashboard_entity: dashboard::ActiveModel = dashboard.into();
    if let Some(name) = payload.name {
        dashboard_entity.name = Set(name);
    }
    if let Some(description) = payload.description {
        dashboard_entity.description = Set(description);
    }
    dashboard_entity.status = Set(payload.status.to_string());
    dashboard_entity.layout = Set(dto::Layout::to_json(payload.layout));
    dashboard_entity.updated_at = Set(time_utils::now_utc_into());

    dashboard_entity.update(db).await.map(Some)
}
