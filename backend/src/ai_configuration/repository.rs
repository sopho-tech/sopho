use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::common::time_utils;
use crate::entity::ai_configuration;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, Set};

pub async fn find(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
) -> Result<Option<ai_configuration::Model>, DbErr> {
    ai_configuration::Entity::find_by_id(tenant_id.get_uuid())
        .one(db)
        .await
}

pub async fn upsert(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    provider_db: &str,
    api_key_encrypted: &str,
    liveness_status_db: &str,
    last_checked_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    actor: &Actor,
) -> Result<ai_configuration::Model, DbErr> {
    let now = time_utils::now_utc_into();
    if let Some(existing) = find(db, tenant_id).await? {
        let mut active: ai_configuration::ActiveModel = existing.into();
        active.provider = Set(provider_db.to_string());
        active.api_key_encrypted = Set(api_key_encrypted.to_string());
        active.liveness_status = Set(liveness_status_db.to_string());
        active.last_checked_at = Set(last_checked_at);
        active.updated_at = Set(now);
        active.modified_by_user_id = Set(actor.user_id());
        active.modified_by_actor = Set(actor.kind_name());
        active.update(db).await
    } else {
        let model = ai_configuration::ActiveModel {
            tenant_id: Set(tenant_id.get_uuid()),
            provider: Set(provider_db.to_string()),
            api_key_encrypted: Set(api_key_encrypted.to_string()),
            liveness_status: Set(liveness_status_db.to_string()),
            last_checked_at: Set(last_checked_at),
            created_by_user_id: Set(actor.user_id()),
            created_by_actor: Set(actor.kind_name()),
            modified_by_user_id: Set(actor.user_id()),
            modified_by_actor: Set(actor.kind_name()),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(db).await
    }
}

pub async fn delete(db: &DatabaseConnection, tenant_id: &TenantId) -> Result<u64, DbErr> {
    let result = ai_configuration::Entity::delete_by_id(tenant_id.get_uuid())
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}
