use crate::authentication::service::UserStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct TenantMembershipDto {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub role_name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TenantMemberDto {
    pub user_id: Uuid,
    pub full_name: String,
    pub email: String,
    pub role_name: String,
    pub account_status: UserStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserTenantMembershipDto {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub role_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTenantDto {
    pub name: String,
}
