use crate::entity::password_reset_token;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};
use uuid::Uuid;

pub async fn insert_reset_token(
    connection: &impl ConnectionTrait,
    reset_token: password_reset_token::Model,
) -> Result<password_reset_token::Model, DbErr> {
    let reset_token_active_model: password_reset_token::ActiveModel = reset_token.into();
    reset_token_active_model.insert(connection).await
}

pub async fn find_unused_reset_token_by_hash(
    database_connection: &DatabaseConnection,
    token_hash: &str,
) -> Result<Option<password_reset_token::Model>, DbErr> {
    password_reset_token::Entity::find()
        .filter(password_reset_token::Column::TokenHash.eq(token_hash))
        .filter(password_reset_token::Column::UsedAt.is_null())
        .one(database_connection)
        .await
}

pub async fn use_up_reset_tokens_for_user(
    connection: &impl ConnectionTrait,
    user_id: Uuid,
    now: DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(password_reset_token::Entity::update_many()
        .col_expr(password_reset_token::Column::UsedAt, Expr::value(now))
        .col_expr(password_reset_token::Column::UpdatedAt, Expr::value(now))
        .filter(password_reset_token::Column::UserId.eq(user_id))
        .filter(password_reset_token::Column::UsedAt.is_null())
        .exec(connection)
        .await?
        .rows_affected)
}

pub async fn use_up_reset_token(
    connection: &impl ConnectionTrait,
    reset_token_id: Uuid,
    now: DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(password_reset_token::Entity::update_many()
        .col_expr(password_reset_token::Column::UsedAt, Expr::value(now))
        .col_expr(password_reset_token::Column::UpdatedAt, Expr::value(now))
        .filter(password_reset_token::Column::Id.eq(reset_token_id))
        .filter(password_reset_token::Column::UsedAt.is_null())
        .exec(connection)
        .await?
        .rows_affected)
}
