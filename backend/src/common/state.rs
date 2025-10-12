use sea_orm::DatabaseConnection;
use std::borrow::Cow;
use anyhow::bail;
use serde::Deserialize;
use anyhow;
use crate::db;
use reqwest;


#[derive(Clone, Debug, Deserialize)]
pub struct Configurations {
    pub database_url: Cow<'static, str>,
    pub port: u16,
    pub frontend_dir: Cow<'static, str>,
    pub google_client_id: Cow<'static, str>,
    pub google_client_secret: Cow<'static, str>,
    pub google_redirect_uri: Cow<'static, str>,
    pub environment: Cow<'static, str>,
    pub cookie_domain: Cow<'static, str>,
    pub cookie_secure: bool,
}

impl Configurations {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv::dotenv().ok();

        Ok(Self {
            database_url: match dotenv::var("DATABASE_URL") {
                Ok(url) => url.into(),
                Err(err) => bail!("missing DATABASE_URL: {err}"),
            },
            port: match dotenv::var("PORT") {
                Ok(port) => port.parse()?,
                _ => 8000,
            },  
            frontend_dir: match dotenv::var("FRONTEND_DIR") {
                Ok(frontend_dir) => frontend_dir.into(),
                Err(err) => bail!("missing FRONTEND_DIR: {err}"),
            },
            google_client_id: match dotenv::var("GOOGLE_CLIENT_ID") {
                Ok(google_client_id) => google_client_id.into(),
                Err(err) => bail!("missing GOOGLE_CLIENT_ID: {err}"),
            },
            google_client_secret: match dotenv::var("GOOGLE_CLIENT_SECRET") {
                Ok(google_client_secret) => google_client_secret.into(),
                Err(err) => bail!("missing GOOGLE_CLIENT_SECRET: {err}"),
            },
            google_redirect_uri: match dotenv::var("GOOGLE_REDIRECT_URI") {
                Ok(google_redirect_uri) => google_redirect_uri.into(),
                Err(err) => bail!("missing GOOGLE_REDIRECT_URI: {err}"),
            },
            environment: match dotenv::var("ENVIRONMENT") {
                Ok(environment) => environment.into(),
                Err(err) => bail!("missing ENVIRONMENT: {err}"),
            },
            cookie_domain: match dotenv::var("COOKIE_DOMAIN") {
                Ok(cookie_domain) => cookie_domain.into(),
                Err(err) => bail!("missing COOKIE_DOMAIN: {err}"),
            },
            cookie_secure: match dotenv::var("COOKIE_SECURE") {
                Ok(cookie_secure) => cookie_secure.parse()?,
                Err(err) => bail!("missing COOKIE_SECURE: {err}"),
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
