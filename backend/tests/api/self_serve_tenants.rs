use crate::support::canvas_builder::string_field;
use crate::support::constants::{
    MAXIMUM_TENANT_NAME_CHARACTERS, MEMBER_ROLE, ONE_TENANT_LIMIT_MESSAGE, OWNER_ROLE,
    PLATFORM_MEMBER, SELF_SERVE_TENANT_CREATION_TABLE, SYSTEM_ROLES, TENANT_CREATE_PERMISSION,
    TENANT_CREATION_CAPABILITY_PATH, TENANT_CREATION_LIMIT_REACHED_CODE,
    TENANT_NAME_TOO_LONG_MESSAGE, TWO_TENANT_LIMIT_MESSAGE,
};
use crate::support::database::count_rows;
use crate::support::tenants::{TestUser, add_member, create_tenant, create_user, sign_in};
use crate::support::{TestApp, UserSession};
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

struct SignedInMember {
    user: TestUser,
    session: UserSession,
}

async fn signed_in_member(app: &TestApp, label: &str) -> SignedInMember {
    let user = create_user(app, label, PLATFORM_MEMBER).await;
    let session = sign_in(app, &user).await;
    SignedInMember { user, session }
}

async fn create_own_tenant(session: &UserSession, name: &str) -> Uuid {
    let created = session.post("/tenant", json!({ "name": name })).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    Uuid::parse_str(&string_field(&created.body, "id")).expect("the created tenant id")
}

async fn creation_records_for(app: &TestApp, user_id: Uuid) -> i64 {
    count_rows(
        app,
        SELF_SERVE_TENANT_CREATION_TABLE,
        &[("creator_user_id", user_id.into())],
    )
    .await
}

fn creation_allowed() -> Value {
    json!({ "can_create_tenant": true, "limit_reached_message": null })
}

#[tokio::test]
async fn flag_off_refuses_members_and_reports_no_capability() {
    let app = TestApp::spawn().await;
    let member = signed_in_member(&app, "member").await;

    let capability = member.session.get(TENANT_CREATION_CAPABILITY_PATH).await;
    assert_eq!(capability.status, StatusCode::OK, "{}", capability.body);
    assert_eq!(
        capability.body,
        json!({ "can_create_tenant": false, "limit_reached_message": null })
    );

    let refused = member
        .session
        .post("/tenant", json!({ "name": "Side project" }))
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.body);
    assert_eq!(
        refused.body["error"],
        format!("Permission denied: {TENANT_CREATE_PERMISSION}")
    );
    assert_eq!(creation_records_for(&app, member.user.id).await, 0);
}

#[tokio::test]
async fn a_member_creates_their_first_tenant_and_owns_it() {
    let app = TestApp::spawn_with_self_serve_tenants().await;
    let member = signed_in_member(&app, "founder").await;
    assert_eq!(
        member
            .session
            .get(TENANT_CREATION_CAPABILITY_PATH)
            .await
            .body,
        creation_allowed()
    );

    let created = member
        .session
        .post("/tenant", json!({ "name": "  Acme Analytics  " }))
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    assert_eq!(created.body["name"], "Acme Analytics");
    let created_tenant_id =
        Uuid::parse_str(&string_field(&created.body, "id")).expect("the created tenant id");

    let memberships = member.session.get("/tenant").await;
    let memberships = memberships.body.as_array().expect("memberships");
    assert_eq!(memberships.len(), 1);
    assert_eq!(
        memberships[0]["tenant_id"],
        created_tenant_id.to_string().as_str()
    );
    assert_eq!(memberships[0]["role_name"], OWNER_ROLE);

    for system_role in SYSTEM_ROLES {
        assert_eq!(
            count_rows(
                &app,
                "role",
                &[
                    ("tenant_id", created_tenant_id.into()),
                    ("name", system_role.into()),
                ],
            )
            .await,
            1,
            "the new tenant has the {system_role} role"
        );
    }
    assert_eq!(
        count_rows(
            &app,
            SELF_SERVE_TENANT_CREATION_TABLE,
            &[
                ("creator_user_id", member.user.id.into()),
                ("created_tenant_id", created_tenant_id.into()),
            ],
        )
        .await,
        1
    );
}

#[tokio::test]
async fn the_third_tenant_is_refused_with_the_limit_message() {
    let app = TestApp::spawn_with_self_serve_tenants().await;
    let member = signed_in_member(&app, "builder").await;
    create_own_tenant(&member.session, "First").await;
    assert_eq!(
        member
            .session
            .get(TENANT_CREATION_CAPABILITY_PATH)
            .await
            .body,
        creation_allowed(),
        "one tenant leaves room for a second"
    );
    create_own_tenant(&member.session, "Second").await;

    let capability = member.session.get(TENANT_CREATION_CAPABILITY_PATH).await;
    assert_eq!(
        capability.body,
        json!({
            "can_create_tenant": false,
            "limit_reached_message": TWO_TENANT_LIMIT_MESSAGE,
        })
    );

    let tenants_before = count_rows(&app, "tenant", &[]).await;
    let refused = member
        .session
        .post("/tenant", json!({ "name": "Third" }))
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.body);
    assert_eq!(refused.body["error"], TWO_TENANT_LIMIT_MESSAGE);
    assert_eq!(
        refused.body["error_code"],
        TENANT_CREATION_LIMIT_REACHED_CODE
    );
    assert_eq!(count_rows(&app, "tenant", &[]).await, tenants_before);
    assert_eq!(creation_records_for(&app, member.user.id).await, 2);

    let other_member = signed_in_member(&app, "newcomer").await;
    assert_eq!(
        other_member
            .session
            .get(TENANT_CREATION_CAPABILITY_PATH)
            .await
            .body,
        creation_allowed(),
        "one person's limit does not affect another"
    );
}

