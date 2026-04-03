use std::path::PathBuf;

use migration::MigratorTrait;

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

use tracing::log::LevelFilter;

pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    let schema_manager = migration::SchemaManager::new(db);
    migration::Migrator::up(db, None).await?;
    assert!(schema_manager.has_table("configuration").await?);
    assert!(schema_manager.has_table("dashboard").await?);
    assert!(schema_manager.has_table("connection").await?);
    assert!(schema_manager.has_table("chart").await?);
    assert!(schema_manager.has_table("canvas").await?);
    assert!(schema_manager.has_table("user").await?);
    assert!(schema_manager.has_table("notebook").await?);
    assert!(schema_manager.has_table("cell").await?);
    Ok(())
}

pub async fn get_db(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(database_url);
    opt.sqlx_logging(true).sqlx_logging_level(LevelFilter::Info);
    let db = Database::connect(opt).await?;
    Ok(db)
}

pub fn get_sqlite_database_file_path() -> String {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let db_path = current_dir.join("data").join("db.sqlite");
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let path_str = db_path.to_string_lossy().replace('\\', "/");
    format!("sqlite:///{}?mode=rwc", path_str)
}
