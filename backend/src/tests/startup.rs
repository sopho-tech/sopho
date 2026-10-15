use crate::authorization::constants::{SystemRole, Visibility};
use crate::authorization::dto::{TenantId, UserId};
use crate::common::AppState;
use crate::tests::common::app_state::init_test_app_state;
use crate::tests::common::resources::{
    insert_canvas, insert_cell, insert_connection, insert_notebook,
};
use crate::tests::common::tenancy::create_tenant_with_owner;
use uuid::Uuid;

const TEST_ADMIN_EMAIL: &str = "admin@test.local";
const DEMO_CANVAS_NAME: &str = "Ecommerce Sales";

async fn admin_memberships(app_state: &AppState) -> Vec<crate::tenant::dto::TenantMembershipDto> {
    let admin = crate::authentication::service::find_user_by_email(app_state, TEST_ADMIN_EMAIL)
        .await
        .unwrap();
    crate::tenant::service::list_tenants_for_user(
        app_state,
        UserId::from_system_unchecked(admin.id),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn a_fresh_database_boots_into_a_usable_default_tenant() {
    let app_state = init_test_app_state("startup_fresh_database").await;

    crate::bootstrap(&app_state).await.unwrap();

    let memberships = admin_memberships(&app_state).await;
    assert_eq!(
        memberships.len(),
        1,
        "the admin owns exactly the default tenant"
    );
    assert_eq!(memberships[0].role_name, SystemRole::Owner.as_str());
    assert!(memberships[0]
        .permissions
        .contains(&"canvas:create".to_string()));

    let admin = crate::authentication::service::find_user_by_email(&app_state, TEST_ADMIN_EMAIL)
        .await
        .unwrap();
    let context = crate::authorization::service::resolve_tenant_context(
        &app_state,
        UserId::from_system_unchecked(admin.id),
        memberships[0].tenant_id,
    )
    .await
    .unwrap();
    let canvases =
        crate::canvas::service::get_all_canvases(app_state.clone(), &context, 0, 50, None)
            .await
            .unwrap();
    assert!(
        canvases
            .data
            .iter()
            .any(|canvas| canvas.name == DEMO_CANVAS_NAME),
        "demo data is seeded into the default tenant"
    );
}

#[tokio::test]
async fn booting_twice_changes_nothing() {
    let app_state = init_test_app_state("startup_second_boot").await;

    crate::bootstrap(&app_state).await.unwrap();
    let first_boot = admin_memberships(&app_state).await;

    crate::bootstrap(&app_state).await.unwrap();
    let second_boot = admin_memberships(&app_state).await;

    assert_eq!(second_boot.len(), 1);
    assert_eq!(second_boot[0].tenant_id, first_boot[0].tenant_id);
}

#[tokio::test]
async fn existence_checks_report_missing_rows_as_missing() {
    let app_state = init_test_app_state("startup_existence_checks").await;
    let fixture = create_tenant_with_owner(&app_state, "existence").await;
    let tenant_id = TenantId::from_system_unchecked(fixture.tenant_id);

    let canvas_id = insert_canvas(
        &app_state,
        fixture.tenant_id,
        "canvas",
        fixture.user_id,
        Visibility::Tenant,
    )
    .await;
    let notebook_id = insert_notebook(&app_state, fixture.tenant_id, canvas_id, "notebook").await;
    let cell_id = insert_cell(&app_state, fixture.tenant_id, notebook_id, "cell").await;
    let connection_id = insert_connection(
        &app_state,
        fixture.tenant_id,
        "connection",
        fixture.user_id,
        Visibility::Tenant,
    )
    .await;

    let unknown_id = Uuid::new_v4();
    assert!(
        crate::notebook::service::does_notebook_exist(&app_state, &tenant_id, notebook_id).await
    );
    assert!(
        !crate::notebook::service::does_notebook_exist(&app_state, &tenant_id, unknown_id).await
    );
    assert!(crate::cell::service::does_cell_exist(&app_state, &tenant_id, cell_id).await);
    assert!(!crate::cell::service::does_cell_exist(&app_state, &tenant_id, unknown_id).await);
    assert!(
        crate::connection::service::does_connection_exist(&app_state, &tenant_id, connection_id)
            .await
    );
    assert!(
        !crate::connection::service::does_connection_exist(&app_state, &tenant_id, unknown_id)
            .await
    );
    assert!(
        !crate::dashboard::service::does_dashboard_exist(&app_state, &tenant_id, unknown_id).await
    );
}
