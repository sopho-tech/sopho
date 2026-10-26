use crate::support::constants::{
    ACCEPTED_INVITATION_STATUS, ACCOUNT_EXISTS_CODE, FULL_NAME_REQUIRED_CODE,
    INVITATION_FOR_ANOTHER_ACCOUNT_CODE, INVITATION_LINK_INVALID_CODE, INVITATION_LINK_PATH,
    INVITATION_TABLE, MEMBER_ROLE, NEW_ACCOUNT_PASSWORD, OWNER_ROLE,
    PASSWORD_CONTAINS_SERVICE_NAME_CODE, PASSWORD_TOO_SHORT_CODE, PENDING_INVITATION_STATUS,
    PLATFORM_MEMBER, USER_STATUS_ACTIVE,
};
use crate::support::database::{count_rows, now, select_rows, update_rows};
use crate::support::mail_server::{link_in, shared_mail_server};
use crate::support::tenants::{TenantMember, create_tenant, create_user, join_tenant, sign_in};
use crate::support::test_app::JsonResponse;
use crate::support::{TestApp, UserSession};
use chrono::{DateTime, Duration, FixedOffset};
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

fn unique_recipient(label: &str) -> String {
    format!("{label}-{}@example.test", Uuid::new_v4().simple())
}

async fn tenant_with_owner(app: &TestApp) -> (Uuid, TenantMember) {
    let tenant_id = create_tenant(app, "acme").await;
    let owner = join_tenant(app, tenant_id, "owner", OWNER_ROLE).await;
    (tenant_id, owner)
}

async fn invite(owner: &TenantMember, tenant_id: Uuid, email: &str) -> Uuid {
    let response = owner
        .session
        .post(
            &format!("/tenant/{tenant_id}/invitation"),
            json!({ "email": email, "role_name": MEMBER_ROLE }),
        )
        .await;
    assert_eq!(response.status, StatusCode::CREATED, "{}", response.body);
    response.body["id"]
        .as_str()
        .expect("an invitation id")
        .parse()
        .expect("the invitation id is a UUID")
}

async fn emailed_link_token(app: &TestApp, recipient: &str) -> String {
    app.deliver_due_emails().await;
    let received = shared_mail_server().await.emails_to(recipient).await;
    let latest = received.last().expect("an invitation email");
    let link = link_in(latest, &format!("{}{INVITATION_LINK_PATH}", app.base_url()));
    link.split_once("#token=")
        .map(|(_, token)| token.to_string())
        .expect("the link carries a token")
}

async fn lookup(session: &UserSession, token: &str) -> JsonResponse {
    session
        .post("/invitation_link/lookup", json!({ "token": token }))
        .await
}

async fn sign_up(
    session: &UserSession,
    token: &str,
    full_name: &str,
    password: &str,
) -> JsonResponse {
    session
        .post(
            "/invitation_link/sign_up",
            json!({ "token": token, "full_name": full_name, "password": password }),
        )
        .await
}

async fn accept_link(session: &UserSession, token: &str) -> JsonResponse {
    session
        .post("/invitation_link/accept", json!({ "token": token }))
        .await
}

fn assert_link_error(response: &JsonResponse, status: StatusCode, error_code: &str) {
    assert_eq!(response.status, status, "{}", response.body);
    assert_eq!(response.body["error_code"], error_code, "{}", response.body);
}

async fn invitation_status_and_token(
    app: &TestApp,
    invitation_id: Uuid,
) -> (String, Option<String>) {
    let rows = select_rows(
        app,
        INVITATION_TABLE,
        &["status", "token_hash"],
        &[("id", invitation_id.into())],
    )
    .await;
    (
        rows[0].try_get("", "status").expect("read the status"),
        rows[0]
            .try_get("", "token_hash")
            .expect("read the token hash"),
    )
}

async fn email_verified_at(app: &TestApp, email: &str) -> Option<DateTime<FixedOffset>> {
    select_rows(
        app,
        "user",
        &["email_verified_at"],
        &[("email", email.into())],
    )
    .await[0]
        .try_get("", "email_verified_at")
        .expect("read the email verification time")
}

async fn tenant_ids(session: &UserSession) -> Vec<String> {
    let memberships = session.get("/tenant").await;
    assert_eq!(memberships.status, StatusCode::OK, "{}", memberships.body);
    memberships
        .body
        .as_array()
        .expect("memberships")
        .iter()
        .map(|membership| {
            membership["tenant_id"]
                .as_str()
                .expect("a tenant id")
                .to_string()
        })
        .collect()
}

