use crate::support::constants::{
    ADMIN_EMAIL, ADMIN_PASSWORD, EMAIL_MANAGE_PERMISSION, EMAIL_NOT_VERIFIED, LOGIN_ATTEMPT_TABLE,
    OIDC_STATE_COOKIE, OTHER_SINGLE_SIGN_ON_PROVIDER, PASSWORD_ACCOUNT, PLATFORM_MEMBER,
    SINGLE_SIGN_ON_PROVIDER, SSO_EXPIRED, SSO_FAILED, TENANT_APPOINT_OWNER_PERMISSION,
    TENANT_CREATE_PERMISSION, USER_MANAGE_PERMISSION,
};
use crate::support::database::{count_rows, now, select_rows, update_rows};
use crate::support::identity_provider::{
    SignInOutcome, complete_sign_in, sign_in_at_provider, sign_in_with_provider, start_sign_in,
};
use crate::support::{TestApp, UserSession};
use chrono::Duration;
use reqwest::header::{COOKIE, LOCATION};
use reqwest::redirect::Policy;
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};

fn verified_claims(email: &str, full_name: &str) -> Value {
    json!({ "email": email, "email_verified": true, "name": full_name })
}

async fn signed_in_user(session: &UserSession) -> Value {
    let profile = session.get("/auth/me").await;
    assert_eq!(profile.status, StatusCode::OK, "{}", profile.body);
    profile.body
}

async fn attempts_with_state(app: &TestApp, state: &str) -> i64 {
    count_rows(app, LOGIN_ATTEMPT_TABLE, &[("state", state.into())]).await
}

async fn expire_attempt(app: &TestApp, state: &str) {
    let expired = update_rows(
        app,
        LOGIN_ATTEMPT_TABLE,
        &[("expires_at", (now() - Duration::minutes(1)).into())],
        &[("state", state.into())],
    )
    .await;
    assert_eq!(expired, 1);
}

async fn callback_with_state_cookie(
    callback_url: &str,
    state_cookie: Option<&str>,
) -> SignInOutcome {
    let request = Client::builder()
        .redirect(Policy::none())
        .build()
        .expect("build a client without the starting cookie")
        .get(callback_url);
    let request = match state_cookie {
        Some(value) => request.header(COOKIE, format!("{OIDC_STATE_COOKIE}={value}")),
        None => request,
    };
    let response = request.send().await.expect("return to the application");
    SignInOutcome {
        location: response
            .headers()
            .get(LOCATION)
            .expect("the callback redirects")
            .to_str()
            .expect("the location is text")
            .to_string(),
    }
}

