use crate::support::constants::{
    CONTACT_ADMINISTRATOR, EMAIL_OUTBOX_TABLE, INVITATION_LINK_PATH, LAST_SIGN_IN_METHOD_CODE,
    LISTED_ADMINISTRATOR_EMAIL, MEMBER_ROLE, NEW_ACCOUNT_PASSWORD, OTHER_SINGLE_SIGN_ON_PROVIDER,
    PASSWORD_SIGN_IN_OFF_CODE, PASSWORD_SIGN_IN_OFF_MESSAGE, PLATFORM_MEMBER,
    RETIRED_SINGLE_SIGN_ON_PROVIDER, SINGLE_SIGN_ON_PROVIDER, TEST_USER_PASSWORD,
};
use crate::support::database::{count_rows, insert_row, now, select_rows};
use crate::support::identity_provider::sign_in_with_provider;
use crate::support::mail_server::{link_in, shared_mail_server};
use crate::support::tenants::{TestUser, create_user};
use crate::support::test_app::JsonResponse;
use crate::support::{DeploymentFlags, TestApp, UserSession};
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

fn verified_claims(email: &str, full_name: &str) -> Value {
    json!({ "email": email, "email_verified": true, "name": full_name })
}

fn unique_email(label: &str) -> String {
    format!("{label}-{}@example.test", Uuid::new_v4().simple())
}

async fn spawn_passwordless() -> TestApp {
    TestApp::spawn_with_deployment_flags(DeploymentFlags::passwordless(LISTED_ADMINISTRATOR_EMAIL))
        .await
}

async fn signed_in_through_provider(app: &TestApp, email: &str, full_name: &str) -> UserSession {
    let session = app.session(None);
    let outcome = sign_in_with_provider(
        &session,
        SINGLE_SIGN_ON_PROVIDER,
        &format!("subject-{}", Uuid::new_v4().simple()),
        verified_claims(email, full_name),
    )
    .await;
    assert!(
        outcome.is_signed_in(),
        "sign-in went to {}",
        outcome.location
    );
    session
}

async fn signed_in_user_id(session: &UserSession) -> Uuid {
    let profile = session.get("/auth/me").await;
    assert_eq!(profile.status, StatusCode::OK, "{}", profile.body);
    profile.body["id"]
        .as_str()
        .expect("the signed-in user id")
        .parse()
        .expect("the user id is a UUID")
}

fn assert_password_sign_in_is_off(response: &JsonResponse, request: &str) {
    assert_eq!(
        response.status,
        StatusCode::FORBIDDEN,
        "{request}: {}",
        response.body
    );
    assert_eq!(
        response.body["error_code"], PASSWORD_SIGN_IN_OFF_CODE,
        "{request}"
    );
    assert_eq!(
        response.body["message"], PASSWORD_SIGN_IN_OFF_MESSAGE,
        "{request}"
    );
}

async fn password_hash_of(app: &TestApp, user: &TestUser) -> Option<String> {
    select_rows(app, "user", &["password_hash"], &[("id", user.id.into())])
        .await
        .first()
        .expect("the user row")
        .try_get("", "password_hash")
        .expect("read the password hash")
}

async fn add_federated_identity(app: &TestApp, user_id: Uuid, provider_name: &str, email: &str) {
    insert_row(
        app,
        "federated_identity",
        &[
            ("id", Uuid::new_v4().into()),
            ("user_id", user_id.into()),
            ("provider_name", provider_name.into()),
            (
                "subject",
                format!("subject-{}", Uuid::new_v4().simple()).into(),
            ),
            ("email", email.into()),
            ("email_verified", true.into()),
            ("created_at", now().into()),
            ("updated_at", now().into()),
        ],
    )
    .await;
}

fn provider_named<'a>(sign_in_methods: &'a Value, provider_name: &str) -> &'a Value {
    sign_in_methods["providers"]
        .as_array()
        .expect("the providers")
        .iter()
        .find(|provider| provider["name"] == provider_name)
        .unwrap_or_else(|| panic!("no provider named {provider_name}"))
}

