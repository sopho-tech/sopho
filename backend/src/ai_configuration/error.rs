use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiConfigurationError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("AI client construction failed: {0}")]
    ClientBuild(String),
    #[error("API key must not be empty")]
    EmptyApiKey,
    #[error("AI configuration not found")]
    NotFound,
}
