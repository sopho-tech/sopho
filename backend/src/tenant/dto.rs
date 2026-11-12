use crate::authentication::service::UserStatus;
use crate::authorization::errors::AuthorizationError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct TenantMembershipDto {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub role_name: String,
    pub role_display_name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TenantMemberDto {
    pub user_id: Uuid,
    pub full_name: String,
    pub email: String,
    pub role_name: String,
    pub role_display_name: String,
    pub account_status: UserStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserTenantMembershipDto {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub role_name: String,
    pub role_display_name: String,
}

#[derive(Debug, Serialize)]
pub struct TenantCreationCapabilityDto {
    pub can_create_tenant: bool,
    pub limit_reached_message: Option<String>,
}

pub enum TenantCreationDecision {
    AllowedByPlatformPermission,
    AllowedBySelfServe,
    RefusedWithoutPermission {
        authorization_error: AuthorizationError,
    },
    RefusedAtLimit {
        limit: u64,
    },
}

#[derive(Debug, Deserialize)]
pub struct CreateTenantDto {
    pub name: String,
}
