use crate::authorization::constants::{MembershipStatus, ResourceType};
use crate::authorization::dto::TenantId;
use crate::common::time_utils;
use crate::entity::{resource_grant, role, role_permission, tenant_membership};
use sea_orm::sea_query::{Expr, Query, SelectStatement};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

pub fn grant_resource_ids_subquery(
    tenant_id: &TenantId,
    principal_id: Uuid,
    resource_type: ResourceType,
) -> SelectStatement {
    Query::select()
        .column(resource_grant::Column::ResourceId)
        .from(resource_grant::Entity)
        .and_where(Expr::col(resource_grant::Column::ResourceType).eq(resource_type.as_str()))
        .and_where(Expr::col(resource_grant::Column::PrincipalId).eq(principal_id))
        .and_where(Expr::col(resource_grant::Column::TenantId).eq(tenant_id.get_uuid()))
        .to_owned()
}

pub async fn find_role_by_name(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    name: &str,
) -> Result<Option<role::Model>, DbErr> {
    role::Entity::find()
        .filter(role::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(role::Column::Name.eq(name))
        .one(database_connection)
        .await
}

pub async fn insert_system_role(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    name: &str,
) -> Result<role::Model, DbErr> {
    role::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id.get_uuid()),
        name: Set(name.to_string()),
        is_system: Set(true),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(database_connection)
    .await
}

pub async fn find_permissions_for_role(
    database_connection: &DatabaseConnection,
    role_id: Uuid,
) -> Result<Vec<role_permission::Model>, DbErr> {
    role_permission::Entity::find()
        .filter(role_permission::Column::RoleId.eq(role_id))
        .all(database_connection)
        .await
}

pub async fn insert_role_permission(
    database_connection: &DatabaseConnection,
    role_id: Uuid,
    permission: &str,
) -> Result<role_permission::Model, DbErr> {
    role_permission::ActiveModel {
        id: Set(Uuid::new_v4()),
        role_id: Set(role_id),
        permission: Set(permission.to_string()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(database_connection)
    .await
}

pub async fn find_membership(
    database_connection: &DatabaseConnection,
    tenant_id: Uuid,
    user_id: Uuid,
) -> Result<Option<tenant_membership::Model>, DbErr> {
    tenant_membership::Entity::find()
        .filter(tenant_membership::Column::TenantId.eq(tenant_id))
        .filter(tenant_membership::Column::UserId.eq(user_id))
        .one(database_connection)
        .await
        .map(|membership| {
            membership.filter(|found| MembershipStatus::from_str(&found.status).is_ok())
        })
}

pub async fn find_grant(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    resource_type: &str,
    resource_id: Uuid,
    principal_id: Uuid,
) -> Result<Option<resource_grant::Model>, DbErr> {
    resource_grant::Entity::find()
        .filter(resource_grant::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(resource_grant::Column::ResourceType.eq(resource_type))
        .filter(resource_grant::Column::ResourceId.eq(resource_id))
        .filter(resource_grant::Column::PrincipalId.eq(principal_id))
        .one(database_connection)
        .await
}
