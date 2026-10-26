use crate::entity::{identity, oidc_login_attempt};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};
use uuid::Uuid;

pub async fn find_identity(
    database_connection: &DatabaseConnection,
    provider_name: &str,
    subject: &str,
) -> Result<Option<identity::Model>, DbErr> {
    identity::Entity::find()
        .filter(identity::Column::ProviderName.eq(provider_name))
        .filter(identity::Column::Subject.eq(subject))
        .one(database_connection)
        .await
}

pub async fn find_identities_for_user(
    database_connection: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Vec<identity::Model>, DbErr> {
    identity::Entity::find()
        .filter(identity::Column::UserId.eq(user_id))
        .all(database_connection)
        .await
}

pub async fn find_identities_for_users(
    database_connection: &DatabaseConnection,
    user_ids: Vec<Uuid>,
) -> Result<Vec<identity::Model>, DbErr> {
    identity::Entity::find()
        .filter(identity::Column::UserId.is_in(user_ids))
        .all(database_connection)
        .await
}

pub async fn delete_identity(
    connection: &impl ConnectionTrait,
    identity_id: Uuid,
) -> Result<u64, DbErr> {
    Ok(identity::Entity::delete_by_id(identity_id)
        .exec(connection)
        .await?
        .rows_affected)
}

pub async fn insert_identity(
    connection: &impl ConnectionTrait,
    identity: identity::Model,
) -> Result<identity::Model, DbErr> {
    let active: identity::ActiveModel = identity.into();
    active.insert(connection).await
}

pub async fn insert_login_attempt(
    database_connection: &DatabaseConnection,
    attempt: oidc_login_attempt::Model,
) -> Result<oidc_login_attempt::Model, DbErr> {
    let active: oidc_login_attempt::ActiveModel = attempt.into();
    active.insert(database_connection).await
}

pub async fn find_login_attempt_by_state(
    database_connection: &DatabaseConnection,
    state: &str,
) -> Result<Option<oidc_login_attempt::Model>, DbErr> {
    oidc_login_attempt::Entity::find()
        .filter(oidc_login_attempt::Column::State.eq(state))
        .one(database_connection)
        .await
}

pub async fn delete_login_attempt(
    database_connection: &DatabaseConnection,
    attempt_id: Uuid,
) -> Result<u64, DbErr> {
    let result = oidc_login_attempt::Entity::delete_many()
        .filter(oidc_login_attempt::Column::Id.eq(attempt_id))
        .exec(database_connection)
        .await?;
    Ok(result.rows_affected)
}

pub async fn delete_expired_login_attempts(
    database_connection: &DatabaseConnection,
    now: DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    let result = oidc_login_attempt::Entity::delete_many()
        .filter(oidc_login_attempt::Column::ExpiresAt.lte(now))
        .exec(database_connection)
        .await?;
    Ok(result.rows_affected)
}
