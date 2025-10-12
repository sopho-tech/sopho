use thiserror::Error;

#[derive(Error, Debug)]
pub enum SophoError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    #[error("Notebook not found")]
    NotebookNotFound,
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Unsupported cell type for content parsing")]
    UnsupportedCellType,
}
