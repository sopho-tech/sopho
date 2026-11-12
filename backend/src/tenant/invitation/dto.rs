use crate::email_delivery::dto::EmailDeliveryDto;
use crate::tenant::invitation::constants::{
    InvitationDeliveryMethod, InvitationLinkNextStep, InvitationStatus,
};
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
    pub role_display_name: String,
    pub status: InvitationStatus,
    pub invited_by_name: String,
    pub expires_at: DateTime<FixedOffset>,
    pub expiry_label: String,
    pub email_delivery: Option<EmailDeliveryDto>,
}

#[derive(Debug, Serialize)]
pub struct InvitationDeliveryDto {
    pub method: InvitationDeliveryMethod,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReceivedInvitationDto {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub role_name: String,
    pub role_display_name: String,
    pub invited_by_name: String,
    pub expires_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize)]
pub struct InvitationLinkTokenDto {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct InvitationSignUpDto {
    pub token: String,
    pub full_name: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct InvitationLinkDto {
    pub tenant_name: String,
    pub role_name: String,
    pub invited_by_name: String,
    pub email: String,
    pub expires_at: DateTime<FixedOffset>,
    pub next_step: InvitationLinkNextStep,
    pub password_sign_up_available: bool,
}

#[derive(Debug, Serialize)]
pub struct AcceptedInvitationDto {
    pub tenant_id: Uuid,
}
