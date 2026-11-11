use crate::support::constants::{
    ACCOUNT_DEACTIVATED_SIGN_IN_ERROR, CONNECT_RESULT_LOCATION_PREFIX, EMAIL_OUTBOX_TABLE,
    LAST_SIGN_IN_METHOD_CODE, OTHER_SINGLE_SIGN_ON_PROVIDER, PASSWORD_ACCOUNT, PLATFORM_MEMBER,
    SIGN_IN_METHOD_ADDED_EMAIL_KIND, SIGN_IN_METHOD_REMOVED_EMAIL_KIND, SINGLE_SIGN_ON_PROVIDER,
    SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME, SSO_FAILED, USER_STATUS_DEACTIVATED,
};
use crate::support::database::{count_rows, insert_row, now, select_rows, update_rows};
use crate::support::identity_provider::{
    complete_sign_in, sign_in_at_provider, sign_in_with_provider, start_connect,
};
use crate::support::tenants::{TestUser, create_user, sign_in};
use crate::support::{TestApp, UserSession};
use chrono::{DateTime, FixedOffset};
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

fn verified_claims(email: &str, full_name: &str) -> Value {
    json!({ "email": email, "email_verified": true, "name": full_name })
}

async fn federated_identities_of(app: &TestApp, user: &TestUser) -> Vec<(String, String)> {
    select_rows(
        app,
        "federated_identity",
        &["provider_name", "subject"],
        &[("user_id", user.id.into())],
    )
    .await
    .iter()
    .map(|row| {
        (
            row.try_get("", "provider_name").expect("read the provider"),
            row.try_get("", "subject").expect("read the subject"),
        )
    })
    .collect()
}

async fn add_federated_identity(
    app: &TestApp,
    user: &TestUser,
    provider_name: &str,
    subject: &str,
) {
    insert_row(
        app,
        "federated_identity",
        &[
            ("id", Uuid::new_v4().into()),
            ("user_id", user.id.into()),
            ("provider_name", provider_name.into()),
            ("subject", subject.into()),
            ("email", user.email.clone().into()),
            ("email_verified", true.into()),
            ("created_at", now().into()),
            ("updated_at", now().into()),
        ],
    )
    .await;
}

async fn set_user_columns(app: &TestApp, user: &TestUser, changes: &[(&str, sea_orm::Value)]) {
    assert_eq!(
        update_rows(app, "user", changes, &[("id", user.id.into())]).await,
        1
    );
}

async fn prove_email(app: &TestApp, user: &TestUser) {
    set_user_columns(app, user, &[("email_verified_at", now().into())]).await;
}

async fn remove_password(app: &TestApp, user: &TestUser) {
    set_user_columns(
        app,
        user,
        &[("password_hash", sea_orm::Value::String(None))],
    )
    .await;
}

async fn connect(
    session: &UserSession,
    provider_name: &str,
    subject: &str,
    claims: Value,
) -> String {
    let started = start_connect(session, provider_name).await;
    let callback_url = sign_in_at_provider(&started, subject, claims).await;
    complete_sign_in(session, &callback_url).await.location
}

async fn signed_in_user_id(session: &UserSession) -> String {
    let profile = session.get("/auth/me").await;
    assert_eq!(profile.status, StatusCode::OK, "{}", profile.body);
    profile.body["id"].as_str().expect("a user id").to_string()
}

async fn queued_notices(app: &TestApp, user: &TestUser, kind: &str) -> i64 {
    count_rows(
        app,
        EMAIL_OUTBOX_TABLE,
        &[
            ("recipient", user.email.clone().into()),
            ("kind", kind.into()),
        ],
    )
    .await
}

#[tokio::test]
async fn sso_links_into_an_account_with_a_proven_email() {
    let app = TestApp::spawn_with_email_and_single_sign_on().await;
    let user = create_user(&app, "proven", PLATFORM_MEMBER).await;
    prove_email(&app, &user).await;
    let browser = app.session(None);

    let outcome = sign_in_with_provider(
        &browser,
        SINGLE_SIGN_ON_PROVIDER,
        "proven-subject",
        verified_claims(&user.email, &user.full_name),
    )
    .await;

    assert!(outcome.is_signed_in(), "{}", outcome.location);
    assert_eq!(signed_in_user_id(&browser).await, user.id.to_string());
    assert_eq!(
        federated_identities_of(&app, &user).await,
        vec![(
            SINGLE_SIGN_ON_PROVIDER.to_string(),
            "proven-subject".to_string()
        )]
    );
    assert_eq!(
        queued_notices(&app, &user, SIGN_IN_METHOD_ADDED_EMAIL_KIND).await,
        1
    );
}

