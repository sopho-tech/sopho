use crate::support::constants::{
    ACCOUNT_DEACTIVATED_CODE, ACCOUNT_DEACTIVATED_SIGN_IN_ERROR, ADMIN_ROLE,
    APPOINTEE_DEACTIVATED_CODE, CANNOT_DEACTIVATE_YOURSELF_CODE, EMAIL_OUTBOX_TABLE, HUMAN_ACTOR,
    INVITATION_TABLE, LAST_OWNER_MESSAGE, MEMBER_ROLE, NO_PASSWORD_CODE, OWNER_ROLE,
    PASSWORD_RESET_EMAIL_KIND, PLATFORM_MEMBER, RESET_EMAILS_PER_HOUR, REVOKED_INVITATION_STATUS,
    SIMULTANEOUS_RESET_REQUESTS, SINGLE_SIGN_ON_PROVIDER, TEST_USER_PASSWORD,
    USER_STATUS_DEACTIVATED,
};
use crate::support::database::{count_rows, insert_row, now, select_rows, update_rows};
use crate::support::identity_provider::sign_in_with_provider;
use crate::support::tenants::{TestUser, create_tenant, create_user, join_tenant, sign_in};
use crate::support::test_app::JsonResponse;
use crate::support::{TestApp, UserSession};
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

async fn signed_in_admin(app: &TestApp) -> Uuid {
    app.login_as_admin().await;
    app.get("/auth/me").await.body["id"]
        .as_str()
        .expect("the admin id")
        .parse()
        .expect("a UUID")
}

async fn deactivate(session: &UserSession, user_id: Uuid) -> JsonResponse {
    session
        .post(&format!("/platform/user/{user_id}/deactivate"), json!({}))
        .await
}

async fn appoint_owner(session: &UserSession, tenant_id: Uuid, user_id: Uuid) -> JsonResponse {
    session
        .put(
            &format!("/platform/tenant/{tenant_id}/owner/{user_id}"),
            json!({}),
        )
        .await
}

async fn user_capabilities(app: &TestApp, email: &str, user_id: Uuid) -> Value {
    let users = app.get(&format!("/platform/user?search={email}")).await;
    assert_eq!(users.status, StatusCode::OK, "{}", users.body);
    users.body["users"]
        .as_array()
        .expect("users")
        .iter()
        .find(|row| row["id"] == user_id.to_string())
        .expect("the user's row")["capabilities"]
        .clone()
}

fn assert_error_code(response: &JsonResponse, status: StatusCode, error_code: &str) {
    assert_eq!(response.status, status, "{}", response.body);
    assert_eq!(response.body["error_code"], error_code, "{}", response.body);
}

async fn membership_role(app: &TestApp, tenant_id: Uuid, user: &TestUser) -> Option<String> {
    let rows = select_rows(
        app,
        "tenant_membership",
        &["role_id"],
        &[("tenant_id", tenant_id.into()), ("user_id", user.id.into())],
    )
    .await;
    let role_id: Uuid = rows
        .first()?
        .try_get("", "role_id")
        .expect("read the role id");
    select_rows(app, "role", &["name"], &[("id", role_id.into())]).await[0]
        .try_get("", "name")
        .ok()
}

#[tokio::test]
async fn the_users_page_needs_user_manage() {
    let app = TestApp::spawn().await;
    let member = create_user(&app, "member", PLATFORM_MEMBER).await;
    let member_session = sign_in(&app, &member).await;

    assert_eq!(
        member_session.get("/platform/user").await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        deactivate(&member_session, member.id).await.status,
        StatusCode::FORBIDDEN
    );

    signed_in_admin(&app).await;
    assert_eq!(app.get("/platform/user").await.status, StatusCode::OK);
}

