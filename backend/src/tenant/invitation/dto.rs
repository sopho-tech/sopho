use crate::tenant::invitation::constants::InvitationStatus;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateInvitationDto {
    pub email: String,
    pub role_name: String,
}

#[derive(Debug, Serialize)]
pub struct InvitationDto {
    pub id: Uuid,
    pub email: String,
    pub role_name: String,
    pub status: InvitationStatus,
    pub invited_by_name: String,
    pub expires_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct ReceivedInvitationDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub role_name: String,
    pub invited_by_name: String,
    pub expires_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct AcceptedInvitationDto {
    pub tenant_id: Uuid,
}
