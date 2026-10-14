use crate::authorization::constants::{MembershipStatus, TenantStatus};
use crate::authorization::dto::TenantId;
use crate::common::time_utils;
use crate::entity::{role, tenant, tenant_membership, user};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, Set,
};
use uuid::Uuid;

pub struct TenantMemberRow {
    pub membership: tenant_membership::Model,
    pub user: Option<user::Model>,
    pub role: Option<role::Model>,
}

pub struct TenantMembershipRow {
    pub membership: tenant_membership::Model,
    pub tenant: Option<tenant::Model>,
    pub role: Option<role::Model>,
}

pub async fn insert_tenant(
    database_connection: &DatabaseConnection,
    name: String,
) -> Result<tenant::Model, DbErr> {
    tenant::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name),
        status: Set(TenantStatus::Active.as_str().to_string()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(database_connection)
    .await
}

pub async fn insert_membership(
    database_connection: &DatabaseConnection,
    tenant_id: Uuid,
    user_id: Uuid,
    role_id: Uuid,
) -> Result<tenant_membership::Model, DbErr> {
    tenant_membership::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        user_id: Set(user_id),
        role_id: Set(role_id),
        status: Set(MembershipStatus::Active.as_str().to_string()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(database_connection)
    .await
}

pub async fn find_tenant_by_id(
    database_connection: &DatabaseConnection,
    tenant_id: Uuid,
) -> Result<Option<tenant::Model>, DbErr> {
    tenant::Entity::find_by_id(tenant_id)
        .one(database_connection)
        .await
}

pub async fn list_memberships_for_user(
    database_connection: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Vec<TenantMembershipRow>, DbErr> {
    let memberships = tenant_membership::Entity::find()
        .filter(tenant_membership::Column::UserId.eq(user_id))
        .filter(tenant_membership::Column::Status.eq(MembershipStatus::Active.as_str().to_string()))
        .all(database_connection)
        .await?;

    let mut rows = Vec::with_capacity(memberships.len());
    for membership in memberships {
        let tenant = find_tenant_by_id(database_connection, membership.tenant_id).await?;
        let role = role::Entity::find_by_id(membership.role_id)
            .one(database_connection)
            .await?;
        rows.push(TenantMembershipRow {
            membership,
            tenant,
            role,
        });
    }
    Ok(rows)
}

pub async fn count_tenants(database_connection: &DatabaseConnection) -> Result<u64, DbErr> {
    tenant::Entity::find().count(database_connection).await
}

pub async fn list_all_tenant_ids(
    database_connection: &DatabaseConnection,
) -> Result<Vec<Uuid>, DbErr> {
    Ok(tenant::Entity::find()
        .all(database_connection)
        .await?
        .into_iter()
        .map(|found| found.id)
        .collect())
}

pub async fn list_active_members(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
) -> Result<Vec<TenantMemberRow>, DbErr> {
    let memberships = tenant_membership::Entity::find()
        .filter(tenant_membership::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(tenant_membership::Column::Status.eq(MembershipStatus::Active.as_str().to_string()))
        .all(database_connection)
        .await?;

    let user_ids: Vec<Uuid> = memberships
        .iter()
        .map(|membership| membership.user_id)
        .collect();
    let role_ids: Vec<Uuid> = memberships
        .iter()
        .map(|membership| membership.role_id)
        .collect();

    let users = user::Entity::find()
        .filter(user::Column::Id.is_in(user_ids))
        .all(database_connection)
        .await?;
    let roles = role::Entity::find()
        .filter(role::Column::Id.is_in(role_ids))
        .filter(role::Column::TenantId.eq(tenant_id.get_uuid()))
        .all(database_connection)
        .await?;

    Ok(memberships
        .into_iter()
        .map(|membership| TenantMemberRow {
            user: users
                .iter()
                .find(|candidate| candidate.id == membership.user_id)
                .cloned(),
            role: roles
                .iter()
                .find(|candidate| candidate.id == membership.role_id)
                .cloned(),
            membership,
        })
        .collect())
}
