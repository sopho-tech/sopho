use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct TenantMembershipDto {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub role_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTenantDto {
    pub name: String,
}
