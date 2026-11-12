use crate::ai::agent_utils::ModelClient;
use crate::ai_configuration;
use crate::authorization::dto::TenantId;
use crate::common::constants::{
    ADMINISTRATOR_EMAILS_NEED_PROVIDER_REASON, DEFAULT_SELF_SERVE_TENANT_LIMIT,
    DEFAULT_TENANT_DEMO_NEEDS_SQLITE_REASON, DEFAULT_TENANT_NEEDS_ADMINISTRATOR_REASON,
    DISABLED_CONNECTION_TYPE_SEPARATOR, EnvironmentVariable, MINIMUM_SELF_SERVE_TENANT_LIMIT,
    NO_PLATFORM_ADMINISTRATOR_REASON, NOBODY_CAN_SIGN_IN_REASON, OidcProviderVariable,
    PLATFORM_ADMINISTRATOR_EMAIL_SEPARATOR, SEEDED_ADMINISTRATOR_CANNOT_SIGN_IN_REASON,
    SMTP_TIMEOUT_SECONDS, SmtpTls,
};
use crate::common::cryptography_utils::{
    SECRET_KEY_PATH, generate_and_store_encryption_key, get_encryption_key_from_path,
};
use crate::common::errors::ConfigurationError;
use crate::common::url_utils::validated_base_url;
use crate::connection::constants::SourceType;
use crate::database::pool::{DatabasePoolRegistry, new_database_pool_registry};
use crate::db;
use crate::email_delivery;
use crate::email_delivery::service::EmailTransport;
use crate::feature_flag::constants::FeatureFlag;
use crate::feature_flag::dto::FeatureFlags;
use crate::feature_flag::service::feature_flags_from_variables;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env::VarError;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize)]
pub struct OidcProviderConfiguration {
    pub name: String,
    pub display_name: String,
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

pub fn oidc_providers_from_variables(
    read_variable: impl Fn(&str) -> Result<Option<String>, ConfigurationError>,
) -> Result<Vec<OidcProviderConfiguration>, ConfigurationError> {
    let Some(provider_list) = read_variable(EnvironmentVariable::SophoOidcProviders.as_ref())?
    else {
        return Ok(Vec::new());
    };

    let mut providers: Vec<OidcProviderConfiguration> = Vec::new();
    for raw_name in provider_list.split(',') {
        let name = raw_name.trim().to_lowercase();
        if name.is_empty() {
            continue;
        }
        let is_url_safe = name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-');
        if !is_url_safe {
            return Err(ConfigurationError::InvalidOidcProviderName { name });
        }
        if providers.iter().any(|provider| provider.name == name) {
            return Err(ConfigurationError::DuplicateOidcProvider { name });
        }

        let required = |variable: OidcProviderVariable| -> Result<String, ConfigurationError> {
            let variable_name = variable.name_for_provider(&name);
            read_variable(&variable_name)?
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| ConfigurationError::MissingOidcProviderVariable {
                    provider_name: name.clone(),
                    variable_name,
                })
        };
        providers.push(OidcProviderConfiguration {
            display_name: required(OidcProviderVariable::DisplayName)?,
            issuer_url: required(OidcProviderVariable::IssuerUrl)?,
            client_id: required(OidcProviderVariable::ClientId)?,
            client_secret: required(OidcProviderVariable::ClientSecret)?,
            redirect_uri: required(OidcProviderVariable::RedirectUri)?,
            name,
        });
    }
    Ok(providers)
}

#[derive(Clone, Debug, Deserialize)]
pub struct SmtpCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EmailConfiguration {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_tls: SmtpTls,
    pub smtp_credentials: Option<SmtpCredentials>,
    pub smtp_timeout_seconds: u64,
    pub from_address: String,
    pub public_url: String,
}

pub fn email_configuration_from_variables(
    read_variable: impl Fn(EnvironmentVariable) -> Result<Option<String>, ConfigurationError>,
) -> Result<Option<EmailConfiguration>, ConfigurationError> {
    let present_value =
        |variable: EnvironmentVariable| -> Result<Option<String>, ConfigurationError> {
            Ok(read_variable(variable)?.filter(|value| !value.trim().is_empty()))
        };
    let Some(smtp_host) = present_value(EnvironmentVariable::SophoSmtpHost)? else {
        return Ok(None);
    };
    let required_value = |variable: EnvironmentVariable| -> Result<String, ConfigurationError> {
        present_value(variable)?
            .map(|value| value.trim().to_string())
            .ok_or(ConfigurationError::MissingEmailVariable { variable })
    };

    let smtp_port_value = required_value(EnvironmentVariable::SophoSmtpPort)?;
    let smtp_port =
        smtp_port_value
            .parse()
            .map_err(|source| ConfigurationError::InvalidSmtpPort {
                value: smtp_port_value.clone(),
                source,
            })?;
    let smtp_tls_value = required_value(EnvironmentVariable::SophoSmtpTls)?;
    let smtp_tls =
        smtp_tls_value
            .parse::<SmtpTls>()
            .map_err(|_| ConfigurationError::InvalidSmtpTls {
                value: smtp_tls_value.clone(),
            })?;
    let smtp_credentials = match (
        present_value(EnvironmentVariable::SophoSmtpUsername)?,
        present_value(EnvironmentVariable::SophoSmtpPassword)?,
    ) {
        (Some(username), Some(password)) => Some(SmtpCredentials {
            username: username.trim().to_string(),
            password,
        }),
        (None, None) => None,
        _ => return Err(ConfigurationError::IncompleteSmtpCredentials),
    };
    let from_address = required_value(EnvironmentVariable::SophoEmailFrom)?;
    from_address
        .parse::<lettre::message::Mailbox>()
        .map_err(|source| ConfigurationError::InvalidEmailFrom {
            value: from_address.clone(),
            source,
        })?;
    let public_url = validated_base_url(
        EnvironmentVariable::SophoPublicUrl.as_ref(),
        Some(required_value(EnvironmentVariable::SophoPublicUrl)?.into()),
    )?
    .ok_or(ConfigurationError::MissingEmailVariable {
        variable: EnvironmentVariable::SophoPublicUrl,
    })?
    .trim_end_matches('/')
    .to_string();

    Ok(Some(EmailConfiguration {
        smtp_host: smtp_host.trim().to_string(),
        smtp_port,
        smtp_tls,
        smtp_credentials,
        smtp_timeout_seconds: SMTP_TIMEOUT_SECONDS,
        from_address,
        public_url,
    }))
}