#[tokio::test]
async fn every_password_request_is_refused_when_password_sign_in_is_off() {
    let app = spawn_passwordless().await;
    let password_account = create_user(&app, "password-account", PLATFORM_MEMBER).await;
    let stored_password_hash = password_hash_of(&app, &password_account).await;
    let unknown_email = unique_email("nobody");
    let visitor = app.session(None);

    for email in [password_account.email.as_str(), unknown_email.as_str()] {
        assert_password_sign_in_is_off(
            &visitor.sign_in(email, TEST_USER_PASSWORD).await,
            "sign in",
        );
        assert_password_sign_in_is_off(
            &visitor
                .post("/auth/password_reset_request", json!({ "email": email }))
                .await,
            "request a reset",
        );
    }
    assert_password_sign_in_is_off(
        &visitor
            .post(
                "/auth/password_reset",
                json!({ "token": "any-token", "new_password": NEW_ACCOUNT_PASSWORD }),
            )
            .await,
        "reset with a token",
    );
    assert_password_sign_in_is_off(
        &visitor
            .post(
                "/invitation_link/sign_up",
                json!({
                    "token": "any-token",
                    "full_name": "New Person",
                    "password": NEW_ACCOUNT_PASSWORD,
                }),
            )
            .await,
        "sign up from an invitation link",
    );

    let member = signed_in_through_provider(&app, &unique_email("member"), "A Member").await;
    assert_password_sign_in_is_off(
        &member
            .put(
                "/auth/password",
                json!({
                    "current_password": TEST_USER_PASSWORD,
                    "new_password": NEW_ACCOUNT_PASSWORD,
                }),
            )
            .await,
        "change a password",
    );

    let administrator =
        signed_in_through_provider(&app, LISTED_ADMINISTRATOR_EMAIL, "The Founder").await;
    assert_password_sign_in_is_off(
        &administrator
            .post(
                &format!("/platform/user/{}/password_reset", password_account.id),
                json!({}),
            )
            .await,
        "send a reset as a platform administrator",
    );

    assert_eq!(
        password_hash_of(&app, &password_account).await,
        stored_password_hash,
        "the stored password is left untouched"
    );
    assert_eq!(count_rows(&app, "password_reset_token", &[]).await, 0);
    assert_eq!(count_rows(&app, EMAIL_OUTBOX_TABLE, &[]).await, 0);
    assert_eq!(
        count_rows(&app, "session", &[("user_id", password_account.id.into())]).await,
        0
    );
}

#[tokio::test]
async fn sign_in_options_report_passwords_unavailable() {
    let app = spawn_passwordless().await;
    let options = app.session(None).get("/auth/sign_in_page_option").await;
    assert_eq!(options.status, StatusCode::OK, "{}", options.body);
    assert_eq!(options.body["password_sign_in_available"], false);
    assert_eq!(
        options.body["password_reset_available"], false,
        "email is configured, but a reset is useless without passwords"
    );
    assert_eq!(options.body["providers"].as_array().map(Vec::len), Some(2));

    let with_passwords = TestApp::spawn_with_email().await;
    let options = with_passwords
        .session(None)
        .get("/auth/sign_in_page_option")
        .await;
    assert_eq!(options.body["password_sign_in_available"], true);
    assert_eq!(options.body["password_reset_available"], true);
}

#[tokio::test]
async fn a_stored_password_or_a_retired_provider_is_not_another_way_to_sign_in() {
    let app = spawn_passwordless().await;
    let email = unique_email("member");
    let member = signed_in_through_provider(&app, &email, "A Member").await;
    let member_id = signed_in_user_id(&member).await;
    insert_password_hash(&app, member_id).await;
    add_federated_identity(&app, member_id, RETIRED_SINGLE_SIGN_ON_PROVIDER, &email).await;

    let sign_in_methods = member.get("/auth/my_sign_in_method").await;
    assert_eq!(
        sign_in_methods.status,
        StatusCode::OK,
        "{}",
        sign_in_methods.body
    );
    assert_eq!(sign_in_methods.body["password_available"], false);
    assert_eq!(sign_in_methods.body["has_password"], true);
    let only_configured_provider = provider_named(&sign_in_methods.body, SINGLE_SIGN_ON_PROVIDER);
    assert_eq!(only_configured_provider["can_disconnect"], false);
    assert!(only_configured_provider["disconnect_blocked_reason"].is_string());
    assert_eq!(
        provider_named(&sign_in_methods.body, RETIRED_SINGLE_SIGN_ON_PROVIDER)["can_disconnect"],
        true,
        "a retired provider can be dropped while a configured one remains"
    );

    let refused = member
        .delete(&format!(
            "/auth/oidc/{SINGLE_SIGN_ON_PROVIDER}/federated_identity"
        ))
        .await;
    assert_eq!(refused.status, StatusCode::CONFLICT, "{}", refused.body);
    assert_eq!(refused.body["error_code"], LAST_SIGN_IN_METHOD_CODE);

    add_federated_identity(&app, member_id, OTHER_SINGLE_SIGN_ON_PROVIDER, &email).await;
    let sign_in_methods = member.get("/auth/my_sign_in_method").await;
    assert_eq!(
        provider_named(&sign_in_methods.body, SINGLE_SIGN_ON_PROVIDER)["can_disconnect"],
        true,
        "a second configured provider is another way to sign in"
    );
}

