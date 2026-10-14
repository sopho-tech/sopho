use crate::tenant::dto::TenantMemberDto;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct MemberCapabilities {
    pub can_change_role: bool,
    pub can_remove: bool,
}

#[derive(Debug, Serialize)]
pub struct MemberDto {
    #[serde(flatten)]
    pub member: TenantMemberDto,
    pub capabilities: MemberCapabilities,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRoleDto {
    pub role_name: String,
}

#[derive(Debug, Serialize)]
pub struct RoleDto {
    pub name: String,
    pub is_system: bool,
    pub assignable: bool,
}
