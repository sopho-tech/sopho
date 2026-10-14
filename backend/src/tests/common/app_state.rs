use crate::common::{AppState, Configurations, OidcProviderConfiguration};
use crate::database::pool::new_database_pool_registry;
use crate::db;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

const TEST_ENCRYPTION_KEY: &str = "test-secret-key-32-chars-long!!";

fn test_database_path(test_name: &str) -> String {
    format!("./data/test_{}.db", test_name)
}

pub fn absolute_test_database_path(test_name: &str) -> String {
    std::fs::canonicalize(test_database_path(test_name))
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

pub async fn init_test_app_state(test_name: &str) -> AppState {
    build_test_app_state(test_name, Vec::new()).await
}

pub async fn init_test_app_state_with_single_sign_on(test_name: &str) -> AppState {
    build_test_app_state(
        test_name,
        vec![OidcProviderConfiguration {
            name: "test".to_string(),
            display_name: "Test".to_string(),
            issuer_url: "https://identity.example.test".to_string(),
            client_id: "client-id".to_string(),
            client_secret: "client-secret".to_string(),
            redirect_uri: "https://app.example.test/api/v1/auth/oidc/test/callback".to_string(),
        }],
    )
    .await
}

async fn build_test_app_state(
    test_name: &str,
    oidc_providers: Vec<OidcProviderConfiguration>,
) -> AppState {
    let database_path = test_database_path(test_name);
    if let Some(parent) = Path::new(&database_path).parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::remove_file(&database_path).ok();

    let database_url = format!("sqlite://{}?mode=rwc", database_path);
    let database_connection = db::get_db(&database_url).await.unwrap();
    db::run_migrations(&database_connection).await.unwrap();

    AppState::new(
        database_connection,
        Configurations::builder()
            .database_url(database_url)
            .encryption_key(TEST_ENCRYPTION_KEY)
            .admin_username("admin")
            .admin_password("test_password")
            .admin_email("admin@test.local")
            .admin_full_name("Test Admin")
            .oidc_providers(oidc_providers)
            .build()
            .unwrap(),
        reqwest::Client::new(),
        Arc::new(RwLock::new(HashMap::new())),
        new_database_pool_registry(),
    )
}
