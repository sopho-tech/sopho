use crate::email_delivery::constants::EmailStatus;
use chrono::{DateTime, FixedOffset, Utc};
use serde::Serialize;
use uuid::Uuid;

pub struct EmailMessage {
    pub kind: String,
    pub recipient: String,
    pub subject: String,
    pub text_body: String,
    pub html_body: String,
    pub send_before: DateTime<Utc>,
    pub tenant_id: Option<Uuid>,
    pub requested_by_user_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct EmailLogEntryDto {
    pub id: Uuid,
    pub kind: String,
    pub recipient: String,
    pub subject: String,
    pub status: EmailStatus,
    pub attempt_count: i32,
    pub last_error: Option<String>,
    pub created_at: DateTime<FixedOffset>,
    pub next_attempt_at: DateTime<FixedOffset>,
    pub sent_at: Option<DateTime<FixedOffset>>,
    pub can_retry: bool,
}

#[derive(Debug, Serialize)]
pub struct EmailLogPageDto {
    pub emails: Vec<EmailLogEntryDto>,
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
    pub total_pages: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmailDeliveryDto {
    pub status: EmailStatus,
    pub last_error: Option<String>,
}
