use crate::authorization::constants::{PlatformPermission, SystemRole};
use crate::authorization::dto::UserId;
use crate::authorization::errors::AuthorizationError;
use crate::common::AppState;
use crate::tenant::error::TenantError;
use crate::tests::common::app_state::init_test_app_state;
use crate::tests::common::tenancy::insert_user;

const TEST_ADMIN_EMAIL: &str = "admin@test.local";

async fn administrator(app_state: &AppState) -> UserId {
    let admin = crate::authentication::service::find_user_by_email(app_state, TEST_ADMIN_EMAIL)
        .await
        .unwrap();
    UserId::from_system_unchecked(admin.id)
}

async fn tenant_count(app_state: &AppState) -> usize {
    crate::tenant::service::list_all_tenant_ids(app_state)
        .await
        .unwrap()
        .len()
}

#[tokio::test]
async fn only_platform_administrators_create_workspaces() {
    let app_state = init_test_app_state("platform_workspace_creation").await;
    crate::initialize(&app_state).await.unwrap();
    let tenants_before = tenant_count(&app_state).await;

    let member = UserId::from_system_unchecked(insert_user(&app_state, "member").await);
    let refused = crate::tenant::service::create_tenant_for_user(
        &app_state,
        member,
        "Side project".to_string(),
    )
    .await;
    assert!(matches!(
        refused,
        Err(TenantError::Unauthorized(AuthorizationError::MissingPermission(ref permission)))
            if permission == PlatformPermission::TenantCreate.as_str()
    ));
    assert_eq!(
        tenant_count(&app_state).await,
        tenants_before,
        "a refused request creates nothing"
    );

    let administrator = administrator(&app_state).await;
    let created = crate::tenant::service::create_tenant_for_user(
        &app_state,
        administrator,
        "  Analytics  ".to_string(),
    )
    .await
    .unwrap();
    assert_eq!(created.name, "Analytics");
    let memberships = crate::tenant::service::list_tenants_for_user(&app_state, administrator)
        .await
        .unwrap();
    let membership = memberships
        .iter()
        .find(|membership| membership.tenant_id == created.id)
        .unwrap();
    assert_eq!(membership.role_name, SystemRole::Owner.as_str());

    assert!(matches!(
        crate::tenant::service::create_tenant_for_user(&app_state, administrator, "   ".into())
            .await,
        Err(TenantError::NameRequired)
    ));
}

#[tokio::test]
async fn current_user_reports_platform_permissions() {
    let app_state = init_test_app_state("platform_current_user").await;
    crate::initialize(&app_state).await.unwrap();

    let administrator_profile = crate::authentication::service::current_user_profile(
        &app_state,
        administrator(&app_state).await,
    )
    .await
    .unwrap();
    assert_eq!(
        administrator_profile.platform_permissions,
        vec![PlatformPermission::TenantCreate.as_str().to_string()]
    );

    let member = UserId::from_system_unchecked(insert_user(&app_state, "member").await);
    let member_profile = crate::authentication::service::current_user_profile(&app_state, member)
        .await
        .unwrap();
    assert!(member_profile.platform_permissions.is_empty());
}
