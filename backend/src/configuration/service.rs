use crate::common::AppState;
use crate::configuration::constants::{ADMIN_USER_SETUP_KEY, DEMO_DATA_SETUP_KEY};
use crate::configuration::repository;
use tracing::error;

pub async fn is_admin_user_setup_done(app_state: &AppState) -> bool {
    is_setup_done(app_state, ADMIN_USER_SETUP_KEY).await
}

pub async fn is_demo_data_setup_done(app_state: &AppState) -> bool {
    is_setup_done(app_state, DEMO_DATA_SETUP_KEY).await
}

async fn is_setup_done(app_state: &AppState, key: &str) -> bool {
    match repository::find_by_key(&app_state.database_connection, key).await {
        Ok(Some(model)) => model.value.eq_ignore_ascii_case("true"),
        Ok(None) => false,
        Err(e) => {
            error!("Failed to get configuration {}: {:?}", key, e);
            false
        }
    }
}

pub async fn mark_admin_user_setup_done(app_state: &AppState) {
    mark_setup_done(app_state, ADMIN_USER_SETUP_KEY).await
}

pub async fn mark_demo_data_setup_done(app_state: &AppState) {
    mark_setup_done(app_state, DEMO_DATA_SETUP_KEY).await
}

async fn mark_setup_done(app_state: &AppState, key: &str) {
    let db = &app_state.database_connection;
    let result = match repository::find_by_key(db, key).await {
        Ok(Some(model)) => repository::update_value(db, model, "true").await,
        Ok(None) => {
            error!(
                "Configuration {} not found; migration may not have run",
                key
            );
            return;
        }
        Err(e) => {
            error!("Failed to get configuration {}: {:?}", key, e);
            return;
        }
    };
    if let Err(e) = result {
        error!("Failed to mark configuration {} as done: {:?}", key, e);
    }
}
