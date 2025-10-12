use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionDto {
    pub provider: String,
    pub code: String,
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
pub struct GoogleUserDto {
    pub id: Uuid,
    pub google_id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub locale: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
}