pub fn self_serve_tenant_limit_from_variable(
    value: Option<String>,
) -> Result<u64, ConfigurationError> {
    let Some(value) = value else {
        return Ok(DEFAULT_SELF_SERVE_TENANT_LIMIT);
    };
    match value.parse::<u64>() {
        Ok(limit) if limit >= MINIMUM_SELF_SERVE_TENANT_LIMIT => Ok(limit),
        _ => Err(ConfigurationError::InvalidSelfServeTenantLimit { value }),
    }
}

pub fn platform_administrator_emails_from_variable(
    value: Option<String>,
) -> Result<Vec<String>, ConfigurationError> {
    let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
        return Ok(Vec::new());
    };
    let mut platform_administrator_emails: Vec<String> = Vec::new();
    for entry in value.split(PLATFORM_ADMINISTRATOR_EMAIL_SEPARATOR) {
        let email = crate::authentication::service::normalize_email(entry);
        if email.parse::<lettre::Address>().is_err() {
            return Err(ConfigurationError::InvalidPlatformAdministratorEmail {
                value: entry.trim().to_string(),
            });
        }
        if platform_administrator_emails.contains(&email) {
            return Err(ConfigurationError::DuplicatePlatformAdministratorEmail { value: email });
        }
        platform_administrator_emails.push(email);
    }
    Ok(platform_administrator_emails)
}

pub fn disabled_connection_types_from_variable(
    value: Option<String>,
) -> Result<Vec<SourceType>, ConfigurationError> {
    let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
        return Ok(Vec::new());
    };
    let mut disabled_connection_types: Vec<SourceType> = Vec::new();
    for entry in value.split(DISABLED_CONNECTION_TYPE_SEPARATOR) {
        let source_type_name = entry.trim();
        let source_type = SourceType::from_str(source_type_name).map_err(|_| {
            ConfigurationError::InvalidDisabledConnectionType {
                value: source_type_name.to_string(),
            }
        })?;
        if disabled_connection_types.contains(&source_type) {
            return Err(ConfigurationError::DuplicateDisabledConnectionType {
                value: source_type_name.to_string(),
            });
        }
        disabled_connection_types.push(source_type);
    }
    Ok(disabled_connection_types)
}

