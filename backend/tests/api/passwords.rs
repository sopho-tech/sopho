use crate::support::constants::{
    ACCOUNT_DEACTIVATED_CODE, EMAIL_OUTBOX_TABLE, INVALID_RESET_LINK_CODE, NEW_ACCOUNT_PASSWORD,
    NO_PASSWORD_EMAIL_KIND, PASSWORD_CHANGED_EMAIL_KIND, PASSWORD_RESET_EMAIL_KIND,
    PASSWORD_TOO_SHORT_CODE, PLATFORM_MEMBER, QUEUED_EMAIL_POLL_MILLISECONDS,
    QUEUED_EMAIL_WAIT_SECONDS, RESET_LINK_PATH, RESET_REQUESTED_MESSAGE, SINGLE_SIGN_ON_PROVIDER,
    SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME, TEST_USER_PASSWORD, TOO_MANY_SIGN_IN_ATTEMPTS_CODE,
    USER_STATUS_DEACTIVATED, WRONG_CURRENT_PASSWORD_CODE,
};
use crate::support::database::{count_rows, insert_row, now, select_rows, update_rows};
use crate::support::mail_server::{ReceivedEmail, link_in, shared_mail_server};
use crate::support::tenants::{TestUser, create_user, sign_in};
use crate::support::test_app::JsonResponse;
use crate::support::{TestApp, UserSession};
use chrono::{DateTime, Duration, FixedOffset};
use reqwest::StatusCode;
use serde_json::{Value, json};
use std::time::Instant;
use uuid::Uuid;

const CHANGED_PASSWORD: &str = "a brand new passphrase for tests";

async fn request_reset(session: &UserSession, email: &str) -> JsonResponse {
    session
        .post("/auth/password_reset_request", json!({ "email": email }))
        .await
}

async fn reset(session: &UserSession, token: &str, new_password: &str) -> JsonResponse {
    session
        .post(
            "/auth/password_reset",
            json!({ "token": token, "new_password": new_password }),
        )
        .await
}

async fn change(session: &UserSession, current_password: &str, new_password: &str) -> JsonResponse {
    session
        .put(
            "/auth/password",
            json!({ "current_password": current_password, "new_password": new_password }),
        )
        .await
}

async fn queued_count(app: &TestApp, recipient: &str, kind: &str) -> i64 {
    count_rows(
        app,
        EMAIL_OUTBOX_TABLE,
        &[("recipient", recipient.into()), ("kind", kind.into())],
    )
    .await
}

async fn wait_for_queued_emails(app: &TestApp, recipient: &str, kind: &str, expected: i64) {
    let deadline = Instant::now() + std::time::Duration::from_secs(QUEUED_EMAIL_WAIT_SECONDS);
    while queued_count(app, recipient, kind).await < expected {
        assert!(
            Instant::now() < deadline,
            "no {kind} email was queued for {recipient}"
        );
        tokio::time::sleep(std::time::Duration::from_millis(
            QUEUED_EMAIL_POLL_MILLISECONDS,
        ))
        .await;
    }
}

async fn emailed_reset_token(app: &TestApp, user: &TestUser) -> String {
    wait_for_queued_emails(app, &user.email, PASSWORD_RESET_EMAIL_KIND, 1).await;
    app.deliver_due_emails().await;
    let received = shared_mail_server().await.emails_to(&user.email).await;
    let reset_email = latest_with_subject(&received, "Reset your Sopho password");
    link_in(reset_email, &format!("{}{RESET_LINK_PATH}", app.base_url()))
        .split_once("#token=")
        .map(|(_, token)| token.to_string())
        .expect("the reset link carries a token")
}

fn latest_with_subject<'a>(received: &'a [ReceivedEmail], subject: &str) -> &'a ReceivedEmail {
    received
        .iter()
        .rev()
        .find(|email| email.subject == subject)
        .unwrap_or_else(|| panic!("no email with subject {subject:?}"))
}

async fn user_column<T: sea_orm::TryGetable>(app: &TestApp, user: &TestUser, column: &str) -> T {
    select_rows(app, "user", &[column], &[("id", user.id.into())]).await[0]
        .try_get("", column)
        .unwrap_or_else(|error| panic!("read user.{column}: {error}"))
}

fn assert_error_code(response: &JsonResponse, status: StatusCode, error_code: &str) {
    assert_eq!(response.status, status, "{}", response.body);
    assert_eq!(response.body["error_code"], error_code, "{}", response.body);
}

