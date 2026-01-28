use crate::authentication::constants::SessionStatus;
use crate::common::time_utils;
use crate::entity;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use uuid::Uuid;

pub async fn save_user(
    db: &DatabaseConnection,
    user: entity::user::Model,
) -> Result<entity::user::Model, DbErr> {
    let user_active_model: entity::user::ActiveModel = user.into();
    let user_active_model = user_active_model.insert(db).await?;
    Ok(user_active_model.into())
}

pub async fn get_user(db: &DatabaseConnection, id: Uuid) -> Result<entity::user::Model, DbErr> {
    let user = entity::user::Entity::find_by_id(id).one(db).await?;
    match user {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("User not found".into())),
    }
}

pub async fn get_user_by_email(
    db: &DatabaseConnection,
    email: String,
) -> Result<entity::user::Model, DbErr> {
    let user = entity::user::Entity::find()
        .filter(entity::user::Column::Email.eq(email))
        .one(db)
        .await?;
    match user {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("User not found".into())),
    }
}

pub async fn get_active_session_by_access_token(
    db: &DatabaseConnection,
    access_token: String,
) -> Result<entity::session::Model, DbErr> {
    let session = entity::session::Entity::find()
        .filter(entity::session::Column::AccessToken.eq(access_token))
        .filter(entity::session::Column::Status.eq(SessionStatus::Active.as_str()))
        .one(db)
        .await?;
    match session {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Session not found".into())),
    }
}

pub async fn get_active_session_by_refresh_token(
    db: &DatabaseConnection,
    refresh_token: String,
) -> Result<entity::session::Model, DbErr> {
    let session = entity::session::Entity::find()
        .filter(entity::session::Column::RefreshToken.eq(refresh_token))
        .filter(entity::session::Column::Status.eq(SessionStatus::Active.as_str()))
        .one(db)
        .await?;
    match session {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Session not found".into())),
    }
}

pub async fn create_session(
    db: &DatabaseConnection,
    session: entity::session::Model,
) -> Result<entity::session::Model, DbErr> {
    let session_active_model: entity::session::ActiveModel = session.into();
    let session_active_model = session_active_model.insert(db).await?;
    Ok(session_active_model.into())
}

pub async fn deactivate_session(
    db: &DatabaseConnection,
    session_id: Uuid,
) -> Result<entity::session::Model, DbErr> {
    let session = entity::session::Entity::find_by_id(session_id)
        .one(db)
        .await?;

    match session {
        Some(session_model) => {
            let mut session_active_model: entity::session::ActiveModel = session_model.into();
            session_active_model.status = sea_orm::Set(SessionStatus::Inactive.as_str().to_string());
            session_active_model.updated_at = sea_orm::Set(time_utils::now_utc_into());
            session_active_model.update(db).await
        }
        None => Err(DbErr::RecordNotFound("Session not found".into())),
    }
}