#[tokio::test]
async fn memberships_in_tenants_someone_else_created_do_not_count() {
    let app = TestApp::spawn_with_self_serve_tenants().await;
    app.login_as_admin().await;
    let member = signed_in_member(&app, "guest").await;

    let invited_into = create_tenant(&app, "Invited into").await;
    add_member(&app, invited_into, member.user.id, MEMBER_ROLE).await;
    let administrator_created = create_own_tenant(&app.admin, "Administrator created").await;
    add_member(&app, administrator_created, member.user.id, OWNER_ROLE).await;

    assert_eq!(creation_records_for(&app, member.user.id).await, 0);
    assert_eq!(
        member
            .session
            .get(TENANT_CREATION_CAPABILITY_PATH)
            .await
            .body,
        creation_allowed()
    );
    create_own_tenant(&member.session, "Own first").await;
    create_own_tenant(&member.session, "Own second").await;
    assert_eq!(
        member
            .session
            .post("/tenant", json!({ "name": "Own third" }))
            .await
            .status,
        StatusCode::FORBIDDEN
    );
}

async fn assert_administrator_is_unlimited(app: &TestApp) {
    app.login_as_admin().await;
    assert_eq!(
        app.get(TENANT_CREATION_CAPABILITY_PATH).await.body,
        creation_allowed()
    );
    for tenant_name in ["One", "Two", "Three"] {
        create_own_tenant(&app.admin, tenant_name).await;
    }
    assert_eq!(
        app.get(TENANT_CREATION_CAPABILITY_PATH).await.body,
        creation_allowed()
    );
    assert_eq!(
        count_rows(app, SELF_SERVE_TENANT_CREATION_TABLE, &[]).await,
        0,
        "tenants a platform administrator creates are not recorded"
    );
}

#[tokio::test]
async fn platform_administrators_are_unlimited_with_the_flag_on() {
    assert_administrator_is_unlimited(&TestApp::spawn_with_self_serve_tenants().await).await;
}

#[tokio::test]
async fn platform_administrators_are_unlimited_with_the_flag_off() {
    assert_administrator_is_unlimited(&TestApp::spawn().await).await;
}

#[tokio::test]
async fn an_overridden_limit_is_honoured() {
    let app = TestApp::spawn_with_self_serve_tenant_limit(1).await;
    let member = signed_in_member(&app, "solo").await;
    create_own_tenant(&member.session, "Only").await;

    assert_eq!(
        member
            .session
            .get(TENANT_CREATION_CAPABILITY_PATH)
            .await
            .body,
        json!({
            "can_create_tenant": false,
            "limit_reached_message": ONE_TENANT_LIMIT_MESSAGE,
        })
    );
    let refused = member
        .session
        .post("/tenant", json!({ "name": "Second" }))
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.body);
    assert_eq!(refused.body["error"], ONE_TENANT_LIMIT_MESSAGE);
}

#[tokio::test]
async fn tenant_names_are_bounded() {
    let app = TestApp::spawn_with_self_serve_tenants().await;
    let member = signed_in_member(&app, "namer").await;

    let blank = member
        .session
        .post("/tenant", json!({ "name": "   " }))
        .await;
    assert_eq!(blank.status, StatusCode::BAD_REQUEST, "{}", blank.body);

    let too_long_name = "é".repeat(MAXIMUM_TENANT_NAME_CHARACTERS + 1);
    let too_long = member
        .session
        .post("/tenant", json!({ "name": too_long_name }))
        .await;
    assert_eq!(
        too_long.status,
        StatusCode::BAD_REQUEST,
        "{}",
        too_long.body
    );
    assert_eq!(too_long.body["error"], TENANT_NAME_TOO_LONG_MESSAGE);
    assert_eq!(
        creation_records_for(&app, member.user.id).await,
        0,
        "a refused name uses no slot"
    );

    let longest_name = "é".repeat(MAXIMUM_TENANT_NAME_CHARACTERS);
    let longest = member
        .session
        .post("/tenant", json!({ "name": longest_name }))
        .await;
    assert_eq!(longest.status, StatusCode::CREATED, "{}", longest.body);
    assert_eq!(longest.body["name"], longest_name);
}

#[tokio::test]
async fn a_self_served_tenant_cannot_be_reached_from_another() {
    let app = TestApp::spawn_with_self_serve_tenants().await;
    let first = signed_in_member(&app, "first").await;
    let second = signed_in_member(&app, "second").await;
    let first_tenant = create_own_tenant(&first.session, "First").await;
    let second_tenant = create_own_tenant(&second.session, "Second").await;

    for tenant_scoped_path in ["/canvas", "/member", "/connection"] {
        assert_eq!(
            first
                .session
                .in_tenant(first_tenant)
                .get(tenant_scoped_path)
                .await
                .status,
            StatusCode::OK,
            "{tenant_scoped_path} is readable by the tenant's owner"
        );
        assert_eq!(
            second
                .session
                .in_tenant(first_tenant)
                .get(tenant_scoped_path)
                .await
                .status,
            StatusCode::NOT_FOUND,
            "{tenant_scoped_path} must not be readable from another tenant"
        );
        assert_eq!(
            first
                .session
                .in_tenant(second_tenant)
                .get(tenant_scoped_path)
                .await
                .status,
            StatusCode::NOT_FOUND
        );
    }
}

#[tokio::test]
async fn the_capability_requires_a_session() {
    let app = TestApp::spawn_with_self_serve_tenants().await;
    let signed_out = app.session(None);
    assert_eq!(
        signed_out.get(TENANT_CREATION_CAPABILITY_PATH).await.status,
        StatusCode::UNAUTHORIZED
    );
}
