use crate::support::constants::{
    ADMIN_EMAIL, DEFAULT_TENANT_CONFIGURATION_KEY, DEFAULT_TENANT_NAME, LISTED_ADMINISTRATOR_EMAIL,
    OWNER_ROLE, SINGLE_SIGN_ON_PROVIDER, SYSTEM_ROLES,
};
use crate::support::database::{count_rows, insert_row};
use crate::support::identity_provider::sign_in_with_provider;
use crate::support::tenants::{create_tenant, with_system_attribution};
use crate::support::{DeploymentFlags, TestApp, UserSession};
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

const SEEDED_TABLES: [&str; 6] = [
    "user",
    "tenant",
    "tenant_membership",
    "connection",
    "canvas",
    "cell",
];

async fn seeded_row_counts(app: &TestApp) -> Vec<i64> {
    let mut row_counts = Vec::with_capacity(SEEDED_TABLES.len());
    for table in SEEDED_TABLES {
        row_counts.push(count_rows(app, table, &[]).await);
    }
    row_counts
}

async fn spawn_passwordless() -> TestApp {
    TestApp::spawn_with_deployment_flags(DeploymentFlags::passwordless(LISTED_ADMINISTRATOR_EMAIL))
        .await
}

async fn record_default_tenant_id(app: &TestApp, default_tenant_id: Uuid) {
    insert_row(
        app,
        "configuration",
        &[
            ("key", DEFAULT_TENANT_CONFIGURATION_KEY.into()),
            ("value", default_tenant_id.to_string().into()),
        ],
    )
    .await;
}

async fn signed_in_through_provider(app: &TestApp, email: &str) -> UserSession {
    let session = app.session(None);
    let outcome = sign_in_with_provider(
        &session,
        SINGLE_SIGN_ON_PROVIDER,
        &format!("subject-{}", Uuid::new_v4().simple()),
        json!({ "email": email, "email_verified": true, "name": "A Person" }),
    )
    .await;
    assert!(
        outcome.is_signed_in(),
        "sign-in went to {}",
        outcome.location
    );
    session
}

#[tokio::test]
async fn a_passwordless_deployment_starts_with_no_users_tenants_or_demo_data() {
    let app = spawn_passwordless().await;
    assert_eq!(seeded_row_counts(&app).await, vec![0; SEEDED_TABLES.len()]);
    assert_eq!(
        count_rows(
            &app,
            "configuration",
            &[("key", DEFAULT_TENANT_CONFIGURATION_KEY.into())]
        )
        .await,
        0
    );

    app.restart_initialization().await;
    app.restart_initialization().await;

    assert_eq!(seeded_row_counts(&app).await, vec![0; SEEDED_TABLES.len()]);
}

#[tokio::test]
async fn the_first_listed_sign_in_becomes_administrator_and_owns_a_new_tenant() {
    let app = spawn_passwordless().await;
    let founder = signed_in_through_provider(&app, LISTED_ADMINISTRATOR_EMAIL).await;
    let profile = founder.get("/auth/me").await;
    assert!(
        !profile.body["platform_permissions"]
            .as_array()
            .expect("the platform permissions")
            .is_empty(),
        "{}",
        profile.body
    );
    assert_eq!(
        founder.get("/tenant").await.body,
        json!([]),
        "a fresh deployment has no tenant to belong to"
    );

    let created = founder.post("/tenant", json!({ "name": "Sopho" })).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    let memberships = founder.get("/tenant").await;
    assert_eq!(memberships.body[0]["tenant_id"], created.body["id"]);
    assert_eq!(memberships.body[0]["role_name"], OWNER_ROLE);

    let newcomer = signed_in_through_provider(&app, "newcomer@example.test").await;
    assert_eq!(
        newcomer.get("/auth/me").await.body["platform_permissions"],
        json!([])
    );
    let own_tenant = newcomer.post("/tenant", json!({ "name": "Theirs" })).await;
    assert_eq!(
        own_tenant.status,
        StatusCode::CREATED,
        "{}",
        own_tenant.body
    );
}