pub fn validate_setting_combinations(
    configurations: &Configurations,
) -> Result<(), ConfigurationError> {
    let is_password_sign_in_on = configurations
        .feature_flags
        .is_enabled(FeatureFlag::PasswordSignIn);
    let is_administrator_seeded = configurations
        .feature_flags
        .is_enabled(FeatureFlag::SeedPasswordAdministrator);
    let is_default_tenant_on = configurations
        .feature_flags
        .is_enabled(FeatureFlag::DefaultTenant);
    let has_oidc_provider = !configurations.oidc_providers.is_empty();
    let has_administrator_emails = !configurations.platform_administrator_emails.is_empty();
    let is_sqlite_disabled = configurations
        .disabled_connection_types
        .contains(&SourceType::Sqlite);
    let contradiction = |first_setting: String, second_setting: String, reason: &'static str| {
        Err(ConfigurationError::ContradictorySettings {
            first_setting,
            second_setting,
            reason,
        })
    };

    if !is_password_sign_in_on && !has_oidc_provider {
        return contradiction(
            FeatureFlag::PasswordSignIn.environment_variable_name(),
            EnvironmentVariable::SophoOidcProviders.to_string(),
            NOBODY_CAN_SIGN_IN_REASON,
        );
    }
    if !is_password_sign_in_on && is_administrator_seeded {
        return contradiction(
            FeatureFlag::PasswordSignIn.environment_variable_name(),
            FeatureFlag::SeedPasswordAdministrator.environment_variable_name(),
            SEEDED_ADMINISTRATOR_CANNOT_SIGN_IN_REASON,
        );
    }
    if is_default_tenant_on && !is_administrator_seeded {
        return contradiction(
            FeatureFlag::DefaultTenant.environment_variable_name(),
            FeatureFlag::SeedPasswordAdministrator.environment_variable_name(),
            DEFAULT_TENANT_NEEDS_ADMINISTRATOR_REASON,
        );
    }
    if !is_administrator_seeded && !has_administrator_emails {
        return contradiction(
            FeatureFlag::SeedPasswordAdministrator.environment_variable_name(),
            EnvironmentVariable::SophoPlatformAdministratorEmails.to_string(),
            NO_PLATFORM_ADMINISTRATOR_REASON,
        );
    }
    if has_administrator_emails && !has_oidc_provider {
        return contradiction(
            EnvironmentVariable::SophoPlatformAdministratorEmails.to_string(),
            EnvironmentVariable::SophoOidcProviders.to_string(),
            ADMINISTRATOR_EMAILS_NEED_PROVIDER_REASON,
        );
    }
    if is_sqlite_disabled && is_default_tenant_on {
        return contradiction(
            EnvironmentVariable::SophoDisabledConnectionTypes.to_string(),
            FeatureFlag::DefaultTenant.environment_variable_name(),
            DEFAULT_TENANT_DEMO_NEEDS_SQLITE_REASON,
        );
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize)]
pub struct PasswordAdministratorConfiguration {
    pub username: String,
    pub password: String,
    pub email: String,
    pub full_name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Configurations {
    pub database_url: Cow<'static, str>,
    pub port: u16,
    pub frontend_dir: Cow<'static, str>,
    pub environment: Cow<'static, str>,
    pub cookie_secure: bool,
    pub encryption_key: Cow<'static, str>,
    pub password_administrator: Option<PasswordAdministratorConfiguration>,
    pub anthropic_base_url: Option<Cow<'static, str>>,
    pub openai_base_url: Option<Cow<'static, str>>,
    pub oidc_providers: Vec<OidcProviderConfiguration>,
    pub email: Option<EmailConfiguration>,
    pub feature_flags: FeatureFlags,
    pub self_serve_tenant_limit: u64,
    pub platform_administrator_emails: Vec<String>,
    pub disabled_connection_types: Vec<SourceType>,
}

pub struct ConfigurationsBuilder {
    database_url: Option<Cow<'static, str>>,
    port: Option<u16>,
    frontend_dir: Option<Cow<'static, str>>,
    environment: Option<Cow<'static, str>>,
    cookie_secure: Option<bool>,
    encryption_key: Option<Cow<'static, str>>,
    admin_username: Option<Cow<'static, str>>,
    admin_password: Option<Cow<'static, str>>,
    admin_email: Option<Cow<'static, str>>,
    admin_full_name: Option<Cow<'static, str>>,
    anthropic_base_url: Option<Cow<'static, str>>,
    openai_base_url: Option<Cow<'static, str>>,
    oidc_providers: Vec<OidcProviderConfiguration>,
    email: Option<EmailConfiguration>,
    feature_flags: FeatureFlags,
    self_serve_tenant_limit: Option<u64>,
    platform_administrator_emails: Option<String>,
    disabled_connection_types: Option<String>,
}

impl ConfigurationsBuilder {
    pub fn new() -> Self {
        Self {
            database_url: None,
            port: None,
            frontend_dir: None,
            environment: None,
            cookie_secure: None,
            encryption_key: None,
            admin_username: None,
            admin_password: None,
            admin_email: None,
            admin_full_name: None,
            anthropic_base_url: None,
            openai_base_url: None,
            oidc_providers: Vec::new(),
            email: None,
            feature_flags: FeatureFlags::from_defaults(),
            self_serve_tenant_limit: None,
            platform_administrator_emails: None,
            disabled_connection_types: None,
        }
    }

