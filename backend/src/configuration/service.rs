use crate::common::AppState;
use crate::configuration::constants::ConfigurationKey;
use crate::configuration::repository;
use tracing::error;

pub async fn is_admin_user_setup_done(app_state: &AppState) -> bool {
    is_setup_done(app_state, ConfigurationKey::AdminUserSetup).await
}

pub async fn is_demo_data_setup_done(app_state: &AppState) -> bool {
    is_setup_done(app_state, ConfigurationKey::DemoDataSetup).await
}

async fn is_setup_done(app_state: &AppState, key: ConfigurationKey) -> bool {
    match repository::find_by_key(&app_state.database_connection, key.as_str()).await {
        Ok(Some(model)) => model.value.eq_ignore_ascii_case("true"),
        Ok(None) => false,
        Err(e) => {
            error!("Failed to get configuration {}: {}", key, e);
            false
        }
    }
}

pub async fn mark_admin_user_setup_done(app_state: &AppState) {
    mark_setup_done(app_state, ConfigurationKey::AdminUserSetup).await
}

pub async fn mark_demo_data_setup_done(app_state: &AppState) {
    mark_setup_done(app_state, ConfigurationKey::DemoDataSetup).await
}

async fn mark_setup_done(app_state: &AppState, key: ConfigurationKey) {
    let db = &app_state.database_connection;
    let result = match repository::find_by_key(db, key.as_str()).await {
        Ok(Some(model)) => repository::update_value(db, model, "true").await,
        Ok(None) => {
            error!(
                "Configuration {} not found; migration may not have run",
                key
            );
            return;
        }
        Err(e) => {
            error!("Failed to get configuration {}: {}", key, e);
            return;
        }
    };
    if let Err(e) = result {
        error!("Failed to mark configuration {} as done: {}", key, e);
    }
}

pub async fn get_configuration_value(
    app_state: &AppState,
    key: ConfigurationKey,
) -> Result<Option<String>, sea_orm::DbErr> {
    let configuration =
        repository::find_by_key(&app_state.database_connection, key.as_str()).await?;
    Ok(configuration.map(|found| found.value))
}

pub async fn set_configuration_value(
    app_state: &AppState,
    key: ConfigurationKey,
    value: &str,
) -> Result<(), sea_orm::DbErr> {
    let database_connection = &app_state.database_connection;
    match repository::find_by_key(database_connection, key.as_str()).await? {
        Some(existing) => {
            repository::update_value(database_connection, existing, value).await?;
        }
        None => {
            repository::insert_value(database_connection, key.as_str(), value).await?;
        }
    }
    Ok(())
}
