use crate::authentication::error::CredentialsError;
use crate::authentication::oidc::dto::{CallbackQuery, LoginSecrets, VerifiedClaims};
use crate::authentication::oidc::error::OidcLoginError;
use crate::authentication::oidc::service as oidc_service;
use crate::authorization::constants::PlatformRole;
use crate::common::AppState;
use crate::common::time_utils;
use crate::entity;
use crate::tests::common::app_state::init_test_app_state;
use crate::tests::common::tenancy::insert_user;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use std::collections::HashMap;

const TEST_ADMIN_EMAIL: &str = "admin@test.local";
const TEST_ADMIN_PASSWORD: &str = "test_password";
const TEST_PROVIDER: &str = "google";

fn claims(
    subject: &str,
    email: Option<&str>,
    email_verified: bool,
    full_name: Option<&str>,
) -> VerifiedClaims {
    VerifiedClaims {
        provider_name: TEST_PROVIDER.to_string(),
        subject: subject.to_string(),
        email: email.map(str::to_string),
        email_verified,
        full_name: full_name.map(str::to_string),
    }
}

async fn user_count(app_state: &AppState) -> u64 {
    entity::user::Entity::find()
        .count(&app_state.database_connection)
        .await
        .unwrap()
}

async fn identity_count(app_state: &AppState) -> u64 {
    entity::identity::Entity::find()
        .count(&app_state.database_connection)
        .await
        .unwrap()
}

async fn record_attempt(app_state: &AppState, state: &str) {
    oidc_service::record_login_attempt(
        app_state,
        TEST_PROVIDER,
        LoginSecrets {
            state: state.to_string(),
            nonce: format!("nonce-{state}"),
            code_verifier: format!("verifier-{state}"),
        },
    )
    .await
    .unwrap();
}

async fn expire_attempt(app_state: &AppState, state: &str) {
    let attempt = entity::oidc_login_attempt::Entity::find()
        .filter(entity::oidc_login_attempt::Column::State.eq(state))
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap();
    let mut expiring: entity::oidc_login_attempt::ActiveModel = attempt.into();
    expiring.expires_at = Set(time_utils::now_plus_minutes_into(-1));
    expiring
        .update(&app_state.database_connection)
        .await
        .unwrap();
}

#[tokio::test]
async fn password_login_is_refused_for_an_sso_only_user() {
    let app_state = init_test_app_state("sign_in_sso_only_password").await;
    crate::initialize(&app_state).await.unwrap();

    let administrator = crate::authentication::service::verify_credentials(
        &app_state,
        " Admin@Test.Local ",
        TEST_ADMIN_PASSWORD,
    )
    .await
    .unwrap();
    assert_eq!(
        administrator.platform_role,
        PlatformRole::Administrator.as_str(),
        "the seeded admin administers the platform"
    );
    assert!(matches!(
        crate::authentication::service::verify_credentials(&app_state, TEST_ADMIN_EMAIL, "wrong")
            .await,
        Err(CredentialsError::InvalidCredentials)
    ));

    let sso_user_id = insert_user(&app_state, "sso").await;
    let sso_user = entity::user::Entity::find_by_id(sso_user_id)
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap();
    assert!(sso_user.password_hash.is_none());
    assert!(matches!(
        crate::authentication::service::verify_credentials(&app_state, &sso_user.email, "").await,
        Err(CredentialsError::InvalidCredentials)
    ));
}

#[tokio::test]
async fn a_returning_identity_signs_in_as_the_same_user() {
    let app_state = init_test_app_state("sign_in_returning_identity").await;

    let first = oidc_service::resolve_user(
        &app_state,
        claims(
            "subject-1",
            Some("returning@example.test"),
            true,
            Some("Returning User"),
        ),
    )
    .await
    .unwrap();
    let second = oidc_service::resolve_user(
        &app_state,
        claims("subject-1", Some("changed@example.test"), false, None),
    )
    .await
    .unwrap();

    assert_eq!(
        first, second,
        "the identity, not the email, decides who signs in"
    );
    assert_eq!(user_count(&app_state).await, 1);
    assert_eq!(identity_count(&app_state).await, 1);
}