    pub fn database_url(mut self, database_url: impl Into<Cow<'static, str>>) -> Self {
        self.database_url = Some(database_url.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn frontend_dir(mut self, frontend_dir: impl Into<Cow<'static, str>>) -> Self {
        self.frontend_dir = Some(frontend_dir.into());
        self
    }

    pub fn environment(mut self, environment: impl Into<Cow<'static, str>>) -> Self {
        self.environment = Some(environment.into());
        self
    }

    pub fn cookie_secure(mut self, cookie_secure: bool) -> Self {
        self.cookie_secure = Some(cookie_secure);
        self
    }

    pub fn encryption_key(mut self, encryption_key: impl Into<Cow<'static, str>>) -> Self {
        self.encryption_key = Some(encryption_key.into());
        self
    }

    pub fn admin_username(mut self, admin_username: impl Into<Cow<'static, str>>) -> Self {
        self.admin_username = Some(admin_username.into());
        self
    }

    pub fn admin_password(mut self, admin_password: impl Into<Cow<'static, str>>) -> Self {
        self.admin_password = Some(admin_password.into());
        self
    }

    pub fn admin_email(mut self, admin_email: impl Into<Cow<'static, str>>) -> Self {
        self.admin_email = Some(admin_email.into());
        self
    }

    pub fn admin_full_name(mut self, admin_full_name: impl Into<Cow<'static, str>>) -> Self {
        self.admin_full_name = Some(admin_full_name.into());
        self
    }

    pub fn anthropic_base_url(mut self, anthropic_base_url: impl Into<Cow<'static, str>>) -> Self {
        self.anthropic_base_url = Some(anthropic_base_url.into());
        self
    }

    pub fn openai_base_url(mut self, openai_base_url: impl Into<Cow<'static, str>>) -> Self {
        self.openai_base_url = Some(openai_base_url.into());
        self
    }

    pub fn oidc_providers(mut self, providers: Vec<OidcProviderConfiguration>) -> Self {
        self.oidc_providers = providers;
        self
    }

    pub fn email(mut self, email: EmailConfiguration) -> Self {
        self.email = Some(email);
        self
    }

    pub fn feature_flag(mut self, feature_flag: FeatureFlag, is_enabled: bool) -> Self {
        self.feature_flags.set_enabled(feature_flag, is_enabled);
        self
    }

    pub fn self_serve_tenant_limit(mut self, self_serve_tenant_limit: u64) -> Self {
        self.self_serve_tenant_limit = Some(self_serve_tenant_limit);
        self
    }

    pub fn platform_administrator_emails(
        mut self,
        platform_administrator_emails: impl Into<String>,
    ) -> Self {
        self.platform_administrator_emails = Some(platform_administrator_emails.into());
        self
    }

    pub fn disabled_connection_types(
        mut self,
        disabled_connection_types: impl Into<String>,
    ) -> Self {
        self.disabled_connection_types = Some(disabled_connection_types.into());
        self
    }

    pub fn build(self) -> Result<Configurations, ConfigurationError> {
        let password_administrator = match self
            .feature_flags
            .is_enabled(FeatureFlag::SeedPasswordAdministrator)
        {
            true => Some(PasswordAdministratorConfiguration {
                username: required_setting(
                    self.admin_username,
                    EnvironmentVariable::SophoAdminUsername,
                )?
                .into_owned(),
                password: required_setting(
                    self.admin_password,
                    EnvironmentVariable::SophoAdminPassword,
                )?
                .into_owned(),
                email: required_setting(self.admin_email, EnvironmentVariable::SophoAdminEmail)?
                    .into_owned(),
                full_name: required_setting(
                    self.admin_full_name,
                    EnvironmentVariable::SophoAdminFullName,
                )?
                .into_owned(),
            }),
            false => None,
        };
        let configurations = Configurations {
            database_url: self
                .database_url
                .unwrap_or_else(|| db::get_sqlite_database_file_path().into()),
            port: self.port.unwrap_or(8000),
            frontend_dir: required_setting(
                self.frontend_dir,
                EnvironmentVariable::SophoFrontendDir,
            )?,
            environment: self.environment.unwrap_or(Cow::Borrowed("production")),
            cookie_secure: self.cookie_secure.unwrap_or(false),
            encryption_key: required_setting(
                self.encryption_key,
                EnvironmentVariable::SophoEncryptionKey,
            )?,
            password_administrator,
            anthropic_base_url: validated_base_url(
                EnvironmentVariable::SophoAnthropicBaseUrl.as_ref(),
                self.anthropic_base_url,
            )?,
            openai_base_url: validated_base_url(
                EnvironmentVariable::SophoOpenaiBaseUrl.as_ref(),
                self.openai_base_url,
            )?,
            oidc_providers: self.oidc_providers,
            email: self.email,
            feature_flags: self.feature_flags,
            self_serve_tenant_limit: self_serve_tenant_limit_from_variable(
                self.self_serve_tenant_limit
                    .map(|self_serve_tenant_limit| self_serve_tenant_limit.to_string()),
            )?,
            platform_administrator_emails: platform_administrator_emails_from_variable(
                self.platform_administrator_emails,
            )?,
            disabled_connection_types: disabled_connection_types_from_variable(
                self.disabled_connection_types,
            )?,
        };
        validate_setting_combinations(&configurations)?;
        Ok(configurations)
    }
}

impl Default for ConfigurationsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Configurations {
    pub fn builder() -> ConfigurationsBuilder {
        ConfigurationsBuilder::new()
    }

    pub fn from_env() -> Result<Self, ConfigurationError> {
        dotenvy::dotenv().ok();

        let feature_flags = feature_flags_from_variables(feature_flag_environment_variable)?;
        let password_administrator = match feature_flags
            .is_enabled(FeatureFlag::SeedPasswordAdministrator)
        {
            true => Some(PasswordAdministratorConfiguration {
                username: required_environment_variable(EnvironmentVariable::SophoAdminUsername)?
                    .into_owned(),
                password: required_environment_variable(EnvironmentVariable::SophoAdminPassword)?
                    .into_owned(),
                email: required_environment_variable(EnvironmentVariable::SophoAdminEmail)?
                    .into_owned(),
                full_name: required_environment_variable(EnvironmentVariable::SophoAdminFullName)?
                    .into_owned(),
            }),
            false => None,
        };
        let configurations = Self {
            database_url: match optional_environment_variable(EnvironmentVariable::DatabaseUrl)? {
                Some(database_url) => database_url,
                None => {
                    warn!(
                        "Using SQLite as the database. This is not recommended for production use."
                    );
                    Cow::Owned(db::get_sqlite_database_file_path())
                }
            },
            port: match optional_environment_variable(EnvironmentVariable::Port)? {
                Some(port) => port
                    .parse()
                    .map_err(|source| ConfigurationError::InvalidPort {
                        value: port.to_string(),
                        source,
                    })?,
                None => 8000,
            },
            frontend_dir: required_environment_variable(EnvironmentVariable::SophoFrontendDir)?,
            environment: optional_environment_variable(EnvironmentVariable::SophoEnvironment)?
                .unwrap_or(Cow::Borrowed("production")),
            cookie_secure: match optional_environment_variable(
                EnvironmentVariable::SophoCookieSecure,
            )? {
                Some(cookie_secure) => cookie_secure.parse().map_err(|source| {
                    ConfigurationError::InvalidCookieSecure {
                        value: cookie_secure.to_string(),
                        source,
                    }
                })?,
                None => false,
            },
            encryption_key: match optional_environment_variable(
                EnvironmentVariable::SophoEncryptionKey,
            )? {
                Some(encryption_key) => encryption_key,
                None => stored_or_generated_encryption_key()?,
            },
            password_administrator,
            anthropic_base_url: validated_base_url(
                EnvironmentVariable::SophoAnthropicBaseUrl.as_ref(),
                optional_environment_variable(EnvironmentVariable::SophoAnthropicBaseUrl)?,
            )?,
            openai_base_url: validated_base_url(
                EnvironmentVariable::SophoOpenaiBaseUrl.as_ref(),
                optional_environment_variable(EnvironmentVariable::SophoOpenaiBaseUrl)?,
            )?,
            oidc_providers: oidc_providers_from_variables(oidc_environment_variable)?,
            email: email_configuration_from_variables(|variable| {
                Ok(optional_environment_variable(variable)?.map(Cow::into_owned))
            })?,
            feature_flags,
            self_serve_tenant_limit: self_serve_tenant_limit_from_variable(
                optional_environment_variable(EnvironmentVariable::SophoSelfServeTenantLimit)?
                    .map(Cow::into_owned),
            )?,
            platform_administrator_emails: platform_administrator_emails_from_variable(
                optional_environment_variable(
                    EnvironmentVariable::SophoPlatformAdministratorEmails,
                )?
                .map(Cow::into_owned),
            )?,
            disabled_connection_types: disabled_connection_types_from_variable(
                optional_environment_variable(EnvironmentVariable::SophoDisabledConnectionTypes)?
                    .map(Cow::into_owned),
            )?,
        };
        validate_setting_combinations(&configurations)?;
        Ok(configurations)
    }
}

fn required_setting(
    value: Option<Cow<'static, str>>,
    setting: EnvironmentVariable,
) -> Result<Cow<'static, str>, ConfigurationError> {
    value.ok_or(ConfigurationError::MissingSetting { setting })
}

fn stored_or_generated_encryption_key() -> Result<Cow<'static, str>, ConfigurationError> {
    let stored_key = get_encryption_key_from_path().map_err(|source| {
        ConfigurationError::UnreadableEncryptionKeyFile {
            path: SECRET_KEY_PATH,
            source,
        }
    })?;
    if let Some(stored_key) = stored_key {
        return Ok(stored_key.into());
    }
    let (encryption_key, full_path) = generate_and_store_encryption_key().map_err(|source| {
        ConfigurationError::UnwritableEncryptionKeyFile {
            path: SECRET_KEY_PATH,
            source,
        }
    })?;
    warn!(
        "Since {} is not provided, one is generated and stored at {}. This is not recommended for production use.",
        EnvironmentVariable::SophoEncryptionKey,
        full_path
    );
    Ok(encryption_key.into())
}

fn oidc_environment_variable(variable_name: &str) -> Result<Option<String>, ConfigurationError> {
    match std::env::var(variable_name) {
        Ok(value) => Ok(Some(value)),
        Err(VarError::NotPresent) => Ok(None),
        Err(source) => Err(ConfigurationError::InvalidOidcVariable {
            variable_name: variable_name.to_string(),
            source,
        }),
    }
}

fn feature_flag_environment_variable(
    variable_name: &str,
) -> Result<Option<String>, ConfigurationError> {
    match std::env::var(variable_name) {
        Ok(value) => Ok(Some(value)),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(value)) => Err(ConfigurationError::InvalidFeatureFlag {
            variable_name: variable_name.to_string(),
            value: value.to_string_lossy().into_owned(),
        }),
    }
}

