use crate::support::constants::{
    DEMO_DATABASE_FILE_NAME, DEMO_DATABASE_RELATIVE_PATH, SAMPLE_DATA_RELATIVE_PATH,
    SAMPLE_DATABASE_PREFIX,
};
use crate::support::postgres_server::PostgresDatabase;
use sqlx::{Connection, PgConnection};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct SqliteDemoDatabase {
    directory: TempDir,
}

impl SqliteDemoDatabase {
    pub fn path(&self) -> PathBuf {
        self.directory.path().join(DEMO_DATABASE_FILE_NAME)
    }
}

pub fn copy_sqlite_demo_database() -> SqliteDemoDatabase {
    let demo_database = SqliteDemoDatabase {
        directory: tempfile::tempdir().expect("create a temporary directory"),
    };
    std::fs::copy(
        manifest_path(DEMO_DATABASE_RELATIVE_PATH),
        demo_database.path(),
    )
    .expect("copy the demo database");
    demo_database
}

pub async fn create_postgres_sample_database() -> PostgresDatabase {
    let database = PostgresDatabase::create(SAMPLE_DATABASE_PREFIX).await;
    let sample_data = std::fs::read_to_string(manifest_path(SAMPLE_DATA_RELATIVE_PATH))
        .expect("read the Postgres sample data");
    let mut connection = PgConnection::connect(&database.url)
        .await
        .expect("connect to the Postgres sample database");
    sqlx::raw_sql(&sample_data)
        .execute(&mut connection)
        .await
        .expect("load the Postgres sample data");
    database
}

pub fn manifest_path(relative_path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}