#[tokio::test]
async fn a_default_tenant_left_by_the_old_migration_is_adopted_not_duplicated() {
    let app = spawn_passwordless().await;
    let migrated_tenant_id = create_tenant(&app, DEFAULT_TENANT_NAME).await;
    record_default_tenant_id(&app, migrated_tenant_id).await;

    app.initialize_again_with_deployment_flags(DeploymentFlags::default())
        .await
        .expect("start with the default flags");

    assert_eq!(count_rows(&app, "tenant", &[]).await, 1);
    for system_role in SYSTEM_ROLES {
        assert_eq!(
            count_rows(
                &app,
                "role",
                &[
                    ("tenant_id", migrated_tenant_id.into()),
                    ("name", system_role.into()),
                ],
            )
            .await,
            1
        );
    }
    assert_eq!(
        count_rows(&app, "user", &[("email", ADMIN_EMAIL.into())]).await,
        1
    );
    assert_eq!(
        count_rows(
            &app,
            "tenant_membership",
            &[("tenant_id", migrated_tenant_id.into())]
        )
        .await,
        1,
        "the seeded administrator owns the adopted tenant"
    );
    assert_eq!(
        count_rows(&app, "canvas", &[("tenant_id", migrated_tenant_id.into())]).await,
        1,
        "the demo is seeded into the adopted tenant"
    );

    let counts_after_first_start = seeded_row_counts(&app).await;
    app.initialize_again_with_deployment_flags(DeploymentFlags::default())
        .await
        .expect("start again with the default flags");
    assert_eq!(seeded_row_counts(&app).await, counts_after_first_start);
}

#[tokio::test]
async fn a_recorded_default_tenant_that_does_not_exist_stops_startup() {
    let app = spawn_passwordless().await;
    let missing_tenant_id = Uuid::new_v4();
    record_default_tenant_id(&app, missing_tenant_id).await;

    let startup_error = app
        .initialize_again_with_deployment_flags(DeploymentFlags::default())
        .await
        .expect_err("startup must stop");

    let message = startup_error.to_string();
    assert!(
        message.contains(&missing_tenant_id.to_string()) && message.contains("does not exist"),
        "{message}"
    );
    assert_eq!(count_rows(&app, "tenant", &[]).await, 0);
}

#[tokio::test]
async fn turning_seeding_off_leaves_what_was_already_seeded() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let counts_with_defaults = seeded_row_counts(&app).await;
    assert!(
        counts_with_defaults.iter().all(|row_count| *row_count > 0),
        "{counts_with_defaults:?}"
    );

    app.initialize_again_with_deployment_flags(DeploymentFlags::passwordless(
        LISTED_ADMINISTRATOR_EMAIL,
    ))
    .await
    .expect("start with seeding off");

    assert_eq!(seeded_row_counts(&app).await, counts_with_defaults);
    assert_eq!(
        count_rows(&app, "user", &[("email", ADMIN_EMAIL.into())]).await,
        1
    );
    assert_eq!(
        count_rows(&app, "tenant", &[("id", app.tenant_id.into())]).await,
        1
    );
}

#[tokio::test]
async fn a_migrated_default_tenant_without_roles_gets_its_roles() {
    let app = spawn_passwordless().await;
    let tenant_id = Uuid::new_v4();
    insert_row(
        &app,
        "tenant",
        &with_system_attribution(vec![
            ("id", tenant_id.into()),
            ("name", DEFAULT_TENANT_NAME.into()),
            ("status", "ACTIVE".into()),
        ]),
    )
    .await;
    record_default_tenant_id(&app, tenant_id).await;

    app.initialize_again_with_deployment_flags(DeploymentFlags::default())
        .await
        .expect("a tenant with no roles yet is adopted");

    assert_eq!(
        count_rows(&app, "role", &[("tenant_id", tenant_id.into())]).await,
        SYSTEM_ROLES.len() as i64
    );
}
