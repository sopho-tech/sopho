use crate::common::constants::EnvironmentVariable;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("{message}")]
pub struct ValidationError {
    pub message: String,
}

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
    #[error("Failed to decrypt connection: {0}")]
    Decryption(anyhow::Error),
    #[error("Failed to save connection: {0}")]
    Repository(#[from] sea_orm::DbErr),
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateConnectionError {
    #[error("{0}")]
    GetConnection(#[from] GetDatabaseConnectionError),
    #[error("Failed to get connection status: {0}")]
    ConnectionStatus(#[from] ValidationError),
    #[error("Failed to encrypt connection")]
    Encryption,
    #[error("Failed to decrypt connection: {0}")]
    Decryption(anyhow::Error),
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
    #[error("Failed to decrypt connection: {0}")]
    Decryption(anyhow::Error),
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
    #[error("Failed to fetch data catalog: {0}")]
    DatabaseConnection(#[from] sqlx::Error),
    #[error("{0}")]
    GetConnection(#[from] GetDatabaseConnectionError),
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
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Unsupported cell type for content parsing")]
    UnsupportedCellType,
}

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("missing {variable}")]
    MissingEnvironmentVariable { variable: EnvironmentVariable },
    #[error("invalid {variable}")]
    InvalidEnvironmentVariable {
        variable: EnvironmentVariable,
        #[source]
        source: std::env::VarError,
    },
    #[error("invalid {variable} {value:?}", variable = EnvironmentVariable::Port)]
    InvalidPort {
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },
    #[error("invalid {variable} {value:?}", variable = EnvironmentVariable::SophoCookieSecure)]
    InvalidCookieSecure {
        value: String,
        #[source]
        source: std::str::ParseBoolError,
    },
    #[error("could not read the encryption key from {path}")]
    UnreadableEncryptionKeyFile {
        path: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("could not store a generated encryption key at {path}")]
    UnwritableEncryptionKeyFile {
        path: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    InvalidBaseUrl { source: BaseUrlError },
    #[error("{setting} is required")]
    MissingSetting { setting: EnvironmentVariable },
    #[error("invalid {variable_name}")]
    InvalidOidcVariable {
        variable_name: String,
        #[source]
        source: std::env::VarError,
    },
    #[error("OIDC provider name {name:?} may only contain letters, digits and dashes")]
    InvalidOidcProviderName { name: String },
    #[error("OIDC provider {name:?} is listed twice in {variable}", variable = EnvironmentVariable::SophoOidcProviders)]
    DuplicateOidcProvider { name: String },
    #[error("missing {variable_name} for OIDC provider {provider_name}")]
    MissingOidcProviderVariable {
        variable_name: String,
        provider_name: String,
    },
    #[error("missing {variable}, required when {host} is set", host = EnvironmentVariable::SophoSmtpHost)]
    MissingEmailVariable { variable: EnvironmentVariable },
    #[error("invalid {variable} {value:?}", variable = EnvironmentVariable::SophoSmtpPort)]
    InvalidSmtpPort {
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },
    #[error("invalid {variable} {value:?}: expected tls, starttls or none", variable = EnvironmentVariable::SophoSmtpTls)]
    InvalidSmtpTls { value: String },
    #[error("set both {username} and {password}, or neither", username = EnvironmentVariable::SophoSmtpUsername, password = EnvironmentVariable::SophoSmtpPassword)]
    IncompleteSmtpCredentials,
    #[error("invalid {variable} {value:?}", variable = EnvironmentVariable::SophoEmailFrom)]
    InvalidEmailFrom {
        value: String,
        #[source]
        source: lettre::address::AddressError,
    },
}

impl From<BaseUrlError> for ConfigurationError {
    fn from(source: BaseUrlError) -> Self {
        ConfigurationError::InvalidBaseUrl { source }
    }
}

#[derive(Debug, Error)]
pub enum BaseUrlError {
    #[error("invalid {setting_name} {base_url}")]
    Unparsable {
        setting_name: String,
        base_url: String,
        #[source]
        source: http::uri::InvalidUri,
    },
    #[error("invalid {setting_name} {base_url}: expected an http or https URL")]
    NotHttp {
        setting_name: String,
        base_url: String,
    },
}
