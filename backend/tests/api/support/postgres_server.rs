use crate::support::constants::{
    POSTGRES_ADMIN_DATABASE, POSTGRES_CONTAINER_NAME, POSTGRES_IMAGE_NAME, POSTGRES_IMAGE_TAG,
    POSTGRES_PASSWORD, POSTGRES_PORT, POSTGRES_READY_MESSAGE, POSTGRES_READY_POLL_MILLISECONDS,
    POSTGRES_READY_TIMEOUT_SECONDS, POSTGRES_STARTUP_LOCK_FILE_NAME, POSTGRES_USER,
    TEST_POSTGRES_URL_VARIABLE,
};
use percent_encoding::percent_decode_str;
use sqlx::{Connection, Executor, PgConnection};
use std::fs::File;
use std::time::{Duration, Instant};
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt, ReuseDirective};
use tokio::sync::OnceCell;
use uuid::Uuid;

pub struct PostgresServer {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    admin_database: String,
}

impl PostgresServer {
    pub fn url_for(&self, database_name: &str) -> String {
        let mut url = reqwest::Url::parse(&format!(
            "postgres://{}:{}/{database_name}",
            self.host, self.port
        ))
        .expect("build the Postgres URL");
        url.set_username(&self.username)
            .expect("set the Postgres username");
        url.set_password(Some(&self.password))
            .expect("set the Postgres password");
        url.to_string()
    }

    pub fn admin_url(&self) -> String {
        self.url_for(&self.admin_database)
    }
}

fn decoded_url_part(encoded_part: &str, part_name: &str) -> String {
    percent_decode_str(encoded_part)
        .decode_utf8()
        .unwrap_or_else(|error| {
            panic!("the {part_name} in {TEST_POSTGRES_URL_VARIABLE} is not UTF-8: {error}")
        })
        .into_owned()
}

fn postgres_server_from_url(url: &str) -> PostgresServer {
    let parsed_url = reqwest::Url::parse(url)
        .unwrap_or_else(|error| panic!("{TEST_POSTGRES_URL_VARIABLE} is not a URL: {error}"));
    let username = decoded_url_part(parsed_url.username(), "username");
    if username.is_empty() {
        panic!("{TEST_POSTGRES_URL_VARIABLE} must include a username");
    }
    let password = decoded_url_part(
        parsed_url
            .password()
            .unwrap_or_else(|| panic!("{TEST_POSTGRES_URL_VARIABLE} must include a password")),
        "password",
    );
    let admin_database = decoded_url_part(parsed_url.path().trim_start_matches('/'), "database");
    if admin_database.is_empty() {
        panic!(
            "{TEST_POSTGRES_URL_VARIABLE} must name the database the tests connect to when they create their own databases"
        );
    }
    PostgresServer {
        host: parsed_url
            .host_str()
            .unwrap_or_else(|| panic!("{TEST_POSTGRES_URL_VARIABLE} has no host"))
            .to_string(),
        port: parsed_url.port().unwrap_or(POSTGRES_PORT),
        username,
        password,
        admin_database,
    }
}

static SHARED_POSTGRES_SERVER: OnceCell<PostgresServer> = OnceCell::const_new();

pub async fn shared_postgres_server() -> &'static PostgresServer {
    SHARED_POSTGRES_SERVER
        .get_or_init(start_or_reuse_postgres_server)
        .await
}

pub struct PostgresDatabase {
    pub name: String,
    pub url: String,
    admin_url: String,
}

impl PostgresDatabase {
    pub async fn create(name_prefix: &str) -> Self {
        let server = shared_postgres_server().await;
        let name = format!("{name_prefix}_{}", Uuid::new_v4().simple());
        let admin_url = server.admin_url();
        let mut connection = PgConnection::connect(&admin_url)
            .await
            .expect("connect to the Postgres test server");
        connection
            .execute(format!("CREATE DATABASE \"{name}\"").as_str())
            .await
            .expect("create a Postgres test database");
        Self {
            url: server.url_for(&name),
            name,
            admin_url,
        }
    }
}

impl Drop for PostgresDatabase {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("kept Postgres database {} for inspection", self.name);
            return;
        }
        let admin_url = self.admin_url.clone();
        let name = self.name.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("build a runtime to drop the test database");
            runtime.block_on(async move {
                let mut connection = PgConnection::connect(&admin_url)
                    .await
                    .expect("connect to the Postgres test server");
                connection
                    .execute(format!("DROP DATABASE \"{name}\" WITH (FORCE)").as_str())
                    .await
                    .expect("drop the Postgres test database");
            });
        })
        .join()
        .expect("dropping the Postgres test database failed");
    }
}

async fn start_or_reuse_postgres_server() -> PostgresServer {
    if let Ok(url) = std::env::var(TEST_POSTGRES_URL_VARIABLE) {
        let server = postgres_server_from_url(&url);
        wait_until_ready(&server.admin_url()).await;
        return server;
    }
    let startup_lock = tokio::task::spawn_blocking(|| {
        let lock_file = File::create(std::env::temp_dir().join(POSTGRES_STARTUP_LOCK_FILE_NAME))
            .expect("create the Postgres startup lock file");
        lock_file
            .lock()
            .expect("lock the Postgres startup lock file");
        lock_file
    })
    .await
    .expect("acquire the Postgres startup lock");
    let container = GenericImage::new(POSTGRES_IMAGE_NAME, POSTGRES_IMAGE_TAG)
        .with_exposed_port(POSTGRES_PORT.tcp())
        .with_wait_for(WaitFor::message_on_stderr(POSTGRES_READY_MESSAGE))
        .with_env_var("POSTGRES_USER", POSTGRES_USER)
        .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
        .with_container_name(POSTGRES_CONTAINER_NAME)
        .with_reuse(ReuseDirective::Always)
        .start()
        .await
        .unwrap_or_else(|error| {
            panic!(
                "Postgres-backed tests need a running Docker daemon (set DOCKER_HOST for OrbStack or colima) or {TEST_POSTGRES_URL_VARIABLE} pointing at a Postgres server: {error}"
            )
        });
    let server = PostgresServer {
        host: container
            .get_host()
            .await
            .expect("read the Postgres container host")
            .to_string(),
        port: container
            .get_host_port_ipv4(POSTGRES_PORT)
            .await
            .expect("read the Postgres container port"),
        username: POSTGRES_USER.to_string(),
        password: POSTGRES_PASSWORD.to_string(),
        admin_database: POSTGRES_ADMIN_DATABASE.to_string(),
    };
    wait_until_ready(&server.admin_url()).await;
    drop(startup_lock);
    server
}

async fn wait_until_ready(admin_url: &str) {
    let deadline = Instant::now() + Duration::from_secs(POSTGRES_READY_TIMEOUT_SECONDS);
    loop {
        match PgConnection::connect(admin_url).await {
            Ok(connection) => {
                connection
                    .close()
                    .await
                    .expect("close the readiness connection");
                return;
            }
            Err(_) if Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(POSTGRES_READY_POLL_MILLISECONDS)).await;
            }
            Err(error) => panic!("Postgres at {admin_url} did not become ready: {error}"),
        }
    }
}