#[tokio::test]
async fn sso_refuses_an_account_whose_email_is_not_proven() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let user = create_user(&app, "unproven", PLATFORM_MEMBER).await;

    let outcome = sign_in_with_provider(
        &app.session(None),
        SINGLE_SIGN_ON_PROVIDER,
        "unproven-subject",
        verified_claims(&user.email, &user.full_name),
    )
    .await;

    assert_eq!(outcome.error_code(), Some(PASSWORD_ACCOUNT));
    assert!(federated_identities_of(&app, &user).await.is_empty());
}

#[tokio::test]
async fn sso_refuses_a_second_subject_at_the_same_provider() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let user = create_user(&app, "reassigned", PLATFORM_MEMBER).await;
    prove_email(&app, &user).await;
    add_federated_identity(&app, &user, SINGLE_SIGN_ON_PROVIDER, "original-subject").await;

    let outcome = sign_in_with_provider(
        &app.session(None),
        SINGLE_SIGN_ON_PROVIDER,
        "reassigned-subject",
        verified_claims(&user.email, &user.full_name),
    )
    .await;

    assert_eq!(outcome.error_code(), Some(SSO_FAILED));
    assert_eq!(federated_identities_of(&app, &user).await.len(), 1);
}

#[tokio::test]
async fn sso_refuses_a_deactivated_account() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let user = create_user(&app, "departed", PLATFORM_MEMBER).await;
    add_federated_identity(&app, &user, SINGLE_SIGN_ON_PROVIDER, "departed-subject").await;
    set_user_columns(&app, &user, &[("status", USER_STATUS_DEACTIVATED.into())]).await;

    let outcome = sign_in_with_provider(
        &app.session(None),
        SINGLE_SIGN_ON_PROVIDER,
        "departed-subject",
        verified_claims(&user.email, &user.full_name),
    )
    .await;

    assert_eq!(
        outcome.error_code(),
        Some(ACCOUNT_DEACTIVATED_SIGN_IN_ERROR)
    );
}

#[tokio::test]
async fn connect_adds_the_provider_and_proves_the_email() {
    let app = TestApp::spawn_with_email_and_single_sign_on().await;
    let user = create_user(&app, "connector", PLATFORM_MEMBER).await;
    let session = sign_in(&app, &user).await;

    let location = connect(
        &session,
        SINGLE_SIGN_ON_PROVIDER,
        "connected-subject",
        verified_claims(&user.email, &user.full_name),
    )
    .await;

    assert_eq!(
        location,
        format!("{CONNECT_RESULT_LOCATION_PREFIX}connected&provider={SINGLE_SIGN_ON_PROVIDER}")
    );
    assert_eq!(
        federated_identities_of(&app, &user).await,
        vec![(
            SINGLE_SIGN_ON_PROVIDER.to_string(),
            "connected-subject".to_string()
        )]
    );
    let email_verified_at: Option<DateTime<FixedOffset>> = select_rows(
        &app,
        "user",
        &["email_verified_at"],
        &[("id", user.id.into())],
    )
    .await[0]
        .try_get("", "email_verified_at")
        .expect("read the email verification time");
    assert!(email_verified_at.is_some());
    assert_eq!(signed_in_user_id(&session).await, user.id.to_string());
    assert_eq!(
        queued_notices(&app, &user, SIGN_IN_METHOD_ADDED_EMAIL_KIND).await,
        1
    );

    let later_browser = app.session(None);
    let outcome = sign_in_with_provider(
        &later_browser,
        SINGLE_SIGN_ON_PROVIDER,
        "connected-subject",
        verified_claims(&user.email, &user.full_name),
    )
    .await;
    assert!(outcome.is_signed_in(), "{}", outcome.location);
    assert_eq!(signed_in_user_id(&later_browser).await, user.id.to_string());
}

#[tokio::test]
async fn connect_refuses_a_different_email() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let user = create_user(&app, "mismatched", PLATFORM_MEMBER).await;
    let session = sign_in(&app, &user).await;

    let location = connect(
        &session,
        SINGLE_SIGN_ON_PROVIDER,
        "mismatched-subject",
        verified_claims("someone.else@example.test", &user.full_name),
    )
    .await;

    assert_eq!(
        location,
        format!("{CONNECT_RESULT_LOCATION_PREFIX}email_mismatch")
    );
    assert!(federated_identities_of(&app, &user).await.is_empty());
}

#[tokio::test]
async fn connect_refuses_a_federated_identity_owned_by_another_account() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let owner = create_user(&app, "owner", PLATFORM_MEMBER).await;
    add_federated_identity(&app, &owner, SINGLE_SIGN_ON_PROVIDER, "shared-subject").await;
    let user = create_user(&app, "latecomer", PLATFORM_MEMBER).await;
    let session = sign_in(&app, &user).await;

    let location = connect(
        &session,
        SINGLE_SIGN_ON_PROVIDER,
        "shared-subject",
        verified_claims(&user.email, &user.full_name),
    )
    .await;

    assert_eq!(
        location,
        format!("{CONNECT_RESULT_LOCATION_PREFIX}federated_identity_in_use")
    );
    assert!(federated_identities_of(&app, &user).await.is_empty());
}

