use sea_orm::{ActiveModelTrait, EntityTrait, DatabaseConnection, DbErr};
use crate::entity::dashboard;
use uuid::Uuid;

pub async fn save_dashboard(db: &DatabaseConnection, dashboard: dashboard::Model) -> Result<dashboard::Model, DbErr> {
    let dashboard_active_model: dashboard::ActiveModel = dashboard.into();
    let dashboard_active_model = dashboard_active_model.insert(db).await?;
    Ok(dashboard_active_model.into())
}

pub async fn get_dashboard(db: &DatabaseConnection, id: Uuid) -> Result<dashboard::Model, DbErr> {
    let dashboard = dashboard::Entity::find_by_id(id).one(db).await?;
    match dashboard {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Dashboard not found".into())),
    }
}
