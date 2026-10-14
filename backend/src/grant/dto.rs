use crate::tenant::dto::TenantMemberDto;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateGrantDto {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub principal_user_id: Uuid,
    pub access_level: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVisibilityDto {
    pub visibility: String,
}

#[derive(Debug, Serialize)]
pub struct GrantDto {
    pub id: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub principal_user_id: Uuid,
    pub principal: Option<TenantMemberDto>,
    pub access_level: String,
}

#[derive(Debug, Serialize)]
pub struct ResourceSharingDto {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub visibility: String,
    pub owner: Option<TenantMemberDto>,
    pub grants: Vec<GrantDto>,
}
