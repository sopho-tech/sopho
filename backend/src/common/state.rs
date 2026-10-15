use crate::ai::agent_utils::ModelClient;
use crate::ai_configuration;
use crate::common::cryptography_utils::{
    generate_and_store_encryption_key, get_encryption_key_from_path,
};
use crate::database::pool::{DatabasePoolRegistry, new_database_pool_registry};
use crate::db;
use anyhow::bail;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

#[derive(Clone, Debug, Deserialize)]
pub struct Configurations {
    pub database_url: Cow<'static, str>,
    pub port: u16,
    pub frontend_dir: Cow<'static, str>,
    pub environment: Cow<'static, str>,
    pub cookie_secure: bool,
    pub encryption_key: Cow<'static, str>,
    pub admin_username: Cow<'static, str>,
    pub admin_password: Cow<'static, str>,
    pub admin_email: Cow<'static, str>,
    pub admin_full_name: Cow<'static, str>,
    pub anthropic_base_url: Option<Cow<'static, str>>,
    pub openai_base_url: Option<Cow<'static, str>>,
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

    pub fn build(self) -> anyhow::Result<Configurations> {
        Ok(Configurations {
            database_url: self
                .database_url
                .unwrap_or_else(|| db::get_sqlite_database_file_path().into()),
            port: self.port.unwrap_or(8000),
            frontend_dir: self
                .frontend_dir
                .ok_or_else(|| anyhow::anyhow!("frontend_dir is required"))?,
            environment: self.environment.unwrap_or(Cow::Borrowed("production")),
            cookie_secure: self.cookie_secure.unwrap_or(false),
            encryption_key: self
                .encryption_key
                .ok_or_else(|| anyhow::anyhow!("encryption_key is required"))?,
            admin_username: self
                .admin_username
                .ok_or_else(|| anyhow::anyhow!("admin_username is required"))?,
            admin_password: self
                .admin_password
                .ok_or_else(|| anyhow::anyhow!("admin_password is required"))?,
            admin_email: self
                .admin_email
                .ok_or_else(|| anyhow::anyhow!("admin_email is required"))?,
            admin_full_name: self
                .admin_full_name
                .ok_or_else(|| anyhow::anyhow!("admin_full_name is required"))?,
            anthropic_base_url: validated_base_url("anthropic_base_url", self.anthropic_base_url)?,
            openai_base_url: validated_base_url("openai_base_url", self.openai_base_url)?,
        })
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

    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: match dotenvy::var("DATABASE_URL") {
                Ok(url) => url.into(),
                Err(_) => {
                    let database_url = Cow::Owned(db::get_sqlite_database_file_path());
                    warn!(
                        "Using SQLite as the database. This is not recommended for production use."
                    );
                    database_url
                }
            },
            port: match dotenvy::var("PORT") {
                Ok(port) => port.parse()?,
                _ => 8000,
            },
            frontend_dir: match dotenvy::var("FRONTEND_DIR") {
                Ok(v) => v.into(),
                Err(err) => bail!("missing FRONTEND_DIR: {err}"),
            },
            environment: match dotenvy::var("ENVIRONMENT") {
                Ok(environment) => environment.into(),
                Err(_err) => Cow::Borrowed("production"),
            },
            cookie_secure: match dotenvy::var("COOKIE_SECURE") {
                Ok(cookie_secure) => cookie_secure.parse()?,
                Err(_err) => false,
            },
            encryption_key: match dotenvy::var("ENCRYPTION_KEY") {
                Ok(v) => v.into(),
                Err(_) => {
                    if let Some(key) = get_encryption_key_from_path()? {
                        key.into()
                    } else {
                        let (encryption_key, full_path) = generate_and_store_encryption_key()?;
                        warn!(
                            "Since ENCRYPTION_KEY is not provided, one is generated and stored at {}. This is not recommended for production use.",
                            full_path
                        );
                        encryption_key.into()
                    }
                }
            },
            admin_username: match dotenvy::var("ADMIN_USERNAME") {
                Ok(v) => v.into(),
                Err(err) => bail!("missing ADMIN_USERNAME: {err}"),
            },
            admin_password: match dotenvy::var("ADMIN_PASSWORD") {
                Ok(v) => v.into(),
                Err(err) => bail!("missing ADMIN_PASSWORD: {err}"),
            },
            admin_email: match dotenvy::var("ADMIN_EMAIL") {
                Ok(v) => v.into(),
                Err(err) => bail!("missing ADMIN_EMAIL: {err}"),
            },
            admin_full_name: match dotenvy::var("ADMIN_FULL_NAME") {
                Ok(v) => v.into(),
                Err(err) => bail!("missing ADMIN_FULL_NAME: {err}"),
            },
            anthropic_base_url: validated_base_url(
                "ANTHROPIC_BASE_URL",
                optional_environment_variable("ANTHROPIC_BASE_URL")?,
            )?,
            openai_base_url: validated_base_url(
                "OPENAI_BASE_URL",
                optional_environment_variable("OPENAI_BASE_URL")?,
            )?,
        })
    }
}

fn validated_base_url(
    setting_name: &str,
    base_url: Option<Cow<'static, str>>,
) -> anyhow::Result<Option<Cow<'static, str>>> {
    let Some(base_url) = base_url else {
        return Ok(None);
    };
    let parsed_base_url: http::Uri = base_url
        .parse()
        .map_err(|error| anyhow::anyhow!("invalid {setting_name} {base_url}: {error}"))?;
    let uses_http_scheme = matches!(parsed_base_url.scheme_str(), Some("http" | "https"));
    if !uses_http_scheme || parsed_base_url.authority().is_none() {
        bail!("invalid {setting_name} {base_url}: expected an http or https URL");
    }
    Ok(Some(base_url))
}

fn optional_environment_variable(name: &str) -> anyhow::Result<Option<Cow<'static, str>>> {
    match dotenvy::var(name) {
        Ok(value) => Ok(Some(value.into())),
        Err(dotenvy::Error::EnvVar(std::env::VarError::NotPresent)) => Ok(None),
        Err(err) => bail!("invalid {name}: {err}"),
    }
}

#[derive(Clone)]
pub struct AppState {
    pub database_connection: DatabaseConnection,
    pub config: Configurations,
    pub model_client: Arc<RwLock<Option<ModelClient>>>,
    pub database_pools: DatabasePoolRegistry,
}

impl AppState {
    pub async fn from_configurations(config: Configurations) -> anyhow::Result<Self> {
        let database_connection = db::get_db(&config.database_url).await?;
        Ok(Self {
            database_connection,
            config,
            model_client: Arc::new(RwLock::new(None)),
            database_pools: new_database_pool_registry(),
        })
    }

    pub async fn current_model_client(&self) -> Option<ModelClient> {
        self.model_client.read().await.clone()
    }

    pub async fn require_model_client(&self) -> anyhow::Result<ModelClient> {
        self.current_model_client()
            .await
            .ok_or_else(|| anyhow::anyhow!("AI is not configured"))
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
