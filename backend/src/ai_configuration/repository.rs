use crate::ai_configuration::constants::SENTINEL_ID;
use crate::common::time_utils;
use crate::entity::ai_configuration;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

pub async fn find(
    db: &DatabaseConnection,
) -> Result<Option<ai_configuration::Model>, DbErr> {
    ai_configuration::Entity::find_by_id(SENTINEL_ID.to_string())
        .one(db)
        .await
}

pub async fn upsert(
    db: &DatabaseConnection,
    provider_db: &str,
    api_key_encrypted: &str,
    liveness_status_db: &str,
    last_checked_at: Option<chrono::DateTime<chrono::FixedOffset>>,
) -> Result<ai_configuration::Model, DbErr> {
    let now = time_utils::now_utc_into();
    if let Some(existing) = find(db).await? {
        let mut active: ai_configuration::ActiveModel = existing.into();
        active.provider = Set(provider_db.to_string());
        active.api_key_encrypted = Set(api_key_encrypted.to_string());
        active.liveness_status = Set(liveness_status_db.to_string());
        active.last_checked_at = Set(last_checked_at);
        active.updated_at = Set(now);
        active.update(db).await
    } else {
        let model = ai_configuration::ActiveModel {
            id: Set(SENTINEL_ID.to_string()),
            provider: Set(provider_db.to_string()),
            api_key_encrypted: Set(api_key_encrypted.to_string()),
            liveness_status: Set(liveness_status_db.to_string()),
            last_checked_at: Set(last_checked_at),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(db).await
    }
}

pub async fn delete(db: &DatabaseConnection) -> Result<u64, DbErr> {
    let result = ai_configuration::Entity::delete_many()
        .filter(ai_configuration::Column::Id.eq(SENTINEL_ID))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}
