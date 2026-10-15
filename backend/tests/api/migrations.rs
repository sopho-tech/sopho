use crate::support::TestApp;
use migration::{Migrator, MigratorTrait};

async fn applied_migration_count(app: &TestApp) -> usize {
    Migrator::get_applied_migrations(&app.app_state.database_connection)
        .await
        .expect("read the applied migrations")
        .len()
}

#[tokio::test]
async fn migrations_roll_back_to_the_start_and_reapply() {
    let app = TestApp::spawn().await;
    let total_migrations = Migrator::migrations().len();
    assert_eq!(applied_migration_count(&app).await, total_migrations);

    Migrator::down(&app.app_state.database_connection, None)
        .await
        .expect("roll back every migration");
    assert_eq!(applied_migration_count(&app).await, 0);

    Migrator::up(&app.app_state.database_connection, None)
        .await
        .expect("reapply every migration");
    assert_eq!(applied_migration_count(&app).await, total_migrations);
}

#[tokio::test]
async fn fresh_rebuilds_the_schema() {
    let app = TestApp::spawn().await;

    Migrator::fresh(&app.app_state.database_connection)
        .await
        .expect("drop and reapply every migration");

    assert_eq!(
        applied_migration_count(&app).await,
        Migrator::migrations().len()
    );
}
