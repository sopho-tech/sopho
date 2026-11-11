use crate::authorization::constants::ActorKind;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttributedUserDto {
    pub user_id: Uuid,
    pub full_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttributionDto {
    pub actor: ActorKind,
    pub user: Option<AttributedUserDto>,
}