#[tokio::test]
async fn users_are_searched_by_name_or_email_and_paged() {
    let app = TestApp::spawn().await;
    signed_in_admin(&app).await;
    for _ in 0..51 {
        create_user(&app, "pagecheck", PLATFORM_MEMBER).await;
    }
    let lonely = create_user(&app, "lonely", PLATFORM_MEMBER).await;

    let first_page = app.get("/platform/user?search=PAGECHECK%20USER").await;
    assert_eq!(first_page.status, StatusCode::OK, "{}", first_page.body);
    assert_eq!(first_page.body["total"], 51);
    assert_eq!(first_page.body["page"], 1);
    assert_eq!(first_page.body["page_size"], 50);
    assert_eq!(first_page.body["total_pages"], 2);
    assert_eq!(
        first_page.body["users"].as_array().expect("users").len(),
        50
    );

    let second_page = app.get("/platform/user?search=pagecheck&page=2").await;
    assert_eq!(
        second_page.body["users"].as_array().expect("users").len(),
        1
    );

    let by_email = app
        .get(&format!("/platform/user?search={}", &lonely.email[..20]))
        .await;
    let found = by_email.body["users"].as_array().expect("users");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0]["email"], lonely.email);
    assert_eq!(found[0]["status"], "ACTIVE");
    assert_eq!(found[0]["has_password"], true);
    assert_eq!(
        found[0]["capabilities"]["can_send_password_reset"], false,
        "email is off"
    );

    assert_eq!(
        app.get("/platform/user?page=0").await.status,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn deactivating_ends_sessions_and_blocks_both_kinds_of_sign_in() {
    let app = TestApp::spawn_with_single_sign_on().await;
    signed_in_admin(&app).await;
    let user = create_user(&app, "leaver", PLATFORM_MEMBER).await;
    insert_row(
        &app,
        "identity",
        &[
            ("id", Uuid::new_v4().into()),
            ("user_id", user.id.into()),
            ("provider_name", SINGLE_SIGN_ON_PROVIDER.into()),
            ("subject", "leaver-subject".into()),
            ("email", user.email.clone().into()),
            ("email_verified", true.into()),
            ("created_at", now().into()),
            ("updated_at", now().into()),
        ],
    )
    .await;
    let user_session = sign_in(&app, &user).await;

    let deactivated = deactivate(&app, user.id).await;
    assert_eq!(deactivated.status, StatusCode::OK, "{}", deactivated.body);

    assert_eq!(
        user_session.get("/auth/me").await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_error_code(
        &app.session(None)
            .sign_in(&user.email, TEST_USER_PASSWORD)
            .await,
        StatusCode::FORBIDDEN,
        ACCOUNT_DEACTIVATED_CODE,
    );
    let single_sign_on = sign_in_with_provider(
        &app.session(None),
        SINGLE_SIGN_ON_PROVIDER,
        "leaver-subject",
        json!({ "email": user.email, "email_verified": true, "name": user.full_name }),
    )
    .await;
    assert_eq!(
        single_sign_on.error_code(),
        Some(ACCOUNT_DEACTIVATED_SIGN_IN_ERROR)
    );
}

#[tokio::test]
async fn each_user_row_says_which_actions_apply() {
    let app = TestApp::spawn_with_email().await;
    let admin_id = signed_in_admin(&app).await;
    let admin_email = app.get("/auth/me").await.body["email"]
        .as_str()
        .expect("the admin email")
        .to_string();
    let active = create_user(&app, "rowactive", PLATFORM_MEMBER).await;
    let leaver = create_user(&app, "rowleaver", PLATFORM_MEMBER).await;
    deactivate(&app, leaver.id).await;

    assert_eq!(
        user_capabilities(&app, &admin_email, admin_id).await,
        json!({
            "can_deactivate": false,
            "can_reactivate": false,
            "can_send_password_reset": true,
            "can_appoint_owner": true,
        })
    );
    assert_eq!(
        user_capabilities(&app, &active.email, active.id).await,
        json!({
            "can_deactivate": true,
            "can_reactivate": false,
            "can_send_password_reset": true,
            "can_appoint_owner": true,
        })
    );
    assert_eq!(
        user_capabilities(&app, &leaver.email, leaver.id).await,
        json!({
            "can_deactivate": false,
            "can_reactivate": true,
            "can_send_password_reset": false,
            "can_appoint_owner": false,
        })
    );
}

#[tokio::test]
async fn reactivating_restores_access() {
    let app = TestApp::spawn().await;
    signed_in_admin(&app).await;
    let tenant_id = create_tenant(&app, "acme").await;
    let member = join_tenant(&app, tenant_id, "returning", MEMBER_ROLE).await;
    deactivate(&app, member.user.id).await;

    let reactivated = app
        .post(
            &format!("/platform/user/{}/reactivate", member.user.id),
            json!({}),
        )
        .await;
    assert_eq!(
        reactivated.status,
        StatusCode::NO_CONTENT,
        "{}",
        reactivated.body
    );

    let session = sign_in(&app, &member.user).await;
    let memberships = session.get("/tenant").await;
    assert_eq!(memberships.body[0]["tenant_id"], tenant_id.to_string());
}

#[tokio::test]
async fn you_cannot_deactivate_yourself() {
    let app = TestApp::spawn().await;
    let admin_id = signed_in_admin(&app).await;

    assert_error_code(
        &deactivate(&app, admin_id).await,
        StatusCode::CONFLICT,
        CANNOT_DEACTIVATE_YOURSELF_CODE,
    );
}

#[tokio::test]
async fn deactivating_revokes_the_invitations_the_account_sent() {
    let app = TestApp::spawn_with_single_sign_on().await;
    signed_in_admin(&app).await;
    let tenant_id = create_tenant(&app, "acme").await;
    let tenant_admin = join_tenant(&app, tenant_id, "inviter", ADMIN_ROLE).await;
    let invited = tenant_admin
        .session
        .post(
            &format!("/tenant/{tenant_id}/invitation"),
            json!({ "email": "accomplice@example.test", "role_name": ADMIN_ROLE }),
        )
        .await;
    assert_eq!(invited.status, StatusCode::CREATED, "{}", invited.body);

    deactivate(&app, tenant_admin.user.id).await;

    let status: String = select_rows(
        &app,
        INVITATION_TABLE,
        &["status"],
        &[("email", "accomplice@example.test".into())],
    )
    .await[0]
        .try_get("", "status")
        .expect("read the invitation status");
    assert_eq!(status, REVOKED_INVITATION_STATUS);
}

#[tokio::test]
async fn deactivating_the_only_owner_reports_the_tenant() {
    let app = TestApp::spawn().await;
    signed_in_admin(&app).await;
    let tenant_id = create_tenant(&app, "orphaned").await;
    let owner = join_tenant(&app, tenant_id, "sole-owner", OWNER_ROLE).await;

    let deactivated = deactivate(&app, owner.user.id).await;

    assert_eq!(deactivated.status, StatusCode::OK, "{}", deactivated.body);
    assert_eq!(
        deactivated.body["tenants_without_active_owner"],
        json!([{ "tenant_id": tenant_id.to_string(), "tenant_name": "orphaned" }])
    );
    let tenants = app.get("/platform/tenant").await;
    let orphaned = tenants
        .body
        .as_array()
        .expect("tenants")
        .iter()
        .find(|listed| listed["tenant_id"] == tenant_id.to_string())
        .expect("the orphaned tenant")
        .clone();
    assert_eq!(orphaned["active_owner_count"], 0);
}

#[tokio::test]
async fn the_last_active_owner_cannot_step_down_while_other_owners_are_deactivated() {
    let app = TestApp::spawn().await;
    signed_in_admin(&app).await;
    let tenant_id = create_tenant(&app, "acme").await;
    let departing_owner = join_tenant(&app, tenant_id, "departing", OWNER_ROLE).await;
    let remaining_owner = join_tenant(&app, tenant_id, "remaining", OWNER_ROLE).await;
    deactivate(&app, departing_owner.user.id).await;

    let stepping_down = remaining_owner
        .session
        .put(
            &format!("/member/{}", remaining_owner.user.id),
            json!({ "role_name": ADMIN_ROLE }),
        )
        .await;

    assert_eq!(
        stepping_down.status,
        StatusCode::CONFLICT,
        "{}",
        stepping_down.body
    );
    assert_eq!(stepping_down.body["error"], LAST_OWNER_MESSAGE);
}

#[tokio::test]
async fn the_remaining_owner_can_demote_and_remove_a_deactivated_owner() {
    let app = TestApp::spawn().await;
    signed_in_admin(&app).await;
    let tenant_id = create_tenant(&app, "acme").await;
    let departing_owner = join_tenant(&app, tenant_id, "departing", OWNER_ROLE).await;
    let remaining_owner = join_tenant(&app, tenant_id, "remaining", OWNER_ROLE).await;
    deactivate(&app, departing_owner.user.id).await;

    let members = remaining_owner.session.get("/member").await;
    assert_eq!(members.status, StatusCode::OK, "{}", members.body);
    let departing_row = members
        .body
        .as_array()
        .expect("members")
        .iter()
        .find(|member| member["user_id"] == departing_owner.user.id.to_string())
        .expect("the departing owner's row")
        .clone();
    assert_eq!(
        departing_row["capabilities"],
        json!({ "can_change_role": true, "can_remove": true })
    );

    let demoted = remaining_owner
        .session
        .put(
            &format!("/member/{}", departing_owner.user.id),
            json!({ "role_name": ADMIN_ROLE }),
        )
        .await;
    assert_eq!(demoted.status, StatusCode::OK, "{}", demoted.body);

    let removed = remaining_owner
        .session
        .delete(&format!("/member/{}", departing_owner.user.id))
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT, "{}", removed.body);
}

#[tokio::test]
async fn appointing_an_owner_promotes_a_member_or_adds_a_non_member() {
    let app = TestApp::spawn().await;
    let admin_id = signed_in_admin(&app).await;
    let tenant_id = create_tenant(&app, "acme").await;
    let member = join_tenant(&app, tenant_id, "promoted", MEMBER_ROLE).await;
    let outsider = create_user(&app, "outsider", PLATFORM_MEMBER).await;

    assert_eq!(
        appoint_owner(&app, tenant_id, member.user.id).await.status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        appoint_owner(&app, tenant_id, outsider.id).await.status,
        StatusCode::NO_CONTENT
    );

    assert_eq!(
        membership_role(&app, tenant_id, &member.user)
            .await
            .as_deref(),
        Some(OWNER_ROLE)
    );
    assert_eq!(
        membership_role(&app, tenant_id, &outsider).await.as_deref(),
        Some(OWNER_ROLE)
    );
    let attribution = select_rows(
        &app,
        "tenant_membership",
        &["modified_by_user_id", "modified_by_actor"],
        &[
            ("tenant_id", tenant_id.into()),
            ("user_id", outsider.id.into()),
        ],
    )
    .await;
    assert_eq!(
        attribution[0]
            .try_get::<Option<Uuid>>("", "modified_by_user_id")
            .unwrap(),
        Some(admin_id)
    );
    assert_eq!(
        attribution[0]
            .try_get::<String>("", "modified_by_actor")
            .unwrap(),
        HUMAN_ACTOR
    );
}

#[tokio::test]
async fn appointing_refuses_a_deactivated_account() {
    let app = TestApp::spawn().await;
    signed_in_admin(&app).await;
    let tenant_id = create_tenant(&app, "acme").await;
    let deactivated_user = create_user(&app, "inactive", PLATFORM_MEMBER).await;
    update_rows(
        &app,
        "user",
        &[("status", USER_STATUS_DEACTIVATED.into())],
        &[("id", deactivated_user.id.into())],
    )
    .await;

    assert_error_code(
        &appoint_owner(&app, tenant_id, deactivated_user.id).await,
        StatusCode::CONFLICT,
        APPOINTEE_DEACTIVATED_CODE,
    );
    assert_eq!(
        membership_role(&app, tenant_id, &deactivated_user).await,
        None
    );
}

#[tokio::test]
async fn appointing_needs_tenant_appoint_owner() {
    let app = TestApp::spawn().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let tenant_owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let outsider = create_user(&app, "outsider", PLATFORM_MEMBER).await;

    assert_eq!(
        appoint_owner(&tenant_owner.session, tenant_id, outsider.id)
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        tenant_owner.session.get("/platform/tenant").await.status,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn members_show_deactivated_accounts() {
    let app = TestApp::spawn().await;
    signed_in_admin(&app).await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let leaver = join_tenant(&app, tenant_id, "leaver", MEMBER_ROLE).await;
    deactivate(&app, leaver.user.id).await;

    let members = owner.session.get("/member").await;
    let listed_leaver: Value = members
        .body
        .as_array()
        .expect("members")
        .iter()
        .find(|member| member["user_id"] == leaver.user.id.to_string())
        .expect("the deactivated member is still listed")
        .clone();
    assert_eq!(listed_leaver["account_status"], USER_STATUS_DEACTIVATED);
}

#[tokio::test]
async fn simultaneous_password_resets_stay_within_the_hourly_limit() {
    let app = TestApp::spawn_with_email().await;
    signed_in_admin(&app).await;
    let user = create_user(&app, "burst", PLATFORM_MEMBER).await;
    let reset_url = app.api_url(&format!("/platform/user/{}/password_reset", user.id));

    let mut reset_requests = tokio::task::JoinSet::new();
    for _ in 0..SIMULTANEOUS_RESET_REQUESTS {
        let client = app.client.clone();
        let reset_url = reset_url.clone();
        reset_requests.spawn(async move {
            client
                .post(reset_url)
                .json(&json!({}))
                .send()
                .await
                .expect("request a password reset")
                .status()
        });
    }
    let statuses = reset_requests.join_all().await;

    let sent = statuses
        .iter()
        .filter(|status| **status == StatusCode::NO_CONTENT)
        .count();
    let refused = statuses
        .iter()
        .filter(|status| **status == StatusCode::TOO_MANY_REQUESTS)
        .count();
    assert_eq!(sent, RESET_EMAILS_PER_HOUR, "{statuses:?}");
    assert_eq!(
        refused,
        SIMULTANEOUS_RESET_REQUESTS - RESET_EMAILS_PER_HOUR,
        "{statuses:?}"
    );
    assert_eq!(
        count_rows(
            &app,
            EMAIL_OUTBOX_TABLE,
            &[("recipient", user.email.clone().into())]
        )
        .await,
        i64::try_from(RESET_EMAILS_PER_HOUR).expect("a small count")
    );
}

#[tokio::test]
async fn sending_a_password_reset_from_the_users_page_queues_the_email() {
    let app = TestApp::spawn_with_email().await;
    let admin_id = signed_in_admin(&app).await;
    let user = create_user(&app, "forgetful", PLATFORM_MEMBER).await;

    let sent = app
        .post(
            &format!("/platform/user/{}/password_reset", user.id),
            json!({}),
        )
        .await;
    assert_eq!(sent.status, StatusCode::NO_CONTENT, "{}", sent.body);
    let queued = select_rows(
        &app,
        EMAIL_OUTBOX_TABLE,
        &["requested_by_user_id"],
        &[
            ("recipient", user.email.clone().into()),
            ("kind", PASSWORD_RESET_EMAIL_KIND.into()),
        ],
    )
    .await;
    assert_eq!(queued.len(), 1);
    assert_eq!(
        queued[0]
            .try_get::<Option<Uuid>>("", "requested_by_user_id")
            .unwrap(),
        Some(admin_id)
    );

    let without_password = create_user(&app, "sso-only", PLATFORM_MEMBER).await;
    update_rows(
        &app,
        "user",
        &[("password_hash", sea_orm::Value::String(None))],
        &[("id", without_password.id.into())],
    )
    .await;
    assert_error_code(
        &app.post(
            &format!("/platform/user/{}/password_reset", without_password.id),
            json!({}),
        )
        .await,
        StatusCode::CONFLICT,
        NO_PASSWORD_CODE,
    );
    assert_eq!(
        count_rows(
            &app,
            EMAIL_OUTBOX_TABLE,
            &[("recipient", without_password.email.clone().into())]
        )
        .await,
        0
    );
}