#[tokio::test]
async fn signing_up_from_the_link_creates_a_verified_account_in_the_tenant() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("newcomer");
    let invitation_id = invite(&owner, tenant_id, &recipient).await;
    let token = emailed_link_token(&app, &recipient).await;
    let browser = app.session(None);

    let found = lookup(&browser, &token).await;
    assert_eq!(found.status, StatusCode::OK, "{}", found.body);
    assert_eq!(found.body["next_step"], "SIGN_UP");
    assert_eq!(found.body["tenant_name"], "acme");
    assert_eq!(found.body["role_name"], MEMBER_ROLE);
    assert_eq!(found.body["email"], recipient);
    assert_eq!(found.body["invited_by_name"], owner.user.full_name);

    let signed_up = sign_up(&browser, &token, " New Person ", NEW_ACCOUNT_PASSWORD).await;
    assert_eq!(signed_up.status, StatusCode::CREATED, "{}", signed_up.body);
    assert_eq!(signed_up.body["tenant_id"], tenant_id.to_string());
    assert_eq!(tenant_ids(&browser).await, vec![tenant_id.to_string()]);

    let users = select_rows(
        &app,
        "user",
        &["full_name", "platform_role", "status", "password_hash"],
        &[("email", recipient.as_str().into())],
    )
    .await;
    assert_eq!(users.len(), 1);
    assert_eq!(
        users[0].try_get::<String>("", "full_name").unwrap(),
        "New Person"
    );
    assert_eq!(
        users[0].try_get::<String>("", "platform_role").unwrap(),
        PLATFORM_MEMBER
    );
    assert_eq!(
        users[0].try_get::<String>("", "status").unwrap(),
        USER_STATUS_ACTIVE
    );
    assert!(
        users[0]
            .try_get::<Option<String>>("", "password_hash")
            .unwrap()
            .is_some()
    );
    assert!(email_verified_at(&app, &recipient).await.is_some());
    assert_eq!(
        invitation_status_and_token(&app, invitation_id).await,
        (ACCEPTED_INVITATION_STATUS.to_string(), None)
    );

    let later_visit = app.session(None);
    let signed_in = later_visit.sign_in(&recipient, NEW_ACCOUNT_PASSWORD).await;
    assert_eq!(signed_in.status, StatusCode::OK, "{}", signed_in.body);
}

#[tokio::test]
async fn a_link_signs_up_only_once() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("once");
    invite(&owner, tenant_id, &recipient).await;
    let token = emailed_link_token(&app, &recipient).await;
    let first_browser = app.session(None);
    assert_eq!(
        sign_up(&first_browser, &token, "First Person", NEW_ACCOUNT_PASSWORD)
            .await
            .status,
        StatusCode::CREATED
    );

    let second_browser = app.session(None);
    assert_link_error(
        &sign_up(
            &second_browser,
            &token,
            "Second Person",
            NEW_ACCOUNT_PASSWORD,
        )
        .await,
        StatusCode::NOT_FOUND,
        INVITATION_LINK_INVALID_CODE,
    );
    assert_link_error(
        &lookup(&second_browser, &token).await,
        StatusCode::NOT_FOUND,
        INVITATION_LINK_INVALID_CODE,
    );
    assert_eq!(
        count_rows(&app, "user", &[("email", recipient.as_str().into())]).await,
        1
    );
}

#[tokio::test]
async fn an_expired_or_revoked_link_is_no_longer_valid() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let browser = app.session(None);

    let expiring_recipient = unique_recipient("expiring");
    let expiring_invitation = invite(&owner, tenant_id, &expiring_recipient).await;
    let expiring_token = emailed_link_token(&app, &expiring_recipient).await;
    update_rows(
        &app,
        INVITATION_TABLE,
        &[("expires_at", (now() - Duration::days(1)).into())],
        &[("id", expiring_invitation.into())],
    )
    .await;
    assert_link_error(
        &lookup(&browser, &expiring_token).await,
        StatusCode::NOT_FOUND,
        INVITATION_LINK_INVALID_CODE,
    );

    let revoked_recipient = unique_recipient("revoked");
    let revoked_invitation = invite(&owner, tenant_id, &revoked_recipient).await;
    let revoked_token = emailed_link_token(&app, &revoked_recipient).await;
    let revoked = owner
        .session
        .delete(&format!(
            "/tenant/{tenant_id}/invitation/{revoked_invitation}"
        ))
        .await;
    assert_eq!(revoked.status, StatusCode::NO_CONTENT, "{}", revoked.body);
    assert_link_error(
        &sign_up(&browser, &revoked_token, "Too Late", NEW_ACCOUNT_PASSWORD).await,
        StatusCode::NOT_FOUND,
        INVITATION_LINK_INVALID_CODE,
    );
    assert_eq!(
        count_rows(
            &app,
            "user",
            &[("email", revoked_recipient.as_str().into())]
        )
        .await,
        0
    );
}

#[tokio::test]
async fn joining_from_the_link_needs_the_invited_email() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("intended");
    let invitation_id = invite(&owner, tenant_id, &recipient).await;
    let token = emailed_link_token(&app, &recipient).await;
    let bystander = create_user(&app, "bystander", PLATFORM_MEMBER).await;
    let bystander_session = sign_in(&app, &bystander).await;

    assert_link_error(
        &accept_link(&bystander_session, &token).await,
        StatusCode::FORBIDDEN,
        INVITATION_FOR_ANOTHER_ACCOUNT_CODE,
    );
    assert!(tenant_ids(&bystander_session).await.is_empty());
    assert_eq!(
        invitation_status_and_token(&app, invitation_id).await.0,
        PENDING_INVITATION_STATUS
    );
}

