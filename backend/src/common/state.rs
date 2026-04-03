// TODO anthropic api should not be mandatory for running backend. Make it like a feature flag.

use crate::ai::agent_utils;
use crate::common::cryptography_utils::{
    generate_and_store_encryption_key, get_encryption_key_from_path,
};
use crate::db;
use anyhow;
use anyhow::bail;
use reqwest;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::borrow::Cow;
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
    pub anthropic_api_key: Cow<'static, str>,
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
    anthropic_api_key: Option<Cow<'static, str>>,
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
            anthropic_api_key: None,
        }
    }

    pub fn database_url(mut self, v: impl Into<Cow<'static, str>>) -> Self {
        self.database_url = Some(v.into());
        self
    }

    pub fn port(mut self, v: u16) -> Self {
        self.port = Some(v);
        self
    }

    pub fn frontend_dir(mut self, v: impl Into<Cow<'static, str>>) -> Self {
        self.frontend_dir = Some(v.into());
        self
    }

    pub fn environment(mut self, v: impl Into<Cow<'static, str>>) -> Self {
        self.environment = Some(v.into());
        self
    }

    pub fn cookie_secure(mut self, v: bool) -> Self {
        self.cookie_secure = Some(v);
        self
    }

    pub fn encryption_key(mut self, v: impl Into<Cow<'static, str>>) -> Self {
        self.encryption_key = Some(v.into());
        self
    }

    pub fn admin_username(mut self, v: impl Into<Cow<'static, str>>) -> Self {
        self.admin_username = Some(v.into());
        self
    }

    pub fn admin_password(mut self, v: impl Into<Cow<'static, str>>) -> Self {
        self.admin_password = Some(v.into());
        self
    }

    pub fn admin_email(mut self, v: impl Into<Cow<'static, str>>) -> Self {
        self.admin_email = Some(v.into());
        self
    }

    pub fn admin_full_name(mut self, v: impl Into<Cow<'static, str>>) -> Self {
        self.admin_full_name = Some(v.into());
        self
    }

    pub fn anthropic_api_key(mut self, v: impl Into<Cow<'static, str>>) -> Self {
        self.anthropic_api_key = Some(v.into());
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
                .unwrap_or_else(|| Cow::Borrowed("/app/frontend/dist/")),
            environment: self
                .environment
                .unwrap_or_else(|| Cow::Borrowed("production")),
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
            anthropic_api_key: self
                .anthropic_api_key
                .ok_or_else(|| anyhow::anyhow!("anthropic_api_key is required"))?,
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
        dotenv::dotenv().ok();

        Ok(Self {
            database_url: match dotenv::var("DATABASE_URL") {
                Ok(url) => url.into(),
                Err(_) => {
                    let database_url = Cow::Owned(db::get_sqlite_database_file_path());
                    warn!(
                        "Using SQLite as the database. This is not recommended for production use."
                    );
                    database_url
                }
            },
            port: match dotenv::var("PORT") {
                Ok(port) => port.parse()?,
                _ => 8000,
            },
            frontend_dir: Cow::Borrowed("/app/frontend/dist/"),
            environment: match dotenv::var("ENVIRONMENT") {
                Ok(environment) => environment.into(),
                Err(_err) => Cow::Borrowed("production"),
            },
            cookie_secure: match dotenv::var("COOKIE_SECURE") {
                Ok(cookie_secure) => cookie_secure.parse()?,
                Err(_err) => false,
            },
            encryption_key: match dotenv::var("ENCRYPTION_KEY") {
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
            admin_username: match dotenv::var("ADMIN_USERNAME") {
                Ok(v) => v.into(),
                Err(err) => bail!("missing ADMIN_USERNAME: {err}"),
            },
            admin_password: match dotenv::var("ADMIN_PASSWORD") {
                Ok(v) => v.into(),
                Err(err) => bail!("missing ADMIN_PASSWORD: {err}"),
            },
            admin_email: match dotenv::var("ADMIN_EMAIL") {
                Ok(v) => v.into(),
                Err(err) => bail!("missing ADMIN_EMAIL: {err}"),
            },
            admin_full_name: match dotenv::var("ADMIN_FULL_NAME") {
                Ok(v) => v.into(),
                Err(err) => bail!("missing ADMIN_FULL_NAME: {err}"),
            },
            anthropic_api_key: match dotenv::var("ANTHROPIC_API_KEY") {
                Ok(v) => v.into(),
                Err(err) => bail!("missing ANTHROPIC_API_KEY: {err}"),
            },
        })
    }
}

#[derive(Clone)]
pub struct AppState {
    pub database_connection: DatabaseConnection,
    pub config: Configurations,
    pub client: reqwest::Client,
    pub model_client: agent_utils::ModelClient,
}

impl AppState {
    pub fn new(
        database_connection: DatabaseConnection,
        config: Configurations,
        client: reqwest::Client,
        model_client: agent_utils::ModelClient,
    ) -> Self {
        Self {
            database_connection,
            config,
            client,
            model_client,
        }
    }

    pub async fn from_env() -> anyhow::Result<Self> {
        let config = Configurations::from_env()?;
        let database_connection = db::get_db(&config.database_url).await.unwrap();
        let client = reqwest::Client::new();
        let model_client = agent_utils::ModelClient::anthropic(config.anthropic_api_key.as_ref())?;
        Ok(Self::new(database_connection, config, client, model_client))
    }
}
