use crate::entity::dashboard;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter,
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