#[tokio::test]
async fn a_verified_email_never_links_to_a_password_account() {
    let app_state = init_test_app_state("sign_in_password_account").await;
    crate::initialize(&app_state).await.unwrap();
    let users_before = user_count(&app_state).await;

    assert!(matches!(
        oidc_service::resolve_user(
            &app_state,
            claims(
                "google-admin",
                Some("ADMIN@Test.local"),
                true,
                Some("Admin")
            ),
        )
        .await,
        Err(OidcLoginError::AccountRequiresPasswordSignIn)
    ));
    assert_eq!(
        identity_count(&app_state).await,
        0,
        "an account whose email no identity provider verified is never taken over"
    );
    assert_eq!(user_count(&app_state).await, users_before);
    assert!(
        crate::authentication::service::verify_credentials(
            &app_state,
            TEST_ADMIN_EMAIL,
            TEST_ADMIN_PASSWORD
        )
        .await
        .is_ok()
    );
}

#[tokio::test]
async fn a_verified_email_links_accounts_created_through_single_sign_on() {
    let app_state = init_test_app_state("sign_in_link_across_providers").await;
    let first = oidc_service::resolve_user(
        &app_state,
        claims(
            "google-subject",
            Some("shared@example.test"),
            true,
            Some("Shared"),
        ),
    )
    .await
    .unwrap();

    let linked = oidc_service::resolve_user(
        &app_state,
        VerifiedClaims {
            provider_name: "another-provider".to_string(),
            subject: "another-subject".to_string(),
            email: Some("Shared@Example.test".to_string()),
            email_verified: true,
            full_name: Some("Shared".to_string()),
        },
    )
    .await
    .unwrap();

    assert_eq!(linked, first);
    assert_eq!(user_count(&app_state).await, 1);
    assert_eq!(identity_count(&app_state).await, 2);
}

#[tokio::test]
async fn an_unverified_email_is_refused_and_creates_nothing() {
    let app_state = init_test_app_state("sign_in_unverified_email").await;
    crate::initialize(&app_state).await.unwrap();
    let users_before = user_count(&app_state).await;

    assert!(matches!(
        oidc_service::resolve_user(
            &app_state,
            claims("stranger", Some("new@example.test"), false, Some("New"))
        )
        .await,
        Err(OidcLoginError::EmailNotVerified)
    ));
    assert!(
        matches!(
            oidc_service::resolve_user(
                &app_state,
                claims("impostor", Some(TEST_ADMIN_EMAIL), false, Some("Impostor"))
            )
            .await,
            Err(OidcLoginError::EmailNotVerified)
        ),
        "an unverified email never links to an existing account"
    );
    assert!(matches!(
        oidc_service::resolve_user(&app_state, claims("no-email", None, true, Some("Nobody")))
            .await,
        Err(OidcLoginError::MissingEmail)
    ));

    assert_eq!(user_count(&app_state).await, users_before);
    assert_eq!(identity_count(&app_state).await, 0);
}

#[tokio::test]
async fn a_new_verified_identity_creates_a_member_without_a_password() {
    let app_state = init_test_app_state("sign_in_new_identity").await;

    assert!(matches!(
        oidc_service::resolve_user(
            &app_state,
            claims("nameless", Some("nameless@example.test"), true, None)
        )
        .await,
        Err(OidcLoginError::MissingProfile)
    ));
    assert_eq!(user_count(&app_state).await, 0);

    let user_id = oidc_service::resolve_user(
        &app_state,
        claims(
            "newcomer",
            Some(" Newcomer@Example.test "),
            true,
            Some("New Comer"),
        ),
    )
    .await
    .unwrap();
    let user = entity::user::Entity::find_by_id(user_id)
        .one(&app_state.database_connection)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.email, "newcomer@example.test");
    assert_eq!(user.username, "newcomer@example.test");
    assert_eq!(user.full_name, "New Comer");
    assert_eq!(user.platform_role, PlatformRole::Member.as_str());
    assert!(user.password_hash.is_none());
}

#[tokio::test]
async fn a_login_attempt_is_consumed_once() {
    let app_state = init_test_app_state("sign_in_attempt_single_use").await;
    record_attempt(&app_state, "first").await;

    let attempt = oidc_service::consume_attempt(&app_state, TEST_PROVIDER, "first")
        .await
        .unwrap();
    assert_eq!(attempt.nonce, "nonce-first");
    assert_eq!(attempt.code_verifier, "verifier-first");
    assert!(matches!(
        oidc_service::consume_attempt(&app_state, TEST_PROVIDER, "first").await,
        Err(OidcLoginError::AttemptNotFound)
    ));

    record_attempt(&app_state, "second").await;
    assert!(matches!(
        oidc_service::consume_attempt(&app_state, "another-provider", "second").await,
        Err(OidcLoginError::AttemptNotFound)
    ));
    assert!(
        matches!(
            oidc_service::consume_attempt(&app_state, TEST_PROVIDER, "second").await,
            Err(OidcLoginError::AttemptNotFound)
        ),
        "a callback for the wrong provider still spends the attempt"
    );
}

