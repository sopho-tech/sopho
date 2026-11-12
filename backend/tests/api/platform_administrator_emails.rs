use crate::support::constants::{
    ACCOUNT_DEACTIVATED_SIGN_IN_ERROR, CONNECT_RESULT_LOCATION_PREFIX, LISTED_ADMINISTRATOR_EMAIL,
    PLATFORM_ADMINISTRATOR, PLATFORM_MEMBER, SINGLE_SIGN_ON_PROVIDER, USER_STATUS_DEACTIVATED,
};
use crate::support::database::{now, select_rows, update_rows};
use crate::support::identity_provider::{
    SignInOutcome, complete_sign_in, sign_in_at_provider, sign_in_with_provider, start_connect,
};
use crate::support::tenants::{create_user, sign_in};
use crate::support::{DeploymentFlags, TestApp, UserSession};
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

fn claims(email: &str, is_email_verified: bool) -> Value {
    json!({ "email": email, "email_verified": is_email_verified, "name": "A Person" })
}

fn unique_email(label: &str) -> String {
    format!("{label}-{}@example.test", Uuid::new_v4().simple())
}

async fn spawn_passwordless() -> TestApp {
    TestApp::spawn_with_deployment_flags(DeploymentFlags::passwordless(&format!(
        " Founder@Example.test ,second-{LISTED_ADMINISTRATOR_EMAIL}"
    )))
    .await
}

async fn spawn_with_passwords_and_listed_administrator() -> TestApp {
    TestApp::spawn_with_deployment_flags(DeploymentFlags {
        platform_administrator_emails: Some(LISTED_ADMINISTRATOR_EMAIL.to_string()),
        ..DeploymentFlags::default()
    })
    .await
}

async fn sign_in_as(
    session: &UserSession,
    subject: &str,
    email: &str,
    is_email_verified: bool,
) -> SignInOutcome {
    sign_in_with_provider(
        session,
        SINGLE_SIGN_ON_PROVIDER,
        subject,
        claims(email, is_email_verified),
    )
    .await
}

async fn platform_role_of_email(app: &TestApp, email: &str) -> String {
    select_rows(app, "user", &["platform_role"], &[("email", email.into())])
        .await
        .first()
        .unwrap_or_else(|| panic!("no account uses {email}"))
        .try_get("", "platform_role")
        .expect("read the platform role")
}

async fn platform_permissions(session: &UserSession) -> Vec<Value> {
    let profile = session.get("/auth/me").await;
    assert_eq!(profile.status, StatusCode::OK, "{}", profile.body);
    profile.body["platform_permissions"]
        .as_array()
        .expect("the platform permissions")
        .clone()
}

#[tokio::test]
async fn a_listed_person_becomes_an_administrator_on_their_first_sign_in() {
    let app = spawn_passwordless().await;
    let listed = app.session(None);
    assert!(
        sign_in_as(&listed, "listed-subject", "FOUNDER@example.test", true)
            .await
            .is_signed_in()
    );
    assert_eq!(
        platform_role_of_email(&app, LISTED_ADMINISTRATOR_EMAIL).await,
        PLATFORM_ADMINISTRATOR
    );
    assert!(!platform_permissions(&listed).await.is_empty());

    let unlisted_email = unique_email("unlisted");
    let unlisted = app.session(None);
    assert!(
        sign_in_as(&unlisted, "unlisted-subject", &unlisted_email, true)
            .await
            .is_signed_in()
    );
    assert_eq!(
        platform_role_of_email(&app, &unlisted_email).await,
        PLATFORM_MEMBER
    );
    assert!(platform_permissions(&unlisted).await.is_empty());
}

