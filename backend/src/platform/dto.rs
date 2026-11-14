use crate::authentication::service::UserStatus;
use crate::authorization::constants::TenantStatus;
use crate::email_delivery::constants::EmailStatus;
use crate::email_template::constants::EmailKind;
use crate::tenant::dto::UserTenantMembershipDto;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub search: Option<String>,
    pub page: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct EmailLogQuery {
    pub status: Option<EmailStatus>,
    pub search: Option<String>,
    pub page: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct PlatformUserCapabilities {
    pub can_deactivate: bool,
    pub can_reactivate: bool,
    pub can_send_password_reset: bool,
    pub can_appoint_owner: bool,
    pub can_make_administrator: bool,
    pub can_remove_administrator: bool,
}

#[derive(Debug, Serialize)]
pub struct PlatformUserDto {
    pub id: Uuid,
    pub full_name: String,
    pub email: String,
    pub is_current_user: bool,
    pub is_platform_administrator: bool,
    pub status: UserStatus,
    pub has_password: bool,
    pub can_sign_in_with_password: bool,
    pub provider_names: Vec<String>,
    pub tenants: Vec<UserTenantMembershipDto>,
    pub created_at: DateTime<FixedOffset>,
    pub capabilities: PlatformUserCapabilities,
}

#[derive(Debug, Serialize)]
pub struct PlatformUserPageDto {
    pub users: Vec<PlatformUserDto>,
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
    pub total_pages: u64,
}

#[derive(Debug, Serialize)]
pub struct TenantSummaryDto {
    pub tenant_id: Uuid,
    pub tenant_name: String,
}

#[derive(Debug, Serialize)]
pub struct DeactivatedUserDto {
    pub tenants_without_active_owner: Vec<TenantSummaryDto>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct PlatformTenantCapabilities {
    pub can_suspend: bool,
    pub can_restore: bool,
}

#[derive(Debug, Serialize)]
pub struct PlatformTenantDto {
    pub tenant_id: Uuid,
    pub name: String,
    pub status: TenantStatus,
    pub capabilities: PlatformTenantCapabilities,
    pub active_owner_count: u64,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct EmailTemplatePreviewDto {
    pub kind: EmailKind,
    pub kind_display_name: String,
    pub sent_when: String,
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}
