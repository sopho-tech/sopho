use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateGrantDto {
    pub resource_type: String,
    pub resource_id: Uuid,
    pub principal_user_id: Uuid,
    pub access_level: String,
}

#[derive(Debug, Serialize)]
pub struct GrantDto {
    pub id: Uuid,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub principal_user_id: Uuid,
    pub access_level: String,
}

impl From<crate::entity::resource_grant::Model> for GrantDto {
    fn from(model: crate::entity::resource_grant::Model) -> Self {
        GrantDto {
            id: model.id,
            resource_type: model.resource_type,
            resource_id: model.resource_id,
            principal_user_id: model.principal_id,
            access_level: model.access_level,
        }
    }
}
