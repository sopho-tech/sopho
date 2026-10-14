use crate::authorization::dto::TenantId;
use crate::common::time_utils;
use crate::entity::resource_grant;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, Set,
};
use uuid::Uuid;

pub async fn upsert_grant(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    resource_type: &str,
    resource_id: Uuid,
    principal_id: Uuid,
    access_level: &str,
    granted_by_user_id: Uuid,
) -> Result<resource_grant::Model, DbErr> {
    let existing = resource_grant::Entity::find()
        .filter(resource_grant::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(resource_grant::Column::ResourceType.eq(resource_type))
        .filter(resource_grant::Column::ResourceId.eq(resource_id))
        .filter(resource_grant::Column::PrincipalId.eq(principal_id))
        .one(db)
        .await?;

    match existing {
        Some(found) => {
            let mut active: resource_grant::ActiveModel = found.into();
            active.access_level = Set(access_level.to_string());
            active.updated_at = Set(time_utils::now_utc_into());
            active.update(db).await
        }
        None => {
            resource_grant::ActiveModel {
                id: Set(Uuid::new_v4()),
                tenant_id: Set(tenant_id.get_uuid()),
                resource_type: Set(resource_type.to_string()),
                resource_id: Set(resource_id),
                principal_type: Set(crate::authorization::constants::PrincipalType::User
                    .as_str()
                    .to_string()),
                principal_id: Set(principal_id),
                access_level: Set(access_level.to_string()),
                granted_by_user_id: Set(granted_by_user_id),
                created_at: Set(time_utils::now_utc_into()),
                updated_at: Set(time_utils::now_utc_into()),
            }
            .insert(db)
            .await
        }
    }
}

pub async fn list_grants(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    resource_type: &str,
    resource_id: Uuid,
) -> Result<Vec<resource_grant::Model>, DbErr> {
    resource_grant::Entity::find()
        .filter(resource_grant::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(resource_grant::Column::ResourceType.eq(resource_type))
        .filter(resource_grant::Column::ResourceId.eq(resource_id))
        .all(db)
        .await
}

pub async fn revoke_grant(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    grant_id: Uuid,
) -> Result<u64, DbErr> {
    let result = resource_grant::Entity::delete_many()
        .filter(resource_grant::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(resource_grant::Column::Id.eq(grant_id))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}

pub async fn delete_grants_for_principal_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    principal_id: Uuid,
) -> Result<u64, DbErr> {
    let result = resource_grant::Entity::delete_many()
        .filter(resource_grant::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(resource_grant::Column::PrincipalId.eq(principal_id))
        .exec(txn)
        .await?;
    Ok(result.rows_affected)
}

pub async fn find_grant_by_id(
    db: &DatabaseConnection,
    tenant_id: &TenantId,
    grant_id: Uuid,
) -> Result<Option<resource_grant::Model>, DbErr> {
    resource_grant::Entity::find()
        .filter(resource_grant::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(resource_grant::Column::Id.eq(grant_id))
        .one(db)
        .await
}
