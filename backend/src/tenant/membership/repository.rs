use crate::authorization::constants::MembershipStatus;
use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::common::time_utils;
use crate::entity::{role, tenant_membership, user};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use uuid::Uuid;

pub struct TenantMemberRow {
    pub membership: tenant_membership::Model,
    pub user: Option<user::Model>,
    pub role: Option<role::Model>,
}

pub async fn insert_membership(
    database_connection: &impl ConnectionTrait,
    tenant_id: &TenantId,
    user_id: Uuid,
    role_id: Uuid,
    actor: &Actor,
) -> Result<tenant_membership::Model, DbErr> {
    tenant_membership::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id.get_uuid()),
        user_id: Set(user_id),
        role_id: Set(role_id),
        status: Set(MembershipStatus::Active.as_str().to_string()),
        created_by_user_id: Set(actor.user_id()),
        created_by_actor: Set(actor.kind_name()),
        modified_by_user_id: Set(actor.user_id()),
        modified_by_actor: Set(actor.kind_name()),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(database_connection)
    .await
}

pub async fn find_active_membership(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    user_id: Uuid,
) -> Result<Option<tenant_membership::Model>, DbErr> {
    tenant_membership::Entity::find()
        .filter(tenant_membership::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(tenant_membership::Column::UserId.eq(user_id))
        .filter(tenant_membership::Column::Status.eq(MembershipStatus::Active.as_str()))
        .one(database_connection)
        .await
}

pub async fn count_active_members_with_role(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    role_id: Uuid,
) -> Result<u64, DbErr> {
    tenant_membership::Entity::find()
        .filter(tenant_membership::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(tenant_membership::Column::RoleId.eq(role_id))
        .filter(tenant_membership::Column::Status.eq(MembershipStatus::Active.as_str()))
        .count(database_connection)
        .await
}

pub async fn update_membership_role(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    membership_id: Uuid,
    role_id: Uuid,
    actor: &Actor,
) -> Result<u64, DbErr> {
    let result = tenant_membership::Entity::update_many()
        .col_expr(tenant_membership::Column::RoleId, Expr::value(role_id))
        .col_expr(
            tenant_membership::Column::UpdatedAt,
            Expr::value(time_utils::now_utc_into::<
                sea_orm::prelude::DateTimeWithTimeZone,
            >()),
        )
        .col_expr(
            tenant_membership::Column::ModifiedByUserId,
            Expr::value(actor.user_id()),
        )
        .col_expr(
            tenant_membership::Column::ModifiedByActor,
            Expr::value(actor.kind_name()),
        )
        .filter(tenant_membership::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(tenant_membership::Column::Id.eq(membership_id))
        .exec(database_connection)
        .await?;
    Ok(result.rows_affected)
}

pub async fn delete_membership_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    membership_id: Uuid,
) -> Result<u64, DbErr> {
    let result = tenant_membership::Entity::delete_many()
        .filter(tenant_membership::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(tenant_membership::Column::Id.eq(membership_id))
        .exec(txn)
        .await?;
    Ok(result.rows_affected)
}

pub async fn list_active_members(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
) -> Result<Vec<TenantMemberRow>, DbErr> {
    let memberships = tenant_membership::Entity::find()
        .filter(tenant_membership::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(tenant_membership::Column::Status.eq(MembershipStatus::Active.as_str()))
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