async fn make_single_sign_on_only(app: &TestApp, user: &TestUser) {
    update_rows(
        app,
        "user",
        &[("password_hash", sea_orm::Value::String(None))],
        &[("id", user.id.into())],
    )
    .await;
    insert_row(
        app,
        "federated_identity",
        &[
            ("id", Uuid::new_v4().into()),
            ("user_id", user.id.into()),
            ("provider_name", SINGLE_SIGN_ON_PROVIDER.into()),
            ("subject", Uuid::new_v4().to_string().into()),
            ("email", user.email.clone().into()),
            ("email_verified", true.into()),
            ("created_at", now().into()),
            ("updated_at", now().into()),
        ],
    )
    .await;
}

#[tokio::test]
async fn forgot_password_answers_the_same_for_every_email() {
    let app = TestApp::spawn_with_email_and_single_sign_on().await;
    let with_password = create_user(&app, "with-password", PLATFORM_MEMBER).await;
    let single_sign_on_only = create_user(&app, "sso-only", PLATFORM_MEMBER).await;
    make_single_sign_on_only(&app, &single_sign_on_only).await;
    let unknown_email = format!("unknown-{}@example.test", Uuid::new_v4().simple());
    let visitor = app.session(None);

    let answers: Vec<JsonResponse> = vec![
        request_reset(&visitor, &with_password.email).await,
        request_reset(&visitor, &single_sign_on_only.email).await,
        request_reset(&visitor, &unknown_email).await,
    ];
    for answer in &answers {
        assert_eq!(answer.status, StatusCode::ACCEPTED, "{}", answer.body);
        assert_eq!(answer.body, json!({ "message": RESET_REQUESTED_MESSAGE }));
    }

    wait_for_queued_emails(&app, &with_password.email, PASSWORD_RESET_EMAIL_KIND, 1).await;
    wait_for_queued_emails(&app, &single_sign_on_only.email, NO_PASSWORD_EMAIL_KIND, 1).await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert_eq!(
        count_rows(
            &app,
            EMAIL_OUTBOX_TABLE,
            &[("recipient", unknown_email.as_str().into())]
        )
        .await,
        0
    );
    assert_eq!(
        queued_count(&app, &single_sign_on_only.email, PASSWORD_RESET_EMAIL_KIND).await,
        0
    );
}

#[tokio::test]
async fn an_sso_only_account_gets_a_no_password_email() {
    let app = TestApp::spawn_with_email_and_single_sign_on().await;
    let user = create_user(&app, "sso-only", PLATFORM_MEMBER).await;
    make_single_sign_on_only(&app, &user).await;

    request_reset(&app.session(None), &user.email).await;
    wait_for_queued_emails(&app, &user.email, NO_PASSWORD_EMAIL_KIND, 1).await;
    app.deliver_due_emails().await;

    let received = shared_mail_server().await.emails_to(&user.email).await;
    let notice = latest_with_subject(&received, "Your Sopho account has no password");
    assert!(notice.text.contains(SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME));
    assert!(notice.text.contains(&format!("{}/signin", app.base_url())));
    assert!(!notice.text.contains(RESET_LINK_PATH));
}

