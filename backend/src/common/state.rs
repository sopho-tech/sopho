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
}

impl Configurations {
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
                Err(err) => bail!("missing ENVIRONMENT: {err}"),
            },
            cookie_secure: match dotenv::var("COOKIE_SECURE") {
                Ok(cookie_secure) => cookie_secure.parse()?,
                Err(err) => bail!("missing COOKIE_SECURE: {err}"),
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
        })
    }
}

#[derive(Clone)]
pub struct AppState {
    pub database_connection: DatabaseConnection,
    pub config: Configurations,
    pub client: reqwest::Client,
}

impl AppState {
    pub async fn from_env() -> anyhow::Result<Self> {
        let config = Configurations::from_env()?;
        let database_connection = db::get_db(&config.database_url).await.unwrap();
        Ok(Self {
            database_connection,
            config,
            client: reqwest::Client::new(),
        })
    }
}
