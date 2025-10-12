use crate::migrator;

use sea_orm::{Database, DbErr, DatabaseConnection, ConnectOptions};
use sea_orm_migration::{prelude::*, schema::timestamp_with_time_zone};
use tracing::log::LevelFilter;

#[derive(Iden)]
enum GeneralIds {
    CreatedAt,
    UpdatedAt,
}

pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    let schema_manager = SchemaManager::new(db);
    migrator::Migrator::up(db, None).await?;
    assert!(schema_manager.has_table("dashboard").await?);
    assert!(schema_manager.has_table("connection").await?);
    assert!(schema_manager.has_table("chart").await?);
    assert!(schema_manager.has_table("dashboard_chart_mapping").await?);
    assert!(schema_manager.has_table("user").await?);
    Ok(())
}

pub async fn get_db(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(database_url);
    opt
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Info);
    let db = Database::connect(opt).await?;
    Ok(db)
}

pub fn add_created_at_and_updated_at_timestamps(t: TableCreateStatement) -> TableCreateStatement {
    let mut t = t;
    t.col(timestamp_with_time_zone(GeneralIds::CreatedAt).default(Expr::current_timestamp()))
        .col(timestamp_with_time_zone(GeneralIds::UpdatedAt).default(Expr::current_timestamp()))
        .take()
}
