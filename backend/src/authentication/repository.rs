use crate::authentication::constants::{SessionStatus, UserStatus};
use crate::common::time_utils;
use crate::entity;
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

pub async fn save_user(
    db: &impl ConnectionTrait,
    user: entity::user::Model,
) -> Result<entity::user::Model, DbErr> {
    let user_active_model: entity::user::ActiveModel = user.into();
    let user_active_model = user_active_model.insert(db).await?;
    Ok(user_active_model)
}

pub async fn get_user(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<entity::user::Model>, DbErr> {
    entity::user::Entity::find_by_id(id).one(db).await
}

pub async fn get_users_by_ids(
    db: &DatabaseConnection,
    ids: Vec<Uuid>,
) -> Result<Vec<entity::user::Model>, DbErr> {
    entity::user::Entity::find()
        .filter(entity::user::Column::Id.is_in(ids))
        .all(db)
        .await
}

pub struct UserPage {
    pub users: Vec<entity::user::Model>,
    pub total: u64,
}

pub async fn find_users_page(
    database_connection: &DatabaseConnection,
    search: Option<String>,
    page_index: u64,
    page_size: u64,
) -> Result<UserPage, DbErr> {
    let users = entity::user::Entity::find();
    let users = match search {
        Some(search) => {
            let pattern = format!("%{}%", search.to_lowercase());
            users.filter(
                Condition::any()
                    .add(
                        Expr::expr(Func::lower(Expr::col(entity::user::Column::Email)))
                            .like(pattern.clone()),
                    )
                    .add(
                        Expr::expr(Func::lower(Expr::col(entity::user::Column::FullName)))
                            .like(pattern),
                    ),
            )
        }
        None => users,
    };
    let paginator = users
        .order_by_asc(entity::user::Column::FullName)
        .order_by_asc(entity::user::Column::Email)
        .paginate(database_connection, page_size);
    Ok(UserPage {
        total: paginator.num_items().await?,
        users: paginator.fetch_page(page_index).await?,
    })
}

pub async fn count_active_users_with_platform_role(
    database_connection: &DatabaseConnection,
    platform_role: &str,
) -> Result<u64, DbErr> {
    entity::user::Entity::find()
        .filter(entity::user::Column::PlatformRole.eq(platform_role))
        .filter(entity::user::Column::Status.eq(UserStatus::Active.as_str()))
        .count(database_connection)
        .await
}

pub async fn set_user_status(
    connection: &impl ConnectionTrait,
    user_id: Uuid,
    status: UserStatus,
    now: sea_orm::prelude::DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(entity::user::Entity::update_many()
        .col_expr(entity::user::Column::Status, Expr::value(status.as_str()))
        .col_expr(entity::user::Column::UpdatedAt, Expr::value(now))
        .filter(entity::user::Column::Id.eq(user_id))
        .exec(connection)
        .await?
        .rows_affected)
}

pub async fn get_user_by_email(
    db: &DatabaseConnection,
    email: String,
) -> Result<Option<entity::user::Model>, DbErr> {
    entity::user::Entity::find()
        .filter(entity::user::Column::Email.eq(email))
        .one(db)
        .await
}

pub async fn get_active_session_by_access_token(
    db: &DatabaseConnection,
    access_token_hash: String,
) -> Result<Option<entity::session::Model>, DbErr> {
    entity::session::Entity::find()
        .filter(entity::session::Column::AccessTokenHash.eq(access_token_hash))
        .filter(entity::session::Column::Status.eq(SessionStatus::Active.as_str()))
        .one(db)
        .await
}

pub async fn get_active_session_by_refresh_token(
    db: &DatabaseConnection,
    refresh_token_hash: String,
) -> Result<Option<entity::session::Model>, DbErr> {
    entity::session::Entity::find()
        .filter(entity::session::Column::RefreshTokenHash.eq(refresh_token_hash))
        .filter(entity::session::Column::Status.eq(SessionStatus::Active.as_str()))
        .one(db)
        .await
}

pub async fn create_session(
    db: &DatabaseConnection,
    session: entity::session::Model,
) -> Result<entity::session::Model, DbErr> {
    let session_active_model: entity::session::ActiveModel = session.into();
    let session_active_model = session_active_model.insert(db).await?;
    Ok(session_active_model)
}

pub async fn rotate_session_tokens(
    db: &DatabaseConnection,
    rotated_session: entity::session::Model,
    presented_refresh_token_hash: String,
) -> Result<Option<entity::session::Model>, DbErr> {
    let rotated_rows = entity::session::Entity::update_many()
        .col_expr(
            entity::session::Column::AccessTokenHash,
            Expr::value(rotated_session.access_token_hash.clone()),
        )
        .col_expr(
            entity::session::Column::RefreshTokenHash,
            Expr::value(rotated_session.refresh_token_hash.clone()),
        )
        .col_expr(
            entity::session::Column::AccessTokenExpiresAt,
            Expr::value(rotated_session.access_token_expires_at),
        )
        .col_expr(
            entity::session::Column::RefreshTokenExpiresAt,
            Expr::value(rotated_session.refresh_token_expires_at),
        )
        .col_expr(
            entity::session::Column::UpdatedAt,
            Expr::value(rotated_session.updated_at),
        )
        .filter(entity::session::Column::Id.eq(rotated_session.id))
        .filter(entity::session::Column::RefreshTokenHash.eq(presented_refresh_token_hash))
        .filter(entity::session::Column::Status.eq(SessionStatus::Active.as_str()))
        .exec(db)
        .await?
        .rows_affected;
    Ok((rotated_rows == 1).then_some(rotated_session))
}

pub async fn lock_user_row(connection: &impl ConnectionTrait, user_id: Uuid) -> Result<u64, DbErr> {
    Ok(entity::user::Entity::update_many()
        .col_expr(
            entity::user::Column::UpdatedAt,
            Expr::col(entity::user::Column::UpdatedAt).into(),
        )
        .filter(entity::user::Column::Id.eq(user_id))
        .exec(connection)
        .await?
        .rows_affected)
}

pub async fn update_password_hash(
    connection: &impl ConnectionTrait,
    user_id: Uuid,
    password_hash: String,
    now: sea_orm::prelude::DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(entity::user::Entity::update_many()
        .col_expr(
            entity::user::Column::PasswordHash,
            Expr::value(password_hash),
        )
        .col_expr(entity::user::Column::UpdatedAt, Expr::value(now))
        .filter(entity::user::Column::Id.eq(user_id))
        .exec(connection)
        .await?
        .rows_affected)
}

pub async fn increment_failed_sign_in_count(
    database_connection: &DatabaseConnection,
    user_id: Uuid,
    now: sea_orm::prelude::DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(entity::user::Entity::update_many()
        .col_expr(
            entity::user::Column::FailedSignInCount,
            Expr::col(entity::user::Column::FailedSignInCount).add(1),
        )
        .col_expr(entity::user::Column::UpdatedAt, Expr::value(now))
        .filter(entity::user::Column::Id.eq(user_id))
        .exec(database_connection)
        .await?
        .rows_affected)
}

pub async fn block_sign_in_after_failures(
    database_connection: &DatabaseConnection,
    user_id: Uuid,
    failure_limit: i32,
    blocked_until: sea_orm::prelude::DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(entity::user::Entity::update_many()
        .col_expr(entity::user::Column::FailedSignInCount, Expr::value(0))
        .col_expr(
            entity::user::Column::SignInBlockedUntil,
            Expr::value(blocked_until),
        )
        .filter(entity::user::Column::Id.eq(user_id))
        .filter(entity::user::Column::FailedSignInCount.gte(failure_limit))
        .exec(database_connection)
        .await?
        .rows_affected)
}

pub async fn clear_failed_sign_ins(
    connection: &impl ConnectionTrait,
    user_id: Uuid,
    now: sea_orm::prelude::DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(entity::user::Entity::update_many()
        .col_expr(entity::user::Column::FailedSignInCount, Expr::value(0))
        .col_expr(
            entity::user::Column::SignInBlockedUntil,
            Expr::value(Option::<sea_orm::prelude::DateTimeWithTimeZone>::None),
        )
        .col_expr(entity::user::Column::UpdatedAt, Expr::value(now))
        .filter(entity::user::Column::Id.eq(user_id))
        .exec(connection)
        .await?
        .rows_affected)
}

pub async fn deactivate_sessions_for_user(
    connection: &impl ConnectionTrait,
    user_id: Uuid,
    keep_session_id: Option<Uuid>,
    now: sea_orm::prelude::DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    let sessions_to_end = entity::session::Entity::update_many()
        .col_expr(
            entity::session::Column::Status,
            Expr::value(SessionStatus::Inactive.as_str()),
        )
        .col_expr(entity::session::Column::UpdatedAt, Expr::value(now))
        .filter(entity::session::Column::UserId.eq(user_id))
        .filter(entity::session::Column::Status.eq(SessionStatus::Active.as_str()));
    let sessions_to_end = match keep_session_id {
        Some(keep_session_id) => {
            sessions_to_end.filter(entity::session::Column::Id.ne(keep_session_id))
        }
        None => sessions_to_end,
    };
    Ok(sessions_to_end.exec(connection).await?.rows_affected)
}

pub async fn set_email_verified_if_missing(
    connection: &impl ConnectionTrait,
    user_id: Uuid,
    now: sea_orm::prelude::DateTimeWithTimeZone,
) -> Result<u64, DbErr> {
    Ok(entity::user::Entity::update_many()
        .col_expr(entity::user::Column::EmailVerifiedAt, Expr::value(now))
        .col_expr(entity::user::Column::UpdatedAt, Expr::value(now))
        .filter(entity::user::Column::Id.eq(user_id))
        .filter(entity::user::Column::EmailVerifiedAt.is_null())
        .exec(connection)
        .await?
        .rows_affected)
}

pub async fn deactivate_session(
    db: &DatabaseConnection,
    session_id: Uuid,
) -> Result<Option<entity::session::Model>, DbErr> {
    let session = entity::session::Entity::find_by_id(session_id)
        .one(db)
        .await?;

    match session {
        Some(session_model) => {
            let mut session_active_model: entity::session::ActiveModel = session_model.into();
            session_active_model.status =
                sea_orm::Set(SessionStatus::Inactive.as_str().to_string());
            session_active_model.updated_at = sea_orm::Set(time_utils::now_utc_into());
            session_active_model.update(db).await.map(Some)
        }
        None => Ok(None),
    }
}
