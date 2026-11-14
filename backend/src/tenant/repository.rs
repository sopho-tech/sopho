use crate::authorization::constants::{MembershipStatus, TenantStatus};
use crate::authorization::dto::Actor;
use crate::common::time_utils;
use crate::entity::{role, self_serve_tenant_creation, tenant, tenant_membership};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

pub struct TenantMembershipRow {
    pub membership: tenant_membership::Model,
    pub tenant: Option<tenant::Model>,
    pub role: Option<role::Model>,
}

pub async fn insert_tenant(
    database_connection: &DatabaseConnection,
    name: String,
    actor: &Actor,
) -> Result<tenant::Model, DbErr> {
    tenant::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name),
        status: Set(TenantStatus::Active.as_str().to_string()),
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

pub async fn update_tenant_status(
    database_connection: &DatabaseConnection,
    tenant_id: Uuid,
    current_tenant_status: TenantStatus,
    tenant_status: TenantStatus,
    actor: &Actor,
) -> Result<u64, DbErr> {
    let update_result = tenant::Entity::update_many()
        .col_expr(
            tenant::Column::Status,
            Expr::value(tenant_status.as_str().to_string()),
        )
        .col_expr(
            tenant::Column::ModifiedByUserId,
            Expr::value(actor.user_id()),
        )
        .col_expr(
            tenant::Column::ModifiedByActor,
            Expr::value(actor.kind_name()),
        )
        .col_expr(
            tenant::Column::UpdatedAt,
            Expr::value(time_utils::now_utc_into::<
                sea_orm::prelude::DateTimeWithTimeZone,
            >()),
        )
        .filter(tenant::Column::Id.eq(tenant_id))
        .filter(tenant::Column::Status.eq(current_tenant_status.as_str()))
        .exec(database_connection)
        .await?;
    Ok(update_result.rows_affected)
}

pub async fn list_tenant_ids_with_status(
    database_connection: &DatabaseConnection,
    tenant_status: TenantStatus,
) -> Result<Vec<Uuid>, DbErr> {
    Ok(tenant::Entity::find()
        .filter(tenant::Column::Status.eq(tenant_status.as_str()))
        .all(database_connection)
        .await?
        .into_iter()
        .map(|found_tenant| found_tenant.id)
        .collect())
}

pub async fn find_tenant_by_id(
    database_connection: &DatabaseConnection,
    tenant_id: Uuid,
) -> Result<Option<tenant::Model>, DbErr> {
    tenant::Entity::find_by_id(tenant_id)
        .one(database_connection)
        .await
}

pub async fn find_tenants_by_ids(
    database_connection: &DatabaseConnection,
    tenant_ids: Vec<Uuid>,
) -> Result<Vec<tenant::Model>, DbErr> {
    tenant::Entity::find()
        .filter(tenant::Column::Id.is_in(tenant_ids))
        .all(database_connection)
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

pub async fn list_all_tenants(
    database_connection: &DatabaseConnection,
) -> Result<Vec<tenant::Model>, DbErr> {
    tenant::Entity::find()
        .order_by_asc(tenant::Column::Name)
        .all(database_connection)
        .await
}

pub async fn list_memberships_for_users(
    database_connection: &DatabaseConnection,
    user_ids: Vec<Uuid>,
) -> Result<Vec<TenantMembershipRow>, DbErr> {
    let memberships = tenant_membership::Entity::find()
        .filter(tenant_membership::Column::UserId.is_in(user_ids))
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

pub async fn insert_self_serve_tenant_creation(
    database_connection: &DatabaseConnection,
    creator_user_id: Uuid,
    created_tenant_id: Uuid,
) -> Result<self_serve_tenant_creation::Model, DbErr> {
    self_serve_tenant_creation::ActiveModel {
        id: Set(Uuid::new_v4()),
        creator_user_id: Set(creator_user_id),
        created_tenant_id: Set(created_tenant_id),
        created_at: Set(time_utils::now_utc_into()),
    }
    .insert(database_connection)
    .await
}

pub async fn count_self_serve_tenant_creations(
    database_connection: &DatabaseConnection,
    creator_user_id: Uuid,
) -> Result<u64, DbErr> {
    self_serve_tenant_creation::Entity::find()
        .filter(self_serve_tenant_creation::Column::CreatorUserId.eq(creator_user_id))
        .count(database_connection)
        .await
}