#[tokio::test]
async fn password_login_is_refused_for_an_sso_only_user() {
    let app = TestApp::spawn_with_single_sign_on().await;

    let administrator = app.session(None);
    let signed_in = administrator
        .sign_in(&format!(" {} ", ADMIN_EMAIL.to_uppercase()), ADMIN_PASSWORD)
        .await;
    assert_eq!(signed_in.status, StatusCode::OK, "{}", signed_in.body);
    assert_eq!(
        signed_in_user(&administrator).await["platform_permissions"],
        json!([
            TENANT_CREATE_PERMISSION,
            USER_MANAGE_PERMISSION,
            TENANT_APPOINT_OWNER_PERMISSION,
            EMAIL_MANAGE_PERMISSION
        ]),
        "the seeded admin administers the platform"
    );
    assert_eq!(
        app.session(None).sign_in(ADMIN_EMAIL, "wrong").await.status,
        StatusCode::UNAUTHORIZED
    );

    let sso_user = app.session(None);
    let outcome = sign_in_with_provider(
        &sso_user,
        SINGLE_SIGN_ON_PROVIDER,
        "sso-only",
        verified_claims("sso-only@example.test", "Sso Only"),
    )
    .await;
    assert!(outcome.is_signed_in(), "{}", outcome.location);
    assert_eq!(
        app.session(None)
            .sign_in("sso-only@example.test", "")
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn a_returning_identity_signs_in_as_the_same_user() {
    let app = TestApp::spawn_with_single_sign_on().await;

    let first = app.session(None);
    assert!(
        sign_in_with_provider(
            &first,
            SINGLE_SIGN_ON_PROVIDER,
            "subject-1",
            verified_claims("returning@example.test", "Returning User"),
        )
        .await
        .is_signed_in()
    );
    let users_after_first_sign_in = count_rows(&app, "user", &[]).await;

    let second = app.session(None);
    assert!(
        sign_in_with_provider(
            &second,
            SINGLE_SIGN_ON_PROVIDER,
            "subject-1",
            json!({ "email": "changed@example.test", "email_verified": false }),
        )
        .await
        .is_signed_in()
    );

    assert_eq!(
        signed_in_user(&first).await["id"],
        signed_in_user(&second).await["id"],
        "the identity, not the email, decides who signs in"
    );
    assert_eq!(
        count_rows(&app, "user", &[]).await,
        users_after_first_sign_in
    );
    assert_eq!(count_rows(&app, "identity", &[]).await, 1);
}

#[tokio::test]
async fn a_verified_email_never_links_to_a_password_account() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let users_before = count_rows(&app, "user", &[]).await;

    let session = app.session(None);
    let outcome = sign_in_with_provider(
        &session,
        SINGLE_SIGN_ON_PROVIDER,
        "admin-lookalike",
        verified_claims(&ADMIN_EMAIL.to_uppercase(), "Admin"),
    )
    .await;

    assert_eq!(outcome.error_code(), Some(PASSWORD_ACCOUNT));
    assert_eq!(
        session.get("/auth/me").await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        count_rows(&app, "identity", &[]).await,
        0,
        "an account whose email no identity provider verified is never taken over"
    );
    assert_eq!(count_rows(&app, "user", &[]).await, users_before);
    assert_eq!(
        app.session(None)
            .sign_in(ADMIN_EMAIL, ADMIN_PASSWORD)
            .await
            .status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_verified_email_links_accounts_created_through_single_sign_on() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let users_before = count_rows(&app, "user", &[]).await;

    let first = app.session(None);
    assert!(
        sign_in_with_provider(
            &first,
            SINGLE_SIGN_ON_PROVIDER,
            "test-subject",
            verified_claims("shared@example.test", "Shared"),
        )
        .await
        .is_signed_in()
    );
    let linked = app.session(None);
    assert!(
        sign_in_with_provider(
            &linked,
            OTHER_SINGLE_SIGN_ON_PROVIDER,
            "other-subject",
            verified_claims("Shared@Example.test", "Shared"),
        )
        .await
        .is_signed_in()
    );

    assert_eq!(
        signed_in_user(&linked).await["id"],
        signed_in_user(&first).await["id"]
    );
    assert_eq!(count_rows(&app, "user", &[]).await, users_before + 1);
    assert_eq!(count_rows(&app, "identity", &[]).await, 2);
}

#[tokio::test]
async fn an_unverified_email_is_refused_and_creates_nothing() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let users_before = count_rows(&app, "user", &[]).await;

    let stranger = sign_in_with_provider(
        &app.session(None),
        SINGLE_SIGN_ON_PROVIDER,
        "stranger",
        json!({ "email": "new@example.test", "email_verified": false, "name": "New" }),
    )
    .await;
    assert_eq!(stranger.error_code(), Some(EMAIL_NOT_VERIFIED));

    let impostor = sign_in_with_provider(
        &app.session(None),
        SINGLE_SIGN_ON_PROVIDER,
        "impostor",
        json!({ "email": ADMIN_EMAIL, "email_verified": false, "name": "Impostor" }),
    )
    .await;
    assert_eq!(
        impostor.error_code(),
        Some(EMAIL_NOT_VERIFIED),
        "an unverified email never links to an existing account"
    );

    let without_email = sign_in_with_provider(
        &app.session(None),
        SINGLE_SIGN_ON_PROVIDER,
        "no-email",
        json!({ "email_verified": true, "name": "Nobody" }),
    )
    .await;
    assert_eq!(without_email.error_code(), Some(EMAIL_NOT_VERIFIED));

    assert_eq!(count_rows(&app, "user", &[]).await, users_before);
    assert_eq!(count_rows(&app, "identity", &[]).await, 0);
}

#[tokio::test]
async fn a_new_verified_identity_creates_a_member_without_a_password() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let users_before = count_rows(&app, "user", &[]).await;

    let nameless = sign_in_with_provider(
        &app.session(None),
        SINGLE_SIGN_ON_PROVIDER,
        "nameless",
        json!({ "email": "nameless@example.test", "email_verified": true }),
    )
    .await;
    assert_eq!(nameless.error_code(), Some(SSO_FAILED));
    assert_eq!(count_rows(&app, "user", &[]).await, users_before);

    let newcomer = app.session(None);
    assert!(
        sign_in_with_provider(
            &newcomer,
            SINGLE_SIGN_ON_PROVIDER,
            "newcomer",
            verified_claims(" Newcomer@Example.test ", "New Comer"),
        )
        .await
        .is_signed_in()
    );
    let profile = signed_in_user(&newcomer).await;
    assert_eq!(profile["email"], "newcomer@example.test");
    assert_eq!(profile["username"], "newcomer@example.test");
    assert_eq!(profile["full_name"], "New Comer");
    assert_eq!(profile["platform_permissions"], json!([]));

    let stored = select_rows(
        &app,
        "user",
        &["password_hash", "platform_role"],
        &[("email", "newcomer@example.test".into())],
    )
    .await;
    assert_eq!(stored.len(), 1);
    assert_eq!(
        stored[0]
            .try_get::<Option<String>>("", "password_hash")
            .expect("read the password hash"),
        None
    );
    assert_eq!(
        stored[0]
            .try_get::<String>("", "platform_role")
            .expect("read the platform role"),
        PLATFORM_MEMBER
    );
}

