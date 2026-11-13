use crate::common::constants::EnvironmentVariable;
use crate::common::error_codes::ErrorCode;
use crate::connection::constants::SourceType;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConnectionGuardError {
    #[error("{} connections are turned off on this deployment", .source_type.display_name())]
    TypeDisabled { source_type: SourceType },
    #[error("This host is not reachable from this deployment")]
    HostNotAllowed,
}

impl ConnectionGuardError {
    pub fn error_code(&self) -> ErrorCode {
        match self {
            ConnectionGuardError::TypeDisabled { .. } => ErrorCode::ConnectionTypeDisabled,
            ConnectionGuardError::HostNotAllowed => ErrorCode::ConnectionHostNotAllowed,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConnectionSettingsError {
    #[error("The connection has no host")]
    MissingHost,
    #[error("The connection has no port")]
    MissingPort,
    #[error("The connection port {port:?} is not a number between 1 and 65535")]
    InvalidPort { port: String },
}

#[derive(Debug, Error)]
pub enum ProbeConnectionError {
    #[error("{0}")]
    Refused(#[from] ConnectionGuardError),
    #[error("{0}")]
    Validation(#[from] ValidationError),
}

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
    #[error("{0}")]
    Refused(ConnectionGuardError),
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
    #[error("{0}")]
    Refused(ConnectionGuardError),
    #[error("Failed to get connection status: {0}")]
    ConnectionStatus(#[from] ValidationError),
    #[error("Failed to encrypt connection")]
    Encryption,
    #[error("Failed to decrypt connection: {0}")]
    Decryption(anyhow::Error),
    #[error("Failed to save connection: {0}")]
    Repository(#[from] sea_orm::DbErr),
}

impl From<ProbeConnectionError> for CreateConnectionError {
    fn from(error: ProbeConnectionError) -> Self {
        match error {
            ProbeConnectionError::Refused(guard_error) => {
                CreateConnectionError::Refused(guard_error)
            }
            ProbeConnectionError::Validation(validation_error) => {
                CreateConnectionError::ConnectionStatus(validation_error)
            }
        }
    }
}

impl From<ProbeConnectionError> for UpdateConnectionError {
    fn from(error: ProbeConnectionError) -> Self {
        match error {
            ProbeConnectionError::Refused(guard_error) => {
                UpdateConnectionError::Refused(guard_error)
            }
            ProbeConnectionError::Validation(validation_error) => {
                UpdateConnectionError::ConnectionStatus(validation_error)
            }
        }
    }
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
    #[error("Database pool unavailable: {0}")]
    PoolUnavailable(String),
    #[error("{0}")]
    Refused(#[from] ConnectionGuardError),
    #[error("{0}")]
    InvalidConnectionSettings(#[from] ConnectionSettingsError),
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
    #[error("invalid {variable_name} {value:?}, expected true or false")]
    InvalidFeatureFlag {
        variable_name: String,
        value: String,
    },
    #[error(
        "invalid {variable} {value:?}, expected a whole number of at least 1",
        variable = EnvironmentVariable::SophoSelfServeTenantLimit
    )]
    InvalidSelfServeTenantLimit { value: String },
    #[error(
        "invalid {variable} entry {value:?}, expected an email address",
        variable = EnvironmentVariable::SophoPlatformAdministratorEmails
    )]
    InvalidPlatformAdministratorEmail { value: String },
    #[error(
        "{variable} lists {value:?} more than once",
        variable = EnvironmentVariable::SophoPlatformAdministratorEmails
    )]
    DuplicatePlatformAdministratorEmail { value: String },
    #[error(
        "invalid {variable} entry {value:?}, expected a connection type name such as SQLITE",
        variable = EnvironmentVariable::SophoDisabledConnectionTypes
    )]
    InvalidDisabledConnectionType { value: String },
    #[error(
        "{variable} lists {value:?} more than once",
        variable = EnvironmentVariable::SophoDisabledConnectionTypes
    )]
    DuplicateDisabledConnectionType { value: String },
    #[error("invalid {variable} {value:?}, expected a whole number of at least 1")]
    InvalidRateLimit {
        variable: EnvironmentVariable,
        value: String,
    },
    #[error("{first_setting} and {second_setting} contradict each other: {reason}")]
    ContradictorySettings {
        first_setting: String,
        second_setting: String,
        reason: &'static str,
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
