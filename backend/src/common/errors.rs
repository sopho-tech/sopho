use thiserror::Error;

#[derive(Error, Debug)]
#[error("{0}")]
pub struct ValidationError(pub String);

#[derive(Error, Debug)]
pub enum ExecuteQueryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Unhandled data type: {0}")]
    UnhandledDataType(String),
}

#[derive(Debug, thiserror::Error)]
pub enum CreateConnectionError {
    #[error("Failed to get connection status: {0}")]
    ConnectionStatus(#[from] ValidationError),
    #[error("Failed to encrypt connection")]
    Encryption,
    #[error("Failed to save connection: {0}")]
    Repository(#[from] sea_orm::DbErr),
}

#[derive(Debug, thiserror::Error)]
pub enum CreateCellError {
    #[error("Notebook not found")]
    NotebookNotFound,
    #[error("Failed to generate cell name: {0}")]
    Sopho(#[from] SophoError),
    #[error("Failed to save cell: {0}")]
    Repository(#[from] sea_orm::DbErr),
}

#[derive(Error, Debug)]
pub enum SophoError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    #[error("Notebook not found")]
    NotebookNotFound,
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Unsupported cell type for content parsing")]
    UnsupportedCellType,
}
