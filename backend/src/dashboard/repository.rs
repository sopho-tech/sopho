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
    dashboard: dashboard::Model,
) -> Result<dashboard::Model, DbErr> {
    let dashboard_active_model: dashboard::ActiveModel = dashboard.into();
    let dashboard_active_model = dashboard_active_model.insert(db).await?;
    Ok(dashboard_active_model.into())
}

pub async fn save_dashboard_transaction(
    txn: &DatabaseTransaction,
    dashboard: dashboard::Model,
) -> Result<dashboard::Model, DbErr> {
    let dashboard_active_model: dashboard::ActiveModel = dashboard.into();
    let dashboard_active_model = dashboard_active_model.insert(txn).await?;
    Ok(dashboard_active_model.into())
}

pub async fn get_dashboard(db: &DatabaseConnection, id: Uuid) -> Result<dashboard::Model, DbErr> {
    let dashboard = dashboard::Entity::find_by_id(id).one(db).await?;
    match dashboard {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Dashboard not found".into())),
    }
}

pub async fn get_dashboard_by_canvas_id(
    db: &DatabaseConnection,
    canvas_id: Uuid,
) -> Result<dashboard::Model, DbErr> {
    let dashboard = dashboard::Entity::find()
        .filter(dashboard::Column::CanvasId.eq(canvas_id))
        .one(db)
        .await?;
    match dashboard {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Dashboard not found".into())),
    }
}

pub async fn get_dashboard_by_canvas_id_transaction(
    txn: &DatabaseTransaction,
    canvas_id: Uuid,
) -> Result<dashboard::Model, DbErr> {
    let dashboard = dashboard::Entity::find()
        .filter(dashboard::Column::CanvasId.eq(canvas_id))
        .one(txn)
        .await?;
    match dashboard {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Dashboard not found".into())),
    }
}

pub async fn delete_dashboard_transaction(
    txn: &DatabaseTransaction,
    id: Uuid,
) -> Result<(), DbErr> {
    dashboard::Entity::delete_by_id(id).exec(txn).await?;
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
    Ok(result.into())
}

pub async fn update_dashboard(
    db: &DatabaseConnection,
    dashboard_id: Uuid,
    payload: dto::DashboardDto,
) -> Result<dashboard::Model, DbErr> {
    let dashboard = get_dashboard(&db, dashboard_id).await;
    match dashboard {
        Ok(dashboard) => {
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

            let dashboard_entity = dashboard_entity.update(db).await;
            return dashboard_entity;
        }
        Err(e) => Err(e),
    }
}