#[tokio::test]
async fn an_expired_login_attempt_is_refused() {
    let app_state = init_test_app_state("sign_in_attempt_expiry").await;
    record_attempt(&app_state, "late").await;
    expire_attempt(&app_state, "late").await;

    assert!(matches!(
        oidc_service::consume_attempt(&app_state, TEST_PROVIDER, "late").await,
        Err(OidcLoginError::AttemptExpired)
    ));
    assert!(matches!(
        oidc_service::consume_attempt(&app_state, TEST_PROVIDER, "late").await,
        Err(OidcLoginError::AttemptNotFound)
    ));

    record_attempt(&app_state, "abandoned").await;
    expire_attempt(&app_state, "abandoned").await;
    record_attempt(&app_state, "fresh").await;
    let remaining_states: Vec<String> = entity::oidc_login_attempt::Entity::find()
        .all(&app_state.database_connection)
        .await
        .unwrap()
        .into_iter()
        .map(|attempt| attempt.state)
        .collect();
    assert_eq!(
        remaining_states,
        vec!["fresh".to_string()],
        "starting a login purges abandoned attempts"
    );
}

#[test]
fn a_callback_without_the_starting_cookie_is_refused() {
    assert!(matches!(
        oidc_service::require_matching_state(None, "state"),
        Err(OidcLoginError::StateMismatch)
    ));
    assert!(matches!(
        oidc_service::require_matching_state(Some("other"), "state"),
        Err(OidcLoginError::StateMismatch)
    ));
    assert!(matches!(
        oidc_service::require_matching_state(Some("stat"), "state"),
        Err(OidcLoginError::StateMismatch)
    ));
    assert!(oidc_service::require_matching_state(Some("state"), "state").is_ok());
}

#[test]
fn configured_providers_require_every_variable() {
    let complete: HashMap<&str, &str> = [
        ("OIDC_PROVIDERS", " Google "),
        ("OIDC_GOOGLE_DISPLAY_NAME", "Google"),
        ("OIDC_GOOGLE_ISSUER_URL", "https://accounts.google.com"),
        ("OIDC_GOOGLE_CLIENT_ID", "client-id"),
        ("OIDC_GOOGLE_CLIENT_SECRET", "client-secret"),
        (
            "OIDC_GOOGLE_REDIRECT_URI",
            "https://app.example/api/v1/auth/oidc/google/callback",
        ),
    ]
    .into_iter()
    .collect();
    let read = |variables: HashMap<&'static str, &'static str>| {
        move |variable: &str| variables.get(variable).map(|value| value.to_string())
    };

    let providers = crate::common::oidc_providers_from_variables(read(complete.clone())).unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].name, "google");
    assert_eq!(providers[0].client_secret, "client-secret");

    let mut missing_secret = complete.clone();
    missing_secret.remove("OIDC_GOOGLE_CLIENT_SECRET");
    let error = crate::common::oidc_providers_from_variables(read(missing_secret)).unwrap_err();
    assert!(error.to_string().contains("OIDC_GOOGLE_CLIENT_SECRET"));

    let mut unsafe_name = complete.clone();
    unsafe_name.insert("OIDC_PROVIDERS", "goo gle");
    assert!(crate::common::oidc_providers_from_variables(read(unsafe_name)).is_err());

    assert!(
        crate::common::oidc_providers_from_variables(|_| None)
            .unwrap()
            .is_empty(),
        "without OIDC_PROVIDERS only password sign-in is offered"
    );
}

#[tokio::test]
async fn a_callback_without_the_starting_cookie_leaves_the_attempt_for_its_browser() {
    let app_state = init_test_app_state("sign_in_callback_without_cookie").await;
    record_attempt(&app_state, "victim").await;

    let forged = oidc_service::complete_login(
        &app_state,
        &tower_cookies::Cookies::default(),
        TEST_PROVIDER,
        CallbackQuery {
            code: Some("attacker-code".to_string()),
            state: Some("victim".to_string()),
            error: None,
        },
        None,
    )
    .await;

    assert!(matches!(forged, Err(OidcLoginError::StateMismatch)));
    assert!(
        oidc_service::consume_attempt(&app_state, TEST_PROVIDER, "victim")
            .await
            .is_ok(),
        "a forged callback must not spend the real browser's attempt"
    );
}
