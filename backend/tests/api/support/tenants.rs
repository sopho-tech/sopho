use crate::support::constants::{
    ACTIVE_STATUS, PLATFORM_MEMBER, SYSTEM_ACTOR, TEST_USER_EMAIL_DOMAIN, TEST_USER_PASSWORD,
};
use crate::support::database::{Column, insert_row, now, select_uuid, update_rows};
use crate::support::test_app::{TestApp, UserSession};
use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, SaltString, rand_core::OsRng};
use reqwest::StatusCode;
use sea_orm::Value;
use uuid::Uuid;

pub struct TestUser {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
}

pub struct TenantMember {
    pub user: TestUser,
    pub session: UserSession,
}

fn system_attribution() -> [Column<'static>; 4] {
    [
        ("created_by_user_id", Value::Uuid(None)),
        ("created_by_actor", SYSTEM_ACTOR.into()),
        ("modified_by_user_id", Value::Uuid(None)),
        ("modified_by_actor", SYSTEM_ACTOR.into()),
    ]
}

pub fn with_system_attribution(mut columns: Vec<Column<'_>>) -> Vec<Column<'_>> {
    columns.extend(system_attribution());
    columns.push(("created_at", now().into()));
    columns.push(("updated_at", now().into()));
    columns
}

fn hashed_test_password() -> String {
    Argon2::default()
        .hash_password(
            TEST_USER_PASSWORD.as_bytes(),
            &SaltString::generate(&mut OsRng),
        )
        .expect("hash the test password")
        .to_string()
}

pub async fn create_user(app: &TestApp, label: &str, platform_role: &str) -> TestUser {
    let id = Uuid::new_v4();
    let email = format!("{label}-{}@{TEST_USER_EMAIL_DOMAIN}", id.simple());
    let full_name = format!("{label} user");
    insert_row(
        app,
        "user",
        &[
            ("id", id.into()),
            ("username", email.clone().into()),
            ("password_hash", hashed_test_password().into()),
            ("email", email.clone().into()),
            ("full_name", full_name.clone().into()),
            ("platform_role", platform_role.into()),
            ("created_at", now().into()),
            ("updated_at", now().into()),
        ],
    )
    .await;
    TestUser {
        id,
        email,
        full_name,
    }
}

pub async fn sign_in(app: &TestApp, user: &TestUser) -> UserSession {
    let session = app.session(None);
    let response = session.sign_in(&user.email, TEST_USER_PASSWORD).await;
    assert_eq!(
        response.status,
        StatusCode::OK,
        "{} could not sign in: {}",
        user.email,
        response.body
    );
    session
}

pub async fn create_tenant(app: &TestApp, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    insert_row(
        app,
        "tenant",
        &with_system_attribution(vec![
            ("id", id.into()),
            ("name", name.into()),
            ("status", ACTIVE_STATUS.into()),
        ]),
    )
    .await;
    app.restart_initialization().await;
    id
}

pub async fn role_id(app: &TestApp, tenant_id: Uuid, role_name: &str) -> Uuid {
    select_uuid(
        app,
        "role",
        "id",
        &[("tenant_id", tenant_id.into()), ("name", role_name.into())],
    )
    .await
}

pub async fn add_member(app: &TestApp, tenant_id: Uuid, user_id: Uuid, role_name: &str) {
    let role_id = role_id(app, tenant_id, role_name).await;
    insert_row(
        app,
        "tenant_membership",
        &with_system_attribution(vec![
            ("id", Uuid::new_v4().into()),
            ("tenant_id", tenant_id.into()),
            ("user_id", user_id.into()),
            ("role_id", role_id.into()),
            ("status", ACTIVE_STATUS.into()),
        ]),
    )
    .await;
}

pub async fn set_membership_status(app: &TestApp, tenant_id: Uuid, user_id: Uuid, status: &str) {
    let updated = update_rows(
        app,
        "tenant_membership",
        &[("status", status.into())],
        &[("tenant_id", tenant_id.into()), ("user_id", user_id.into())],
    )
    .await;
    assert_eq!(updated, 1, "expected one membership to update");
}

pub async fn create_custom_role(
    app: &TestApp,
    tenant_id: Uuid,
    role_name: &str,
    permissions: &[&str],
) -> Uuid {
    let role_id = Uuid::new_v4();
    insert_row(
        app,
        "role",
        &with_system_attribution(vec![
            ("id", role_id.into()),
            ("tenant_id", tenant_id.into()),
            ("name", role_name.into()),
            ("is_system", false.into()),
        ]),
    )
    .await;
    for permission in permissions {
        insert_row(
            app,
            "role_permission",
            &[
                ("id", Uuid::new_v4().into()),
                ("role_id", role_id.into()),
                ("permission", (*permission).into()),
                ("created_at", now().into()),
                ("updated_at", now().into()),
            ],
        )
        .await;
    }
    role_id
}

pub async fn join_tenant(
    app: &TestApp,
    tenant_id: Uuid,
    label: &str,
    role_name: &str,
) -> TenantMember {
    let user = create_user(app, label, PLATFORM_MEMBER).await;
    add_member(app, tenant_id, user.id, role_name).await;
    let session = sign_in(app, &user).await.in_tenant(tenant_id);
    TenantMember { user, session }
}