#[tokio::test]
async fn disconnect_refuses_the_last_sign_in_method() {
    let app = TestApp::spawn_with_email_and_single_sign_on().await;
    let single_sign_on_only = create_user(&app, "sso-only", PLATFORM_MEMBER).await;
    remove_password(&app, &single_sign_on_only).await;
    add_federated_identity(
        &app,
        &single_sign_on_only,
        SINGLE_SIGN_ON_PROVIDER,
        "only-subject",
    )
    .await;
    let single_sign_on_session = app.session(None);
    assert!(
        sign_in_with_provider(
            &single_sign_on_session,
            SINGLE_SIGN_ON_PROVIDER,
            "only-subject",
            verified_claims(&single_sign_on_only.email, &single_sign_on_only.full_name),
        )
        .await
        .is_signed_in()
    );

    let refused = single_sign_on_session
        .delete(&format!(
            "/auth/oidc/{SINGLE_SIGN_ON_PROVIDER}/federated_identity"
        ))
        .await;
    assert_eq!(refused.status, StatusCode::CONFLICT, "{}", refused.body);
    assert_eq!(refused.body["error_code"], LAST_SIGN_IN_METHOD_CODE);
    assert_eq!(
        federated_identities_of(&app, &single_sign_on_only)
            .await
            .len(),
        1
    );

    let with_password = create_user(&app, "with-password", PLATFORM_MEMBER).await;
    add_federated_identity(
        &app,
        &with_password,
        SINGLE_SIGN_ON_PROVIDER,
        "spare-subject",
    )
    .await;
    let password_session = sign_in(&app, &with_password).await;
    let disconnected = password_session
        .delete(&format!(
            "/auth/oidc/{SINGLE_SIGN_ON_PROVIDER}/federated_identity"
        ))
        .await;
    assert_eq!(
        disconnected.status,
        StatusCode::NO_CONTENT,
        "{}",
        disconnected.body
    );
    assert!(
        federated_identities_of(&app, &with_password)
            .await
            .is_empty()
    );
    assert_eq!(
        queued_notices(&app, &with_password, SIGN_IN_METHOD_REMOVED_EMAIL_KIND).await,
        1
    );
    assert_eq!(
        password_session
            .delete(&format!(
                "/auth/oidc/{SINGLE_SIGN_ON_PROVIDER}/federated_identity"
            ))
            .await
            .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn my_sign_in_methods_list_says_what_can_be_disconnected() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let with_password = create_user(&app, "with-password", PLATFORM_MEMBER).await;
    add_federated_identity(
        &app,
        &with_password,
        SINGLE_SIGN_ON_PROVIDER,
        "listed-subject",
    )
    .await;
    let password_session = sign_in(&app, &with_password).await;

    let listed = password_session.get("/auth/my_sign_in_method").await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.body);
    assert_eq!(listed.body["has_password"], true);
    let providers = listed.body["providers"].as_array().expect("providers");
    let connected = providers
        .iter()
        .find(|provider| provider["name"] == SINGLE_SIGN_ON_PROVIDER)
        .expect("the connected provider");
    assert_eq!(
        connected["display_name"],
        SINGLE_SIGN_ON_PROVIDER_DISPLAY_NAME
    );
    assert_eq!(connected["connected"], true);
    assert_eq!(connected["email"], with_password.email);
    assert_eq!(connected["can_disconnect"], true);
    let unconnected = providers
        .iter()
        .find(|provider| provider["name"] == OTHER_SINGLE_SIGN_ON_PROVIDER)
        .expect("the other provider");
    assert_eq!(unconnected["connected"], false);
    assert_eq!(unconnected["can_disconnect"], false);

    let single_sign_on_only = create_user(&app, "sso-only", PLATFORM_MEMBER).await;
    remove_password(&app, &single_sign_on_only).await;
    add_federated_identity(
        &app,
        &single_sign_on_only,
        SINGLE_SIGN_ON_PROVIDER,
        "sole-subject",
    )
    .await;
    let single_sign_on_session = app.session(None);
    sign_in_with_provider(
        &single_sign_on_session,
        SINGLE_SIGN_ON_PROVIDER,
        "sole-subject",
        verified_claims(&single_sign_on_only.email, &single_sign_on_only.full_name),
    )
    .await;
    let listed = single_sign_on_session.get("/auth/my_sign_in_method").await;
    assert_eq!(listed.body["has_password"], false);
    let sole = listed.body["providers"]
        .as_array()
        .expect("providers")
        .iter()
        .find(|provider| provider["name"] == SINGLE_SIGN_ON_PROVIDER)
        .expect("the sole provider")
        .clone();
    assert_eq!(sole["can_disconnect"], false);
}