#[tokio::test]
async fn a_login_attempt_is_consumed_once() {
    let app = TestApp::spawn_with_single_sign_on().await;

    let browser = app.session(None);
    let started = start_sign_in(&browser, SINGLE_SIGN_ON_PROVIDER).await;
    let callback_url = sign_in_at_provider(
        &started,
        "single-use",
        verified_claims("single-use@example.test", "Single Use"),
    )
    .await;
    assert!(
        complete_sign_in(&browser, &callback_url)
            .await
            .is_signed_in()
    );
    assert_eq!(attempts_with_state(&app, &started.state).await, 0);
    assert_eq!(
        complete_sign_in(&browser, &callback_url).await.error_code(),
        Some(SSO_EXPIRED)
    );

    let other_browser = app.session(None);
    let started = start_sign_in(&other_browser, SINGLE_SIGN_ON_PROVIDER).await;
    let callback_url = sign_in_at_provider(
        &started,
        "wrong-provider",
        verified_claims("wrong-provider@example.test", "Wrong Provider"),
    )
    .await;
    let wrong_provider_callback_url = callback_url.replace(
        &format!("/auth/oidc/{SINGLE_SIGN_ON_PROVIDER}/callback"),
        &format!("/auth/oidc/{OTHER_SINGLE_SIGN_ON_PROVIDER}/callback"),
    );
    assert_eq!(
        complete_sign_in(&other_browser, &wrong_provider_callback_url)
            .await
            .error_code(),
        Some(SSO_EXPIRED)
    );
    assert_eq!(
        attempts_with_state(&app, &started.state).await,
        0,
        "a callback for the wrong provider still spends the attempt"
    );
}

#[tokio::test]
async fn an_expired_login_attempt_is_refused() {
    let app = TestApp::spawn_with_single_sign_on().await;

    let browser = app.session(None);
    let started = start_sign_in(&browser, SINGLE_SIGN_ON_PROVIDER).await;
    let callback_url = sign_in_at_provider(
        &started,
        "late",
        verified_claims("late@example.test", "Late"),
    )
    .await;
    expire_attempt(&app, &started.state).await;
    assert_eq!(
        complete_sign_in(&browser, &callback_url).await.error_code(),
        Some(SSO_EXPIRED)
    );
    assert_eq!(attempts_with_state(&app, &started.state).await, 0);

    let abandoned = start_sign_in(&app.session(None), SINGLE_SIGN_ON_PROVIDER).await;
    expire_attempt(&app, &abandoned.state).await;
    let fresh = start_sign_in(&app.session(None), SINGLE_SIGN_ON_PROVIDER).await;
    let remaining_states: Vec<String> = select_rows(&app, LOGIN_ATTEMPT_TABLE, &["state"], &[])
        .await
        .iter()
        .map(|row| row.try_get::<String>("", "state").expect("read the state"))
        .collect();
    assert_eq!(
        remaining_states,
        vec![fresh.state],
        "starting a login purges abandoned attempts"
    );
}

#[tokio::test]
async fn a_callback_without_the_starting_cookie_is_refused() {
    let app = TestApp::spawn_with_single_sign_on().await;

    let browser = app.session(None);
    let started = start_sign_in(&browser, SINGLE_SIGN_ON_PROVIDER).await;
    let callback_url = sign_in_at_provider(
        &started,
        "cookie-check",
        verified_claims("cookie-check@example.test", "Cookie Check"),
    )
    .await;
    let state_prefix = &started.state[..started.state.len() - 1];

    for state_cookie in [None, Some("another-state"), Some(state_prefix)] {
        assert_eq!(
            callback_with_state_cookie(&callback_url, state_cookie)
                .await
                .error_code(),
            Some(SSO_EXPIRED),
            "a callback carrying {state_cookie:?} instead of the starting cookie is refused"
        );
    }
    assert!(
        complete_sign_in(&browser, &callback_url)
            .await
            .is_signed_in(),
        "the browser that started the sign-in still completes it"
    );
}

#[tokio::test]
async fn a_callback_without_the_starting_cookie_leaves_the_attempt_for_its_browser() {
    let app = TestApp::spawn_with_single_sign_on().await;

    let browser = app.session(None);
    let started = start_sign_in(&browser, SINGLE_SIGN_ON_PROVIDER).await;
    let callback_url = sign_in_at_provider(
        &started,
        "victim",
        verified_claims("victim@example.test", "Victim"),
    )
    .await;

    let forged = callback_with_state_cookie(&callback_url, None).await;
    assert_eq!(forged.error_code(), Some(SSO_EXPIRED));
    assert_eq!(
        attempts_with_state(&app, &started.state).await,
        1,
        "a forged callback must not spend the real browser's attempt"
    );
    assert!(
        complete_sign_in(&browser, &callback_url)
            .await
            .is_signed_in()
    );
}