fn environment_variable(variable: EnvironmentVariable) -> Result<String, VarError> {
    std::env::var(variable.as_ref())
}

fn optional_environment_variable(
    variable: EnvironmentVariable,
) -> Result<Option<Cow<'static, str>>, ConfigurationError> {
    match environment_variable(variable) {
        Ok(value) => Ok(Some(value.into())),
        Err(VarError::NotPresent) => Ok(None),
        Err(source) => Err(ConfigurationError::InvalidEnvironmentVariable { variable, source }),
    }
}

fn required_environment_variable(
    variable: EnvironmentVariable,
) -> Result<Cow<'static, str>, ConfigurationError> {
    optional_environment_variable(variable)?
        .ok_or(ConfigurationError::MissingEnvironmentVariable { variable })
}

#[derive(Clone)]
pub struct AppState {
    pub database_connection: DatabaseConnection,
    pub config: Configurations,
    pub model_client: Arc<RwLock<HashMap<Uuid, ModelClient>>>,
    pub database_pools: DatabasePoolRegistry,
    pub email_transport: Option<EmailTransport>,
}

impl AppState {
    pub async fn from_configurations(config: Configurations) -> anyhow::Result<Self> {
        let database_connection = db::get_db(&config.database_url).await?;
        let email_transport = config
            .email
            .as_ref()
            .map(email_delivery::service::build_email_transport)
            .transpose()?;
        Ok(Self {
            database_connection,
            config,
            model_client: Arc::new(RwLock::new(HashMap::new())),
            database_pools: new_database_pool_registry(),
            email_transport,
        })
    }

    pub async fn current_model_client(&self, tenant_id: &TenantId) -> Option<ModelClient> {
        self.model_client
            .read()
            .await
            .get(&tenant_id.get_uuid())
            .cloned()
    }

    pub async fn set_model_client(&self, tenant_id: &TenantId, model_client: ModelClient) {
        self.model_client
            .write()
            .await
            .insert(tenant_id.get_uuid(), model_client);
    }

    pub async fn clear_model_client(&self, tenant_id: &TenantId) {
        self.model_client
            .write()
            .await
            .remove(&tenant_id.get_uuid());
    }

    pub async fn require_model_client(&self, tenant_id: &TenantId) -> anyhow::Result<ModelClient> {
        self.current_model_client(tenant_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("AI is not configured for this tenant"))
    }

    pub async fn from_env() -> anyhow::Result<Self> {
        Self::from_configurations(Configurations::from_env()?).await
    }

