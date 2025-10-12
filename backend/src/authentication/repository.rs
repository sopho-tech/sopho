use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use uuid::Uuid;
use crate::entity;


pub async fn save_user(db: &DatabaseConnection, user: entity::user::Model) -> Result<entity::user::Model, DbErr> {
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

pub async fn get_user_by_email(db: &DatabaseConnection, email: String) -> Result<entity::user::Model, DbErr> {
    let user = entity::user::Entity::find().filter(entity::user::Column::Email.contains(email)).one(db).await?;
    match user {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("User not found".into())),
    }
}

pub async fn get_session_by_access_token(db: &DatabaseConnection, access_token: String) -> Result<entity::session::Model, DbErr> {
    let session = entity::session::Entity::find().filter(entity::session::Column::AccessToken.eq(access_token)).one(db).await?;
    match session {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound("Session not found".into())),
    }
}

pub async fn create_session(db: &DatabaseConnection, session: entity::session::Model) -> Result<entity::session::Model, DbErr> {
    let session_active_model: entity::session::ActiveModel = session.into();
    let session_active_model = session_active_model.insert(db).await?;
    Ok(session_active_model.into())
}