#[tokio::test]
async fn opening_a_link_signed_in_as_someone_else_says_who_it_is_for() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("intended");
    invite(&owner, tenant_id, &recipient).await;
    let token = emailed_link_token(&app, &recipient).await;
    let bystander = create_user(&app, "bystander", PLATFORM_MEMBER).await;
    let bystander_session = sign_in(&app, &bystander).await;

    let found = lookup(&bystander_session, &token).await;
    assert_eq!(found.status, StatusCode::OK, "{}", found.body);
    assert_eq!(found.body["next_step"], "WRONG_ACCOUNT");
    assert_eq!(found.body["email"], recipient);
}

#[tokio::test]
async fn a_link_for_an_existing_account_asks_to_sign_in() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let existing = create_user(&app, "existing", PLATFORM_MEMBER).await;
    invite(&owner, tenant_id, &existing.email).await;
    let token = emailed_link_token(&app, &existing.email).await;
    assert!(email_verified_at(&app, &existing.email).await.is_none());

    let browser = app.session(None);
    assert_eq!(lookup(&browser, &token).await.body["next_step"], "SIGN_IN");
    assert_link_error(
        &sign_up(&browser, &token, "Impostor", NEW_ACCOUNT_PASSWORD).await,
        StatusCode::CONFLICT,
        ACCOUNT_EXISTS_CODE,
    );

    let existing_session = sign_in(&app, &existing).await;
    assert_eq!(
        lookup(&existing_session, &token).await.body["next_step"],
        "JOIN"
    );
    let joined = accept_link(&existing_session, &token).await;
    assert_eq!(joined.status, StatusCode::OK, "{}", joined.body);
    assert_eq!(
        tenant_ids(&existing_session).await,
        vec![tenant_id.to_string()]
    );
    assert!(email_verified_at(&app, &existing.email).await.is_some());
}

#[tokio::test]
async fn an_invitation_to_a_mixed_case_email_signs_up_with_the_normalized_email() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let normalized_recipient = unique_recipient("mixed.case");
    let typed_recipient = format!("  {}  ", normalized_recipient.to_uppercase());
    invite(&owner, tenant_id, &typed_recipient).await;
    let token = emailed_link_token(&app, &normalized_recipient).await;

    let browser = app.session(None);
    let found = lookup(&browser, &token).await;
    assert_eq!(found.body["email"], normalized_recipient);
    assert_eq!(
        sign_up(&browser, &token, "Mixed Case", NEW_ACCOUNT_PASSWORD)
            .await
            .status,
        StatusCode::CREATED
    );
    assert_eq!(
        count_rows(
            &app,
            "user",
            &[("email", normalized_recipient.as_str().into())]
        )
        .await,
        1
    );
    let signed_in = app
        .session(None)
        .sign_in(&normalized_recipient.to_uppercase(), NEW_ACCOUNT_PASSWORD)
        .await;
    assert_eq!(signed_in.status, StatusCode::OK, "{}", signed_in.body);
}

#[tokio::test]
async fn signing_up_enforces_the_password_rules() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("rules");
    let invitation_id = invite(&owner, tenant_id, &recipient).await;
    let token = emailed_link_token(&app, &recipient).await;
    let browser = app.session(None);

    let too_short = sign_up(&browser, &token, "Rule Breaker", "too short").await;
    assert_link_error(&too_short, StatusCode::BAD_REQUEST, PASSWORD_TOO_SHORT_CODE);
    assert_eq!(too_short.body["message"], "Use at least 15 characters");
    assert_link_error(
        &sign_up(
            &browser,
            &token,
            "Rule Breaker",
            "my sopho password is long",
        )
        .await,
        StatusCode::BAD_REQUEST,
        PASSWORD_CONTAINS_SERVICE_NAME_CODE,
    );
    assert_link_error(
        &sign_up(&browser, &token, "   ", NEW_ACCOUNT_PASSWORD).await,
        StatusCode::BAD_REQUEST,
        FULL_NAME_REQUIRED_CODE,
    );

    assert_eq!(
        count_rows(&app, "user", &[("email", recipient.as_str().into())]).await,
        0
    );
    assert_eq!(
        invitation_status_and_token(&app, invitation_id).await.0,
        PENDING_INVITATION_STATUS
    );
}

#[tokio::test]
async fn accepting_from_onboarding_clears_the_link() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let existing = create_user(&app, "existing", PLATFORM_MEMBER).await;
    let invitation_id = invite(&owner, tenant_id, &existing.email).await;
    let token = emailed_link_token(&app, &existing.email).await;
    let existing_session = sign_in(&app, &existing).await;

    let accepted = existing_session
        .post(&format!("/invitation/{invitation_id}/accept"), json!({}))
        .await;
    assert_eq!(accepted.status, StatusCode::OK, "{}", accepted.body);

    assert_eq!(
        invitation_status_and_token(&app, invitation_id).await.1,
        None
    );
    assert_link_error(
        &lookup(&existing_session, &token).await,
        StatusCode::NOT_FOUND,
        INVITATION_LINK_INVALID_CODE,
    );
    assert!(
        email_verified_at(&app, &existing.email).await.is_none(),
        "accepting from the list proves nothing about the email"
    );
}