#[tokio::test]
async fn promotion_follows_the_email_the_provider_verified_for_this_sign_in() {
    let app = spawn_passwordless().await;
    let stored_email = unique_email("stored");
    let session = app.session(None);
    assert!(
        sign_in_as(&session, "moving-subject", &stored_email, true)
            .await
            .is_signed_in()
    );
    assert_eq!(
        platform_role_of_email(&app, &stored_email).await,
        PLATFORM_MEMBER
    );

    let unverified = app.session(None);
    assert!(
        sign_in_as(
            &unverified,
            "moving-subject",
            LISTED_ADMINISTRATOR_EMAIL,
            false
        )
        .await
        .is_signed_in(),
        "a known identity signs in whatever email the provider reports"
    );
    assert_eq!(
        platform_role_of_email(&app, &stored_email).await,
        PLATFORM_MEMBER,
        "an unverified listed email promotes nobody"
    );

    let verified = app.session(None);
    assert!(
        sign_in_as(
            &verified,
            "moving-subject",
            LISTED_ADMINISTRATOR_EMAIL,
            true
        )
        .await
        .is_signed_in()
    );
    assert_eq!(
        platform_role_of_email(&app, &stored_email).await,
        PLATFORM_ADMINISTRATOR,
        "the account keeps its stored email and is promoted by the provider's verified one"
    );
}

#[tokio::test]
async fn a_listed_person_demoted_in_the_database_is_promoted_at_the_next_sign_in() {
    let app = spawn_passwordless().await;
    assert!(
        sign_in_as(
            &app.session(None),
            "listed-subject",
            LISTED_ADMINISTRATOR_EMAIL,
            true
        )
        .await
        .is_signed_in()
    );
    assert_eq!(
        update_rows(
            &app,
            "user",
            &[("platform_role", PLATFORM_MEMBER.into())],
            &[("email", LISTED_ADMINISTRATOR_EMAIL.into())],
        )
        .await,
        1
    );

    assert!(
        sign_in_as(
            &app.session(None),
            "listed-subject",
            LISTED_ADMINISTRATOR_EMAIL,
            true
        )
        .await
        .is_signed_in()
    );
    assert_eq!(
        platform_role_of_email(&app, LISTED_ADMINISTRATOR_EMAIL).await,
        PLATFORM_ADMINISTRATOR
    );
}

#[tokio::test]
async fn a_deactivated_listed_account_is_not_promoted() {
    let app = spawn_passwordless().await;
    let stored_email = unique_email("leaver");
    assert!(
        sign_in_as(&app.session(None), "leaver-subject", &stored_email, true)
            .await
            .is_signed_in()
    );
    assert_eq!(
        update_rows(
            &app,
            "user",
            &[("status", USER_STATUS_DEACTIVATED.into())],
            &[("email", stored_email.clone().into())],
        )
        .await,
        1
    );

    let outcome = sign_in_as(
        &app.session(None),
        "leaver-subject",
        LISTED_ADMINISTRATOR_EMAIL,
        true,
    )
    .await;

    assert_eq!(
        outcome.error_code(),
        Some(ACCOUNT_DEACTIVATED_SIGN_IN_ERROR)
    );
    assert_eq!(
        platform_role_of_email(&app, &stored_email).await,
        PLATFORM_MEMBER
    );
}

#[tokio::test]
async fn a_password_sign_in_never_promotes_and_connecting_the_provider_does() {
    let app = spawn_with_passwords_and_listed_administrator().await;
    let listed = create_user(&app, "listed", PLATFORM_MEMBER).await;
    assert_eq!(
        update_rows(
            &app,
            "user",
            &[
                ("email", LISTED_ADMINISTRATOR_EMAIL.into()),
                ("email_verified_at", now().into()),
            ],
            &[("id", listed.id.into())],
        )
        .await,
        1
    );
    let listed = crate::support::tenants::TestUser {
        email: LISTED_ADMINISTRATOR_EMAIL.to_string(),
        ..listed
    };

    let session = sign_in(&app, &listed).await;
    assert_eq!(
        platform_role_of_email(&app, LISTED_ADMINISTRATOR_EMAIL).await,
        PLATFORM_MEMBER,
        "a password proves nothing about the email"
    );
    assert!(platform_permissions(&session).await.is_empty());

    let started = start_connect(&session, SINGLE_SIGN_ON_PROVIDER).await;
    let callback_url = sign_in_at_provider(
        &started,
        "listed-subject",
        claims(LISTED_ADMINISTRATOR_EMAIL, true),
    )
    .await;
    let connected = complete_sign_in(&session, &callback_url).await;
    assert!(
        connected
            .location
            .starts_with(CONNECT_RESULT_LOCATION_PREFIX),
        "connect went to {}",
        connected.location
    );
    assert_eq!(
        platform_role_of_email(&app, LISTED_ADMINISTRATOR_EMAIL).await,
        PLATFORM_ADMINISTRATOR
    );
}
