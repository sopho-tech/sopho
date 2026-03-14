use crate::entity::configuration;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};

pub async fn find_by_key(
    db: &DatabaseConnection,
    key: &str,
) -> Result<Option<configuration::Model>, DbErr> {
    configuration::Entity::find_by_id(key.to_string())
        .one(db)
        .await
}

pub async fn update_value(
    db: &DatabaseConnection,
    model: configuration::Model,
    value: &str,
) -> Result<configuration::Model, DbErr> {
    let mut active_model: configuration::ActiveModel = model.into();
    active_model.value = Set(value.to_string());
    active_model.update(db).await
}