#[tokio::test]
async fn a_reset_link_sets_a_new_password_once() {
    let app = TestApp::spawn_with_email().await;
    let user = create_user(&app, "forgetful", PLATFORM_MEMBER).await;
    let visitor = app.session(None);
    request_reset(&visitor, &user.email).await;
    let token = emailed_reset_token(&app, &user).await;

    let first_reset = reset(&visitor, &token, NEW_ACCOUNT_PASSWORD).await;
    assert_eq!(
        first_reset.status,
        StatusCode::NO_CONTENT,
        "{}",
        first_reset.body
    );

    assert_eq!(
        app.session(None)
            .sign_in(&user.email, NEW_ACCOUNT_PASSWORD)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        app.session(None)
            .sign_in(&user.email, TEST_USER_PASSWORD)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    assert_error_code(
        &reset(&visitor, &token, CHANGED_PASSWORD).await,
        StatusCode::BAD_REQUEST,
        INVALID_RESET_LINK_CODE,
    );
    wait_for_queued_emails(&app, &user.email, PASSWORD_CHANGED_EMAIL_KIND, 1).await;
}

#[tokio::test]
async fn a_reset_link_expires_after_an_hour() {
    let app = TestApp::spawn_with_email().await;
    let user = create_user(&app, "slow", PLATFORM_MEMBER).await;
    let visitor = app.session(None);
    request_reset(&visitor, &user.email).await;
    let token = emailed_reset_token(&app, &user).await;

    let expires_at: DateTime<FixedOffset> = select_rows(
        &app,
        "password_reset_token",
        &["expires_at"],
        &[("user_id", user.id.into())],
    )
    .await[0]
        .try_get("", "expires_at")
        .expect("read the reset link expiry");
    let lifetime = expires_at - now();
    assert!(lifetime > Duration::minutes(55) && lifetime <= Duration::minutes(60));

    update_rows(
        &app,
        "password_reset_token",
        &[("expires_at", (now() - Duration::minutes(1)).into())],
        &[("user_id", user.id.into())],
    )
    .await;
    assert_error_code(
        &reset(&visitor, &token, NEW_ACCOUNT_PASSWORD).await,
        StatusCode::BAD_REQUEST,
        INVALID_RESET_LINK_CODE,
    );
}

#[tokio::test]
async fn resetting_ends_every_session_and_does_not_sign_in() {
    let app = TestApp::spawn_with_email().await;
    let user = create_user(&app, "compromised", PLATFORM_MEMBER).await;
    let earlier_session = sign_in(&app, &user).await;
    assert_eq!(earlier_session.get("/auth/me").await.status, StatusCode::OK);
    let visitor = app.session(None);
    request_reset(&visitor, &user.email).await;
    let token = emailed_reset_token(&app, &user).await;

    assert_eq!(
        reset(&visitor, &token, NEW_ACCOUNT_PASSWORD).await.status,
        StatusCode::NO_CONTENT
    );

    assert_eq!(
        earlier_session.get("/auth/me").await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        visitor.get("/auth/session").await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn resetting_verifies_the_email_and_enforces_the_rules() {
    let app = TestApp::spawn_with_email().await;
    let user = create_user(&app, "unverified", PLATFORM_MEMBER).await;
    assert!(
        user_column::<Option<DateTime<FixedOffset>>>(&app, &user, "email_verified_at")
            .await
            .is_none()
    );
    let visitor = app.session(None);
    request_reset(&visitor, &user.email).await;
    let token = emailed_reset_token(&app, &user).await;

    assert_error_code(
        &reset(&visitor, &token, "short").await,
        StatusCode::BAD_REQUEST,
        PASSWORD_TOO_SHORT_CODE,
    );
    assert_eq!(
        reset(&visitor, &token, NEW_ACCOUNT_PASSWORD).await.status,
        StatusCode::NO_CONTENT
    );
    assert!(
        user_column::<Option<DateTime<FixedOffset>>>(&app, &user, "email_verified_at")
            .await
            .is_some()
    );
}

#[tokio::test]
async fn at_most_three_reset_emails_an_hour() {
    let app = TestApp::spawn_with_email().await;
    let user = create_user(&app, "impatient", PLATFORM_MEMBER).await;
    let visitor = app.session(None);

    for expected in 1..=3 {
        request_reset(&visitor, &user.email).await;
        wait_for_queued_emails(&app, &user.email, PASSWORD_RESET_EMAIL_KIND, expected).await;
    }
    let fourth = request_reset(&visitor, &user.email).await;
    assert_eq!(fourth.status, StatusCode::ACCEPTED);
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    assert_eq!(
        queued_count(&app, &user.email, PASSWORD_RESET_EMAIL_KIND).await,
        3
    );
}

#[tokio::test]
async fn changing_the_password_needs_the_current_one() {
    let app = TestApp::spawn_with_email().await;
    let user = create_user(&app, "changer", PLATFORM_MEMBER).await;
    let session = sign_in(&app, &user).await;

    assert_error_code(
        &change(&session, "not the current password", CHANGED_PASSWORD).await,
        StatusCode::BAD_REQUEST,
        WRONG_CURRENT_PASSWORD_CODE,
    );
    assert_eq!(
        user_column::<i32>(&app, &user, "failed_sign_in_count").await,
        1
    );

    let changed = change(&session, TEST_USER_PASSWORD, CHANGED_PASSWORD).await;
    assert_eq!(changed.status, StatusCode::NO_CONTENT, "{}", changed.body);
    assert_eq!(
        app.session(None)
            .sign_in(&user.email, CHANGED_PASSWORD)
            .await
            .status,
        StatusCode::OK
    );
    wait_for_queued_emails(&app, &user.email, PASSWORD_CHANGED_EMAIL_KIND, 1).await;
}

#[tokio::test]
async fn changing_the_password_keeps_only_this_session() {
    let app = TestApp::spawn().await;
    let user = create_user(&app, "two-browsers", PLATFORM_MEMBER).await;
    let laptop = sign_in(&app, &user).await;
    let phone = sign_in(&app, &user).await;

    assert_eq!(
        change(&laptop, TEST_USER_PASSWORD, CHANGED_PASSWORD)
            .await
            .status,
        StatusCode::NO_CONTENT
    );

    assert_eq!(laptop.get("/auth/me").await.status, StatusCode::OK);
    assert_eq!(phone.get("/auth/me").await.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn ten_failures_block_password_sign_in_for_fifteen_minutes() {
    let app = TestApp::spawn().await;
    let user = create_user(&app, "attacked", PLATFORM_MEMBER).await;
    let attacker = app.session(None);

    for _ in 0..10 {
        assert_eq!(
            attacker.sign_in(&user.email, "a wrong guess").await.status,
            StatusCode::UNAUTHORIZED
        );
    }
    let blocked = attacker.sign_in(&user.email, TEST_USER_PASSWORD).await;
    assert_error_code(
        &blocked,
        StatusCode::TOO_MANY_REQUESTS,
        TOO_MANY_SIGN_IN_ATTEMPTS_CODE,
    );
    assert_eq!(
        blocked.body["message"],
        "Too many attempts. Try again in 15 minutes or reset your password."
    );

    let blocked_until: DateTime<FixedOffset> =
        user_column::<Option<DateTime<FixedOffset>>>(&app, &user, "sign_in_blocked_until")
            .await
            .expect("sign-in is blocked");
    let block = blocked_until - now();
    assert!(block > Duration::minutes(14) && block <= Duration::minutes(15));
}

#[tokio::test]
async fn a_successful_sign_in_clears_the_failure_count() {
    let app = TestApp::spawn().await;
    let user = create_user(&app, "clumsy", PLATFORM_MEMBER).await;
    let session = app.session(None);
    for _ in 0..3 {
        session.sign_in(&user.email, "a wrong guess").await;
    }
    assert_eq!(
        user_column::<i32>(&app, &user, "failed_sign_in_count").await,
        3
    );

    assert_eq!(
        session
            .sign_in(&user.email, TEST_USER_PASSWORD)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        user_column::<i32>(&app, &user, "failed_sign_in_count").await,
        0
    );
}

#[tokio::test]
async fn a_reset_clears_the_block() {
    let app = TestApp::spawn_with_email().await;
    let user = create_user(&app, "blocked", PLATFORM_MEMBER).await;
    update_rows(
        &app,
        "user",
        &[(
            "sign_in_blocked_until",
            (now() + Duration::minutes(15)).into(),
        )],
        &[("id", user.id.into())],
    )
    .await;
    let visitor = app.session(None);
    request_reset(&visitor, &user.email).await;
    let token = emailed_reset_token(&app, &user).await;

    assert_eq!(
        reset(&visitor, &token, NEW_ACCOUNT_PASSWORD).await.status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        visitor
            .sign_in(&user.email, NEW_ACCOUNT_PASSWORD)
            .await
            .status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_deactivated_account_is_told_only_after_the_right_password() {
    let app = TestApp::spawn().await;
    let user = create_user(&app, "departed", PLATFORM_MEMBER).await;
    update_rows(
        &app,
        "user",
        &[("status", USER_STATUS_DEACTIVATED.into())],
        &[("id", user.id.into())],
    )
    .await;
    let session = app.session(None);

    let wrong_password = session.sign_in(&user.email, "a wrong guess").await;
    assert_eq!(wrong_password.status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_password.body["message"], "Invalid credentials");

    assert_error_code(
        &session.sign_in(&user.email, TEST_USER_PASSWORD).await,
        StatusCode::FORBIDDEN,
        ACCOUNT_DEACTIVATED_CODE,
    );
}

#[tokio::test]
async fn the_sign_in_page_options_say_whether_reset_is_available() {
    let with_email = TestApp::spawn_with_email_and_single_sign_on().await;
    let options = with_email
        .session(None)
        .get("/auth/sign_in_page_option")
        .await;
    assert_eq!(options.status, StatusCode::OK);
    assert_eq!(options.body["password_reset_available"], true);
    assert_eq!(
        options.body["providers"][0]["name"],
        SINGLE_SIGN_ON_PROVIDER
    );

    let without_email = TestApp::spawn().await;
    let options = without_email
        .session(None)
        .get("/auth/sign_in_page_option")
        .await;
    assert_eq!(options.body["password_reset_available"], false);
    assert_eq!(options.body["providers"], Value::Array(Vec::new()));
    assert_eq!(
        request_reset(&without_email.session(None), "anyone@example.test")
            .await
            .status,
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn email_links_ignore_the_request_host_header() {
    let app = TestApp::spawn_with_email().await;
    let user = create_user(&app, "spoofed", PLATFORM_MEMBER).await;
    let visitor = app.session(None);

    let response = visitor
        .client
        .post(visitor.api_url("/auth/password_reset_request"))
        .header("Host", "attacker.example")
        .json(&json!({ "email": user.email }))
        .send()
        .await
        .expect("send the reset request");
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let token = emailed_reset_token(&app, &user).await;
    assert!(!token.is_empty());
    let received = shared_mail_server().await.emails_to(&user.email).await;
    let reset_email = latest_with_subject(&received, "Reset your Sopho password");
    assert!(!reset_email.text.contains("attacker.example"));
    assert!(!reset_email.html.contains("attacker.example"));
}
