use crate::email_template::dto::RenderedEmail;
use crate::entity::{password_reset_token, user};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequestDto {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetDto {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordDto {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetRequestedDto {
    pub message: String,
}

pub struct PreparedResetEmail {
    pub rendered_email: RenderedEmail,
    pub send_before: DateTime<Utc>,
    pub reset_token: Option<password_reset_token::Model>,
}

pub struct QueuedAccountEmail<'a> {
    pub user: &'a user::Model,
    pub rendered_email: RenderedEmail,
    pub send_before: DateTime<Utc>,
    pub requested_by_user_id: Option<Uuid>,
}
