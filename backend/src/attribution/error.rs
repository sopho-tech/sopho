use crate::authorization::constants::ActorKind;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AttributionError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("{0}")]
    UnknownActor(String),
    #[error("A system attribution names user {0}")]
    SystemNamesUser(Uuid),
    #[error("A {0} attribution names no user")]
    MissingUser(ActorKind),
    #[error("Attributed user {0} does not exist")]
    UnknownUser(Uuid),
}