async fn insert_password_hash(app: &TestApp, user_id: Uuid) {
    let borrowed_password_hash =
        password_hash_of(app, &create_user(app, "hash-donor", PLATFORM_MEMBER).await).await;
    assert_eq!(
        crate::support::database::update_rows(
            app,
            "user",
            &[("password_hash", borrowed_password_hash.into())],
            &[("id", user_id.into())],
        )
        .await,
        1
    );
}

#[tokio::test]
async fn an_account_without_a_proven_email_is_told_to_contact_an_administrator() {
    let app = spawn_passwordless().await;
    let password_account = create_user(&app, "unproven", PLATFORM_MEMBER).await;

    let outcome = sign_in_with_provider(
        &app.session(None),
        SINGLE_SIGN_ON_PROVIDER,
        "unproven-subject",
        verified_claims(&password_account.email, "Unproven Person"),
    )
    .await;

    assert_eq!(outcome.error_code(), Some(CONTACT_ADMINISTRATOR));
    assert_eq!(
        count_rows(
            &app,
            "federated_identity",
            &[("user_id", password_account.id.into())]
        )
        .await,
        0
    );
}

#[tokio::test]
async fn an_invited_person_joins_through_the_provider() {
    let app = spawn_passwordless().await;
    let owner = signed_in_through_provider(&app, LISTED_ADMINISTRATOR_EMAIL, "The Founder").await;
    let created = owner.post("/tenant", json!({ "name": "Acme" })).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    let tenant_id = created.body["id"].as_str().expect("the tenant id");
    let invited_email = unique_email("invited");
    let invited = owner
        .post(
            &format!("/tenant/{tenant_id}/invitation"),
            json!({ "email": invited_email, "role_name": MEMBER_ROLE }),
        )
        .await;
    assert_eq!(invited.status, StatusCode::CREATED, "{}", invited.body);
    app.deliver_due_emails().await;
    let received = shared_mail_server().await.emails_to(&invited_email).await;
    let link = link_in(
        received.last().expect("an invitation email"),
        &format!("{}{INVITATION_LINK_PATH}", app.base_url()),
    );
    let token = link
        .split_once("#token=")
        .map(|(_, token)| token.to_string())
        .expect("the link carries a token");

    let visitor = app.session(None);
    let found = visitor
        .post("/invitation_link/lookup", json!({ "token": token }))
        .await;
    assert_eq!(found.status, StatusCode::OK, "{}", found.body);
    assert_eq!(found.body["password_sign_up_available"], false);
    assert_password_sign_in_is_off(
        &visitor
            .post(
                "/invitation_link/sign_up",
                json!({
                    "token": token,
                    "full_name": "Invited Person",
                    "password": NEW_ACCOUNT_PASSWORD,
                }),
            )
            .await,
        "sign up from the real invitation link",
    );

    let invited_person = signed_in_through_provider(&app, &invited_email, "Invited Person").await;
    let accepted = invited_person
        .post("/invitation_link/accept", json!({ "token": token }))
        .await;
    assert_eq!(accepted.status, StatusCode::OK, "{}", accepted.body);
    assert_eq!(accepted.body["tenant_id"], tenant_id);
    let memberships = invited_person.get("/tenant").await;
    assert_eq!(
        memberships.body.as_array().map(Vec::len),
        Some(1),
        "{}",
        memberships.body
    );
    assert_eq!(memberships.body[0]["role_name"], MEMBER_ROLE);
}

#[tokio::test]
async fn the_platform_user_list_reports_passwords_unavailable() {
    let app = spawn_passwordless().await;
    create_user(&app, "password-account", PLATFORM_MEMBER).await;
    let administrator =
        signed_in_through_provider(&app, LISTED_ADMINISTRATOR_EMAIL, "The Founder").await;

    let users = administrator.get("/platform/user").await;
    assert_eq!(users.status, StatusCode::OK, "{}", users.body);
    let listed_users = users.body["users"].as_array().expect("the users");
    assert!(
        listed_users.iter().any(|user| user["has_password"] == true),
        "the stored password is still reported"
    );
    for user in listed_users {
        assert_eq!(user["can_sign_in_with_password"], false, "{user}");
        assert_eq!(
            user["capabilities"]["can_send_password_reset"], false,
            "{user}"
        );
    }
}
