use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionDto {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct SessionDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token: String,
    pub access_token: String,
    pub refresh_token_expires_at: DateTime<Utc>,
    pub access_token_expires_at: DateTime<Utc>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct UserDto {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: String,
}
