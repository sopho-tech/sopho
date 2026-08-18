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
    #[error("Notebook already holds the maximum of {0} cells")]
    NotebookFull(usize),
    #[error("Failed to generate cell name: {0}")]
    Sopho(#[from] SophoError),
    #[error("Failed to save cell: {0}")]
    Repository(#[from] sea_orm::DbErr),
}

#[derive(Debug, thiserror::Error)]
pub enum GetDatabaseConnectionError {
    #[error("Connection not found")]
    ConnectionNotFound,
    #[error("Failed to get connection: {0}")]
    Connection(sea_orm::DbErr),
    #[error("Failed to connect to database: {0}")]
    DatabaseConnection(#[from] sqlx::Error),
    #[error("Failed to parse source type: {0}")]
    SourceTypeParse(String),
    #[error("Unsupported source type: {0}")]
    UnsupportedSourceType(String),
    #[error("Database pool unavailable: {0}")]
    PoolUnavailable(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ExecuteSqlError {
    #[error("{0}")]
    GetConnection(#[from] GetDatabaseConnectionError),
    #[error("{0}")]
    ExecuteQuery(#[from] ExecuteQueryError),
}

#[derive(Debug, thiserror::Error)]
pub enum GetDataCatalogError {
    #[error("Unsupported source type: {0}")]
    UnsupportedSourceType(String),
    #[error("Failed to parse source type: {0}")]
    SourceTypeParse(String),
    #[error("Failed to fetch data catalog: {0}")]
    DatabaseConnection(#[from] sqlx::Error),
    #[error("{0}")]
    GetConnection(#[from] GetDatabaseConnectionError),
}

#[derive(Debug, thiserror::Error)]
pub enum TextToSqlError {
    #[error("Tables in pruned data catalog missing from functional role analysis: {0}")]
    MissingTablesInFunctionalRoleAnalysis(String),
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateDashboardError {
    #[error("Dashboard can hold at most {0} charts")]
    TooManyCharts(usize),
    #[error("{0}")]
    Repository(#[from] sea_orm::DbErr),
}

#[derive(Debug, thiserror::Error)]
pub enum ExecuteChartError {
    #[error("Cell not found")]
    CellNotFound,
    #[error("Cell has no content")]
    MissingContent,
    #[error("Invalid chart content format")]
    InvalidChartContent,
    #[error("Source cell content is not SQL")]
    SourceNotSql,
    #[error("Source cell has no connection assigned")]
    MissingConnection,
    #[error("Chart has no {0} specified")]
    MissingChartSetting(&'static str),
    #[error("Chart has {count} series, more than the maximum of {max}")]
    TooManySeries { count: usize, max: usize },
    #[error("Chart has two series with the same output name: {0}")]
    DuplicateSeriesAlias(String),
    #[error("Chart references an invalid column name: {0}")]
    InvalidIdentifier(String),
    #[error("{0}")]
    ExecuteSql(#[from] ExecuteSqlError),
    #[error("Database error: {0}")]
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