    pub async fn load_ai_configuration(&self) {
        if let Err(e) = ai_configuration::service::load_at_startup(self).await {
            warn!("failed to load ai_configuration at startup: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Configurations, ConfigurationsBuilder, OidcProviderConfiguration,
        disabled_connection_types_from_variable, email_configuration_from_variables,
        oidc_providers_from_variables, platform_administrator_emails_from_variable,
        self_serve_tenant_limit_from_variable,
    };
    use crate::common::constants::{EnvironmentVariable, SmtpTls};
    use crate::common::errors::ConfigurationError;
    use crate::connection::constants::SourceType;
    use crate::feature_flag::constants::FeatureFlag;
    use std::collections::HashMap;

    fn variable_reader(
        variables: HashMap<&'static str, &'static str>,
    ) -> impl Fn(&str) -> Result<Option<String>, ConfigurationError> {
        move |variable: &str| Ok(variables.get(variable).map(|value| value.to_string()))
    }

    fn email_variable_reader(
        variables: HashMap<&'static str, &'static str>,
    ) -> impl Fn(EnvironmentVariable) -> Result<Option<String>, ConfigurationError> {
        move |variable: EnvironmentVariable| {
            Ok(variables
                .get(variable.as_ref())
                .map(|value| value.to_string()))
        }
    }

    fn complete_email_variables() -> HashMap<&'static str, &'static str> {
        [
            ("SOPHO_SMTP_HOST", " smtp.example.com "),
            ("SOPHO_SMTP_PORT", "587"),
            ("SOPHO_SMTP_TLS", "starttls"),
            ("SOPHO_SMTP_USERNAME", "mailer"),
            ("SOPHO_SMTP_PASSWORD", " secret with spaces "),
            ("SOPHO_EMAIL_FROM", "Sopho <no-reply@example.com>"),
            ("SOPHO_PUBLIC_URL", "https://sopho.example.com/"),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn the_self_serve_tenant_limit_defaults_to_two_and_can_be_overridden() {
        assert_eq!(self_serve_tenant_limit_from_variable(None).unwrap(), 2);
        assert_eq!(
            self_serve_tenant_limit_from_variable(Some("5".to_string())).unwrap(),
            5
        );
    }

    #[test]
    fn a_self_serve_tenant_limit_below_one_or_not_a_number_is_refused() {
        for refused_value in ["0", "-1", "two", "", "1.5"] {
            let error =
                self_serve_tenant_limit_from_variable(Some(refused_value.to_string())).unwrap_err();
            assert!(
                matches!(
                    &error,
                    ConfigurationError::InvalidSelfServeTenantLimit { value }
                        if value == refused_value
                ),
                "{refused_value:?} was not refused"
            );
            assert!(error.to_string().contains("SOPHO_SELF_SERVE_TENANT_LIMIT"));
        }
    }

    fn configurations_builder() -> ConfigurationsBuilder {
        Configurations::builder()
            .frontend_dir("frontend")
            .encryption_key("encryption-key")
    }

    fn with_password_administrator(builder: ConfigurationsBuilder) -> ConfigurationsBuilder {
        builder
            .admin_username("admin")
            .admin_password("password")
            .admin_email("admin@example.test")
            .admin_full_name("The Administrator")
    }

    fn with_oidc_provider(builder: ConfigurationsBuilder) -> ConfigurationsBuilder {
        builder.oidc_providers(vec![OidcProviderConfiguration {
            name: "google".to_string(),
            display_name: "Google".to_string(),
            issuer_url: "https://accounts.google.com".to_string(),
            client_id: "client-id".to_string(),
            client_secret: "client-secret".to_string(),
            redirect_uri: "https://app.example/api/v1/auth/oidc/google/callback".to_string(),
        }])
    }

    fn passwordless(builder: ConfigurationsBuilder) -> ConfigurationsBuilder {
        builder
            .feature_flag(FeatureFlag::PasswordSignIn, false)
            .feature_flag(FeatureFlag::SeedPasswordAdministrator, false)
            .feature_flag(FeatureFlag::DefaultTenant, false)
    }

    fn assert_contradiction(
        builder: ConfigurationsBuilder,
        first_expected_setting: &str,
        second_expected_setting: &str,
    ) {
        let error = builder.build().unwrap_err();
        assert!(
            matches!(
                &error,
                ConfigurationError::ContradictorySettings { first_setting, second_setting, .. }
                    if first_setting == first_expected_setting
                        && second_setting == second_expected_setting
            ),
            "unexpected error: {error}"
        );
        let message = error.to_string();
        assert!(
            message.contains(first_expected_setting) && message.contains(second_expected_setting),
            "{message}"
        );
    }

    #[test]
    fn every_setting_unset_is_accepted() {
        let configurations = with_password_administrator(configurations_builder())
            .build()
            .unwrap();
        assert!(configurations.platform_administrator_emails.is_empty());
        assert!(configurations.password_administrator.is_some());
    }

    #[test]
    fn the_passwordless_combination_is_accepted_without_administrator_settings() {
        let configurations = passwordless(with_oidc_provider(configurations_builder()))
            .platform_administrator_emails("founder@example.test")
            .build()
            .unwrap();
        assert!(configurations.password_administrator.is_none());
        assert_eq!(
            configurations.platform_administrator_emails,
            vec!["founder@example.test".to_string()]
        );
    }

    #[test]
    fn seeding_the_administrator_requires_its_four_settings() {
        let error = configurations_builder().build().unwrap_err();
        assert!(matches!(
            error,
            ConfigurationError::MissingSetting {
                setting: EnvironmentVariable::SophoAdminUsername
            }
        ));
    }

    #[test]
    fn password_sign_in_off_needs_an_oidc_provider() {
        assert_contradiction(
            passwordless(configurations_builder())
                .platform_administrator_emails("founder@example.test"),
            "SOPHO_FEATURE_PASSWORD_SIGN_IN",
            "SOPHO_OIDC_PROVIDERS",
        );
    }

    #[test]
    fn password_sign_in_off_refuses_a_seeded_password_administrator() {
        assert_contradiction(
            with_password_administrator(with_oidc_provider(configurations_builder()))
                .feature_flag(FeatureFlag::PasswordSignIn, false),
            "SOPHO_FEATURE_PASSWORD_SIGN_IN",
            "SOPHO_FEATURE_SEED_PASSWORD_ADMINISTRATOR",
        );
    }

    #[test]
    fn the_default_tenant_needs_the_seeded_administrator() {
        assert_contradiction(
            with_oidc_provider(configurations_builder())
                .feature_flag(FeatureFlag::SeedPasswordAdministrator, false)
                .platform_administrator_emails("founder@example.test"),
            "SOPHO_FEATURE_DEFAULT_TENANT",
            "SOPHO_FEATURE_SEED_PASSWORD_ADMINISTRATOR",
        );
    }

    #[test]
    fn no_seeded_administrator_needs_a_platform_administrator_email() {
        assert_contradiction(
            passwordless(with_oidc_provider(configurations_builder())),
            "SOPHO_FEATURE_SEED_PASSWORD_ADMINISTRATOR",
            "SOPHO_PLATFORM_ADMINISTRATOR_EMAILS",
        );
    }

    #[test]
    fn platform_administrator_emails_need_an_oidc_provider() {
        assert_contradiction(
            with_password_administrator(configurations_builder())
                .platform_administrator_emails("founder@example.test"),
            "SOPHO_PLATFORM_ADMINISTRATOR_EMAILS",
            "SOPHO_OIDC_PROVIDERS",
        );
    }

    #[test]
    fn disabling_sqlite_is_refused_while_the_default_tenant_is_on() {
        assert_contradiction(
            with_password_administrator(configurations_builder())
                .disabled_connection_types("SQLITE"),
            "SOPHO_DISABLED_CONNECTION_TYPES",
            "SOPHO_FEATURE_DEFAULT_TENANT",
        );
    }

    #[test]
    fn disabling_sqlite_is_accepted_without_the_default_tenant() {
        let configurations = passwordless(with_oidc_provider(configurations_builder()))
            .platform_administrator_emails("founder@example.test")
            .disabled_connection_types("SQLITE")
            .build()
            .unwrap();
        assert_eq!(
            configurations.disabled_connection_types,
            vec![SourceType::Sqlite]
        );
    }

    #[test]
    fn disabling_another_connection_type_is_accepted_with_the_default_tenant() {
        let configurations = with_password_administrator(configurations_builder())
            .disabled_connection_types("SUPABASE")
            .build()
            .unwrap();
        assert_eq!(
            configurations.disabled_connection_types,
            vec![SourceType::Supabase]
        );
    }

    #[test]
    fn disabled_connection_types_are_trimmed_and_default_to_none() {
        assert!(
            disabled_connection_types_from_variable(None)
                .unwrap()
                .is_empty()
        );
        assert!(
            disabled_connection_types_from_variable(Some("  ".to_string()))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            disabled_connection_types_from_variable(Some(" SQLITE , SUPABASE".to_string()))
                .unwrap(),
            vec![SourceType::Sqlite, SourceType::Supabase]
        );
    }

    #[test]
    fn an_unknown_or_repeated_disabled_connection_type_is_refused() {
        for unknown_entry_list in ["sqlite", "SQLITE,", "SQLITE,,SUPABASE", "DUCKDB"] {
            assert!(
                matches!(
                    disabled_connection_types_from_variable(Some(unknown_entry_list.into()))
                        .unwrap_err(),
                    ConfigurationError::InvalidDisabledConnectionType { .. }
                ),
                "{unknown_entry_list:?} was not refused"
            );
        }
        let repeated =
            disabled_connection_types_from_variable(Some("SQLITE, SQLITE".into())).unwrap_err();
        assert!(matches!(
            repeated,
            ConfigurationError::DuplicateDisabledConnectionType { ref value } if value == "SQLITE"
        ));
        assert!(
            repeated
                .to_string()
                .contains("SOPHO_DISABLED_CONNECTION_TYPES")
        );
    }

    #[test]
    fn platform_administrator_emails_are_trimmed_and_normalised() {
        assert!(
            platform_administrator_emails_from_variable(None)
                .unwrap()
                .is_empty()
        );
        assert!(
            platform_administrator_emails_from_variable(Some("  ".to_string()))
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            platform_administrator_emails_from_variable(Some(
                " Founder@Example.test , second@example.test".to_string()
            ))
            .unwrap(),
            vec![
                "founder@example.test".to_string(),
                "second@example.test".to_string()
            ]
        );
    }

    #[test]
    fn an_invalid_or_repeated_platform_administrator_email_is_refused() {
        let invalid =
            platform_administrator_emails_from_variable(Some("founder@example.test,nobody".into()))
                .unwrap_err();
        assert!(
            matches!(
                &invalid,
                ConfigurationError::InvalidPlatformAdministratorEmail { value } if value == "nobody"
            ),
            "{invalid}"
        );
        assert!(
            invalid
                .to_string()
                .contains("SOPHO_PLATFORM_ADMINISTRATOR_EMAILS")
        );

        for blank_entry_list in [
            "founder@example.test,",
            " , ",
            "founder@example.test,,x@y.test",
        ] {
            assert!(
                matches!(
                    platform_administrator_emails_from_variable(Some(blank_entry_list.into()))
                        .unwrap_err(),
                    ConfigurationError::InvalidPlatformAdministratorEmail { value }
                        if value.is_empty()
                ),
                "{blank_entry_list:?} was not refused"
            );
        }

        let repeated = platform_administrator_emails_from_variable(Some(
            "founder@example.test, FOUNDER@example.test".into(),
        ))
        .unwrap_err();
        assert!(
            matches!(
                &repeated,
                ConfigurationError::DuplicatePlatformAdministratorEmail { value }
                    if value == "founder@example.test"
            ),
            "{repeated}"
        );
    }

    #[test]
    fn email_is_off_without_a_host() {
        assert!(
            email_configuration_from_variables(|_| Ok(None))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn a_host_without_a_port_names_the_missing_variable() {
        let mut variables = complete_email_variables();
        variables.remove("SOPHO_SMTP_PORT");
        let error =
            email_configuration_from_variables(email_variable_reader(variables)).unwrap_err();
        assert!(matches!(
            error,
            ConfigurationError::MissingEmailVariable {
                variable: EnvironmentVariable::SophoSmtpPort
            }
        ));
        assert_eq!(
            error.to_string(),
            "missing SOPHO_SMTP_PORT, required when SOPHO_SMTP_HOST is set"
        );
    }

    #[test]
    fn credentials_must_come_in_pairs() {
        let mut variables = complete_email_variables();
        variables.remove("SOPHO_SMTP_PASSWORD");
        assert!(matches!(
            email_configuration_from_variables(email_variable_reader(variables)).unwrap_err(),
            ConfigurationError::IncompleteSmtpCredentials
        ));
    }

    #[test]
    fn an_unknown_tls_mode_is_refused() {
        let mut variables = complete_email_variables();
        variables.insert("SOPHO_SMTP_TLS", "ssl");
        assert!(matches!(
            email_configuration_from_variables(email_variable_reader(variables)).unwrap_err(),
            ConfigurationError::InvalidSmtpTls { value } if value == "ssl"
        ));
    }

    #[test]
    fn an_invalid_sender_or_public_url_is_refused() {
        let mut bad_sender = complete_email_variables();
        bad_sender.insert("SOPHO_EMAIL_FROM", "not an address");
        assert!(matches!(
            email_configuration_from_variables(email_variable_reader(bad_sender)).unwrap_err(),
            ConfigurationError::InvalidEmailFrom { .. }
        ));

        let mut bad_public_url = complete_email_variables();
        bad_public_url.insert("SOPHO_PUBLIC_URL", "sopho.example.com");
        assert!(matches!(
            email_configuration_from_variables(email_variable_reader(bad_public_url)).unwrap_err(),
            ConfigurationError::InvalidBaseUrl { .. }
        ));
    }

    #[test]
    fn a_complete_set_builds_the_configuration() {
        let email =
            email_configuration_from_variables(email_variable_reader(complete_email_variables()))
                .unwrap()
                .expect("email is configured");
        assert_eq!(email.smtp_host, "smtp.example.com");
        assert_eq!(email.smtp_port, 587);
        assert_eq!(email.smtp_tls, SmtpTls::Starttls);
        let credentials = email.smtp_credentials.expect("credentials are set");
        assert_eq!(credentials.username, "mailer");
        assert_eq!(credentials.password, " secret with spaces ");
        assert_eq!(email.from_address, "Sopho <no-reply@example.com>");
        assert_eq!(email.public_url, "https://sopho.example.com");
    }

    #[test]
    fn configured_providers_require_every_variable() {
        let complete: HashMap<&str, &str> = [
            ("SOPHO_OIDC_PROVIDERS", " Google "),
            ("SOPHO_OIDC_GOOGLE_DISPLAY_NAME", "Google"),
            (
                "SOPHO_OIDC_GOOGLE_ISSUER_URL",
                "https://accounts.google.com",
            ),
            ("SOPHO_OIDC_GOOGLE_CLIENT_ID", "client-id"),
            ("SOPHO_OIDC_GOOGLE_CLIENT_SECRET", "client-secret"),
            (
                "SOPHO_OIDC_GOOGLE_REDIRECT_URI",
                "https://app.example/api/v1/auth/oidc/google/callback",
            ),
        ]
        .into_iter()
        .collect();

        let providers = oidc_providers_from_variables(variable_reader(complete.clone())).unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].name, "google");
        assert_eq!(providers[0].client_secret, "client-secret");

        let mut missing_secret = complete.clone();
        missing_secret.remove("SOPHO_OIDC_GOOGLE_CLIENT_SECRET");
        let error = oidc_providers_from_variables(variable_reader(missing_secret)).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("SOPHO_OIDC_GOOGLE_CLIENT_SECRET")
        );

        let mut unsafe_name = complete.clone();
        unsafe_name.insert("SOPHO_OIDC_PROVIDERS", "goo gle");
        assert!(oidc_providers_from_variables(variable_reader(unsafe_name)).is_err());

        assert!(
            oidc_providers_from_variables(|_| Ok(None))
                .unwrap()
                .is_empty(),
            "without SOPHO_OIDC_PROVIDERS only password sign-in is offered"
        );
    }
}
