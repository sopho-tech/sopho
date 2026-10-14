use crate::authorization::dto::TenantId;
use crate::common::time_utils;
use crate::entity::tenant_invitation;
use crate::tenant::invitation::constants::InvitationStatus;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, Set,
};
use uuid::Uuid;

pub struct NewInvitation<'a> {
    pub email: &'a str,
    pub role_id: Uuid,
    pub expires_at: DateTimeWithTimeZone,
    pub invited_by_user_id: Uuid,
}

pub async fn insert_invitation(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    invitation: NewInvitation<'_>,
) -> Result<tenant_invitation::Model, DbErr> {
    tenant_invitation::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id.get_uuid()),
        email: Set(invitation.email.to_string()),
        role_id: Set(invitation.role_id),
        status: Set(InvitationStatus::Pending.as_str().to_string()),
        expires_at: Set(invitation.expires_at),
        invited_by_user_id: Set(invitation.invited_by_user_id),
        accepted_by_user_id: Set(None),
        created_at: Set(time_utils::now_utc_into()),
        updated_at: Set(time_utils::now_utc_into()),
    }
    .insert(database_connection)
    .await
}

pub async fn find_unexpired_pending_invitation(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    email: &str,
    now: DateTimeWithTimeZone,
) -> Result<Option<tenant_invitation::Model>, DbErr> {
    tenant_invitation::Entity::find()
        .filter(tenant_invitation::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(tenant_invitation::Column::Email.eq(email))
        .filter(tenant_invitation::Column::Status.eq(InvitationStatus::Pending.as_str()))
        .filter(tenant_invitation::Column::ExpiresAt.gt(now))
        .one(database_connection)
        .await
}

pub async fn list_pending_invitations(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
) -> Result<Vec<tenant_invitation::Model>, DbErr> {
    tenant_invitation::Entity::find()
        .filter(tenant_invitation::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(tenant_invitation::Column::Status.eq(InvitationStatus::Pending.as_str()))
        .all(database_connection)
        .await
}

pub async fn list_unexpired_pending_invitations_for_email(
    database_connection: &DatabaseConnection,
    email: &str,
    now: DateTimeWithTimeZone,
) -> Result<Vec<tenant_invitation::Model>, DbErr> {
    tenant_invitation::Entity::find()
        .filter(tenant_invitation::Column::Email.eq(email))
        .filter(tenant_invitation::Column::Status.eq(InvitationStatus::Pending.as_str()))
        .filter(tenant_invitation::Column::ExpiresAt.gt(now))
        .all(database_connection)
        .await
}

pub async fn find_invitation_by_id(
    database_connection: &DatabaseConnection,
    invitation_id: Uuid,
) -> Result<Option<tenant_invitation::Model>, DbErr> {
    tenant_invitation::Entity::find_by_id(invitation_id)
        .one(database_connection)
        .await
}

pub async fn revoke_pending_invitation(
    database_connection: &DatabaseConnection,
    tenant_id: &TenantId,
    invitation_id: Uuid,
) -> Result<u64, DbErr> {
    let result = tenant_invitation::Entity::update_many()
        .col_expr(
            tenant_invitation::Column::Status,
            Expr::value(InvitationStatus::Revoked.as_str()),
        )
        .col_expr(
            tenant_invitation::Column::UpdatedAt,
            Expr::value(time_utils::now_utc_into::<DateTimeWithTimeZone>()),
        )
        .filter(tenant_invitation::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(tenant_invitation::Column::Id.eq(invitation_id))
        .filter(tenant_invitation::Column::Status.eq(InvitationStatus::Pending.as_str()))
        .exec(database_connection)
        .await?;
    Ok(result.rows_affected)
}

pub async fn revoke_pending_invitations_sent_by_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    invited_by_user_id: Uuid,
) -> Result<u64, DbErr> {
    let result = tenant_invitation::Entity::update_many()
        .col_expr(
            tenant_invitation::Column::Status,
            Expr::value(InvitationStatus::Revoked.as_str()),
        )
        .col_expr(
            tenant_invitation::Column::UpdatedAt,
            Expr::value(time_utils::now_utc_into::<DateTimeWithTimeZone>()),
        )
        .filter(tenant_invitation::Column::TenantId.eq(tenant_id.get_uuid()))
        .filter(tenant_invitation::Column::InvitedByUserId.eq(invited_by_user_id))
        .filter(tenant_invitation::Column::Status.eq(InvitationStatus::Pending.as_str()))
        .exec(txn)
        .await?;
    Ok(result.rows_affected)
}

pub async fn accept_pending_invitation_transaction(
    txn: &DatabaseTransaction,
    invitation_id: Uuid,
    accepted_by_user_id: Uuid,
) -> Result<u64, DbErr> {
    let result = tenant_invitation::Entity::update_many()
        .col_expr(
            tenant_invitation::Column::Status,
            Expr::value(InvitationStatus::Accepted.as_str()),
        )
        .col_expr(
            tenant_invitation::Column::AcceptedByUserId,
            Expr::value(accepted_by_user_id),
        )
        .col_expr(
            tenant_invitation::Column::UpdatedAt,
            Expr::value(time_utils::now_utc_into::<DateTimeWithTimeZone>()),
        )
        .filter(tenant_invitation::Column::Id.eq(invitation_id))
        .filter(tenant_invitation::Column::Status.eq(InvitationStatus::Pending.as_str()))
        .exec(txn)
        .await?;
    Ok(result.rows_affected)
}
