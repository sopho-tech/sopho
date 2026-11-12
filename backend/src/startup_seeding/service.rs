use crate::authentication::service as authentication_service;
use crate::common::AppState;
use crate::demo::service as demo_service;
use crate::feature_flag::constants::FeatureFlag;
use crate::feature_flag::service::is_feature_enabled;
use crate::tenant::service as tenant_service;

pub async fn run_startup_seeding(app_state: &AppState) -> anyhow::Result<()> {
    if is_feature_enabled(app_state, FeatureFlag::SeedPasswordAdministrator) {
        authentication_service::seed_admin_user(app_state).await?;
    }
    if is_feature_enabled(app_state, FeatureFlag::DefaultTenant) {
        let seeded_administrator =
            authentication_service::find_seeded_password_administrator(app_state).await?;
        let default_tenant_id =
            tenant_service::seed_default_tenant_with_owner(app_state, seeded_administrator.id)
                .await?;
        demo_service::seed_demo_data(app_state, default_tenant_id, seeded_administrator.id).await?;
    }
    Ok(())
}
