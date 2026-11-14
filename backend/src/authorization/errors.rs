use thiserror::Error;

#[derive(Debug, Error)]
pub enum TenantContextError {
    #[error("Tenant not found")]
    NotAMember,
    #[error("Membership suspended")]
    MembershipSuspended,
    #[error("Tenant suspended")]
    TenantSuspended,
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("Unknown permission in role: {0}")]
    UnknownPermission(String),
}

#[derive(Debug, Error)]
pub enum AuthorizationError {
    #[error("Tenant not found")]
    WrongTenant,
    #[error("Resource not found")]
    NotReachable,
    #[error("Permission denied: {0}")]
    MissingPermission(String),
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}
