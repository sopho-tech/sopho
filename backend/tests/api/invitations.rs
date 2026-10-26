use crate::support::canvas_builder::string_field;
use crate::support::constants::{
    ADMIN_ROLE, ALREADY_A_MEMBER_MESSAGE, ALREADY_INVITED_MESSAGE, EXPIRED_INVITATION_STATUS,
    INVALID_EMAIL_MESSAGE, INVITATION_EXPIRED_MESSAGE, INVITATION_NOT_FOUND_MESSAGE,
    INVITATION_TABLE, MEMBER_ROLE, OWNER_ROLE, PENDING_INVITATION_STATUS, PLATFORM_MEMBER,
    ROLE_TOO_POWERFUL_MESSAGE, UNKNOWN_ROLE_MESSAGE, VIEWER_ROLE,
};
use crate::support::constants::{
    EMAIL_CANCELLED_STATUS, EMAIL_OUTBOX_TABLE, EMAIL_PENDING_STATUS, EMAIL_SENT_STATUS,
    INVITATION_LINK_PATH, INVITATIONS_UNAVAILABLE_MESSAGE, RESEND_NEEDS_EMAIL_MESSAGE,
};
use crate::support::database::{now, select_rows, select_uuid, update_rows};
use crate::support::mail_server::{link_in, shared_mail_server};
use crate::support::tenants::{
    TenantMember, TestUser, add_member, create_tenant, create_user, join_tenant, sign_in,
};
use crate::support::test_app::JsonResponse;
use crate::support::{TestApp, UserSession};
use chrono::Duration;
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

fn tenant_invitations_path(tenant_id: Uuid) -> String {
    format!("/tenant/{tenant_id}/invitation")
}

async fn invite(
    inviter: &TenantMember,
    tenant_id: Uuid,
    email: &str,
    role_name: &str,
) -> JsonResponse {
    inviter
        .session
        .post(
            &tenant_invitations_path(tenant_id),
            json!({ "email": email, "role_name": role_name }),
        )
        .await
}

fn invitation_id(response: JsonResponse) -> Uuid {
    assert_eq!(response.status, StatusCode::CREATED, "{}", response.body);
    string_field(&response.body, "id")
        .parse()
        .expect("the invitation id is a UUID")
}

async fn revoke(owner: &TenantMember, tenant_id: Uuid, invitation_id: Uuid) -> JsonResponse {
    owner
        .session
        .delete(&format!(
            "{}/{invitation_id}",
            tenant_invitations_path(tenant_id)
        ))
        .await
}

async fn accept(invitee: &UserSession, invitation_id: Uuid) -> JsonResponse {
    invitee
        .post(&format!("/invitation/{invitation_id}/accept"), json!({}))
        .await
}

async fn tenant_invitations(owner: &TenantMember, tenant_id: Uuid) -> Vec<Value> {
    let invitations = owner.session.get(&tenant_invitations_path(tenant_id)).await;
    assert_eq!(invitations.status, StatusCode::OK, "{}", invitations.body);
    invitations.body.as_array().expect("invitations").clone()
}

async fn received_invitations(invitee: &UserSession) -> Vec<Value> {
    let received = invitee.get("/invitation").await;
    assert_eq!(received.status, StatusCode::OK, "{}", received.body);
    received.body.as_array().expect("invitations").clone()
}

async fn memberships(session: &UserSession) -> Vec<Value> {
    let memberships = session.get("/tenant").await;
    assert_eq!(memberships.status, StatusCode::OK, "{}", memberships.body);
    memberships.body.as_array().expect("memberships").clone()
}

async fn expire(app: &TestApp, invitation_id: Uuid) {
    let expired = update_rows(
        app,
        INVITATION_TABLE,
        &[("expires_at", (now() - Duration::days(1)).into())],
        &[("id", invitation_id.into())],
    )
    .await;
    assert_eq!(expired, 1);
}

fn assert_refused(response: &JsonResponse, status: StatusCode, message: &str) {
    assert_eq!(response.status, status, "{}", response.body);
    assert_eq!(response.body["error"], message);
}

async fn invitee(app: &TestApp, label: &str) -> (TestUser, UserSession) {
    let user = create_user(app, label, PLATFORM_MEMBER).await;
    let session = sign_in(app, &user).await;
    (user, session)
}

#[tokio::test]
async fn an_invitation_is_accepted_into_the_invited_role() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let (invitee_user, invitee_session) = invitee(&app, "invitee").await;

    let invited = invite(&owner, tenant_id, &invitee_user.email, VIEWER_ROLE).await;
    assert_eq!(invited.body["status"], PENDING_INVITATION_STATUS);
    let invitation_id = invitation_id(invited);

    let accepted = accept(&invitee_session, invitation_id).await;
    assert_eq!(accepted.status, StatusCode::OK, "{}", accepted.body);
    assert_eq!(accepted.body["tenant_id"], tenant_id.to_string());

    let joined = memberships(&invitee_session).await;
    assert_eq!(joined.len(), 1);
    assert_eq!(joined[0]["tenant_id"], tenant_id.to_string());
    assert_eq!(joined[0]["role_name"], VIEWER_ROLE);
    assert!(
        tenant_invitations(&owner, tenant_id).await.is_empty(),
        "an accepted invitation is no longer pending"
    );
}

#[tokio::test]
async fn only_the_invited_email_can_accept() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let (invitee_user, invitee_session) = invitee(&app, "invitee").await;
    let (_, stranger_session) = invitee(&app, "stranger").await;

    let invitation_id = invitation_id(
        invite(
            &owner,
            tenant_id,
            &invitee_user.email.to_uppercase(),
            MEMBER_ROLE,
        )
        .await,
    );

    assert_refused(
        &accept(&stranger_session, invitation_id).await,
        StatusCode::NOT_FOUND,
        INVITATION_NOT_FOUND_MESSAGE,
    );
    assert_eq!(
        accept(&invitee_session, invitation_id).await.status,
        StatusCode::OK,
        "addresses compare case-insensitively"
    );
}

#[tokio::test]
async fn expired_and_revoked_invitations_cannot_be_accepted() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let (invitee_user, invitee_session) = invitee(&app, "invitee").await;

    let revoked_id =
        invitation_id(invite(&owner, tenant_id, &invitee_user.email, MEMBER_ROLE).await);
    let revocation = revoke(&owner, tenant_id, revoked_id).await;
    assert!(revocation.status.is_success(), "{}", revocation.body);
    assert_refused(
        &accept(&invitee_session, revoked_id).await,
        StatusCode::NOT_FOUND,
        INVITATION_NOT_FOUND_MESSAGE,
    );
    assert_refused(
        &revoke(&owner, tenant_id, revoked_id).await,
        StatusCode::NOT_FOUND,
        INVITATION_NOT_FOUND_MESSAGE,
    );

    let expiring_id =
        invitation_id(invite(&owner, tenant_id, &invitee_user.email, MEMBER_ROLE).await);
    expire(&app, expiring_id).await;

    let listed = tenant_invitations(&owner, tenant_id).await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["status"], EXPIRED_INVITATION_STATUS);
    assert_refused(
        &accept(&invitee_session, expiring_id).await,
        StatusCode::CONFLICT,
        INVITATION_EXPIRED_MESSAGE,
    );
    assert!(received_invitations(&invitee_session).await.is_empty());
}

#[tokio::test]
async fn an_invitation_cannot_be_accepted_twice() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let (invitee_user, invitee_session) = invitee(&app, "invitee").await;
    let invitation_id =
        invitation_id(invite(&owner, tenant_id, &invitee_user.email, MEMBER_ROLE).await);

    assert_eq!(
        accept(&invitee_session, invitation_id).await.status,
        StatusCode::OK
    );
    assert_refused(
        &accept(&invitee_session, invitation_id).await,
        StatusCode::NOT_FOUND,
        INVITATION_NOT_FOUND_MESSAGE,
    );
}

#[tokio::test]
async fn inviting_a_member_or_a_pending_email_conflicts() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let member = join_tenant(&app, tenant_id, "member", MEMBER_ROLE).await;
    let newcomer_email = "newcomer@example.test";

    assert_refused(
        &invite(&owner, tenant_id, &member.user.email, VIEWER_ROLE).await,
        StatusCode::CONFLICT,
        ALREADY_A_MEMBER_MESSAGE,
    );

    let first_id = invitation_id(invite(&owner, tenant_id, newcomer_email, MEMBER_ROLE).await);
    assert_refused(
        &invite(&owner, tenant_id, " Newcomer@Example.test ", MEMBER_ROLE).await,
        StatusCode::CONFLICT,
        ALREADY_INVITED_MESSAGE,
    );

    assert!(
        revoke(&owner, tenant_id, first_id)
            .await
            .status
            .is_success()
    );
    let second_id = invitation_id(invite(&owner, tenant_id, newcomer_email, MEMBER_ROLE).await);
    expire(&app, second_id).await;
    assert_eq!(
        invite(&owner, tenant_id, newcomer_email, MEMBER_ROLE)
            .await
            .status,
        StatusCode::CREATED,
        "an expired invitation does not block a fresh one"
    );
}

#[tokio::test]
async fn nobody_invites_into_a_role_more_powerful_than_their_own() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let tenant_id = create_tenant(&app, "acme").await;
    join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let admin = join_tenant(&app, tenant_id, "admin", ADMIN_ROLE).await;
    let member = join_tenant(&app, tenant_id, "member", MEMBER_ROLE).await;

    assert_refused(
        &invite(&admin, tenant_id, "future.owner@example.test", OWNER_ROLE).await,
        StatusCode::FORBIDDEN,
        ROLE_TOO_POWERFUL_MESSAGE,
    );
    assert_eq!(
        invite(&admin, tenant_id, "future.admin@example.test", ADMIN_ROLE)
            .await
            .status,
        StatusCode::CREATED
    );
    let by_member = invite(&member, tenant_id, "friend@example.test", VIEWER_ROLE).await;
    assert_eq!(
        by_member.status,
        StatusCode::FORBIDDEN,
        "{}",
        by_member.body
    );
    assert!(
        by_member.body["error"]
            .as_str()
            .expect("error message")
            .starts_with("Permission denied: "),
        "a member fails the capability gate: {}",
        by_member.body
    );
    assert_refused(
        &invite(&admin, tenant_id, "someone@example.test", "SUPERUSER").await,
        StatusCode::BAD_REQUEST,
        UNKNOWN_ROLE_MESSAGE,
    );
    assert_refused(
        &invite(&admin, tenant_id, "not-an-email", VIEWER_ROLE).await,
        StatusCode::BAD_REQUEST,
        INVALID_EMAIL_MESSAGE,
    );
}

#[tokio::test]
async fn received_invitations_list_only_what_the_caller_can_accept() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let first = create_tenant(&app, "first").await;
    let second = create_tenant(&app, "second").await;
    let third = create_tenant(&app, "third").await;
    let first_owner = join_tenant(&app, first, "first", OWNER_ROLE).await;
    let second_owner = join_tenant(&app, second, "second", OWNER_ROLE).await;
    let third_owner = join_tenant(&app, third, "third", OWNER_ROLE).await;
    let (invitee_user, invitee_session) = invitee(&app, "invitee").await;

    let pending_id =
        invitation_id(invite(&first_owner, first, &invitee_user.email, MEMBER_ROLE).await);
    let revoked_id =
        invitation_id(invite(&second_owner, second, &invitee_user.email, MEMBER_ROLE).await);
    assert!(
        revoke(&second_owner, second, revoked_id)
            .await
            .status
            .is_success()
    );
    let expired_id =
        invitation_id(invite(&third_owner, third, &invitee_user.email, MEMBER_ROLE).await);
    expire(&app, expired_id).await;
    invitation_id(
        invite(
            &first_owner,
            first,
            "someone.else@example.test",
            MEMBER_ROLE,
        )
        .await,
    );

    let received = received_invitations(&invitee_session).await;
    assert_eq!(received.len(), 1);
    assert_eq!(received[0]["id"], pending_id.to_string());
    assert_eq!(received[0]["tenant_id"], first.to_string());
    assert_eq!(received[0]["tenant_name"], "first");
    assert_eq!(received[0]["role_name"], MEMBER_ROLE);
    assert_eq!(received[0]["invited_by_name"], first_owner.user.full_name);
}

#[tokio::test]
async fn inviting_needs_email_or_single_sign_on() {
    let app = TestApp::spawn().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;

    assert_refused(
        &invite(&owner, tenant_id, "newcomer@example.test", MEMBER_ROLE).await,
        StatusCode::CONFLICT,
        INVITATIONS_UNAVAILABLE_MESSAGE,
    );
    assert!(tenant_invitations(&owner, tenant_id).await.is_empty());
}

fn unique_recipient(label: &str) -> String {
    format!("{label}-{}@example.test", Uuid::new_v4().simple())
}

async fn stored_token_hash(app: &TestApp, invitation_id: Uuid) -> Option<String> {
    select_rows(
        app,
        INVITATION_TABLE,
        &["token_hash"],
        &[("id", invitation_id.into())],
    )
    .await[0]
        .try_get("", "token_hash")
        .expect("read the token hash")
}

async fn email_status(app: &TestApp, email_id: Uuid) -> String {
    select_rows(
        app,
        EMAIL_OUTBOX_TABLE,
        &["status"],
        &[("id", email_id.into())],
    )
    .await[0]
        .try_get("", "status")
        .expect("read the email status")
}

fn token_in(link: &str) -> &str {
    link.split_once("#token=")
        .map(|(_, token)| token)
        .expect("the link carries a token")
}

#[tokio::test]
async fn inviting_with_email_on_emails_a_link() {
    let app = TestApp::spawn_with_email().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let recipient = unique_recipient("emailed");

    let response = invite(&owner, tenant_id, &recipient, MEMBER_ROLE).await;
    assert_eq!(
        response.body["email_delivery"]["status"],
        EMAIL_PENDING_STATUS
    );
    let invitation_id = invitation_id(response);
    app.deliver_due_emails().await;

    let received = shared_mail_server().await.emails_to(&recipient).await;
    assert_eq!(received.len(), 1);
    assert_eq!(
        received[0].subject,
        format!("{} invited you to acme on Sopho", owner.user.full_name)
    );
    let link = link_in(
        &received[0],
        &format!("{}{INVITATION_LINK_PATH}", app.base_url()),
    );
    let token = token_in(&link);
    assert_eq!(token.len(), 43);
    assert_eq!(
        stored_token_hash(&app, invitation_id).await,
        Some(hex::encode(<sha2::Sha256 as sha2::Digest>::digest(
            token.as_bytes()
        )))
    );
}

#[tokio::test]
async fn resending_replaces_the_link_and_cancels_the_unsent_email() {
    let app = TestApp::spawn_with_email().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let recipient = unique_recipient("resent");
    let invitation_id = invitation_id(invite(&owner, tenant_id, &recipient, MEMBER_ROLE).await);
    let first_email_id = select_uuid(
        &app,
        INVITATION_TABLE,
        "latest_email_id",
        &[("id", invitation_id.into())],
    )
    .await;
    let first_token_hash = stored_token_hash(&app, invitation_id).await;
    expire(&app, invitation_id).await;

    let resent = owner
        .session
        .post(
            &format!(
                "{}/{invitation_id}/resend",
                tenant_invitations_path(tenant_id)
            ),
            json!({}),
        )
        .await;
    assert_eq!(resent.status, StatusCode::OK, "{}", resent.body);
    assert_eq!(resent.body["status"], PENDING_INVITATION_STATUS);
    assert_eq!(
        resent.body["email_delivery"]["status"],
        EMAIL_PENDING_STATUS
    );

    assert_eq!(
        email_status(&app, first_email_id).await,
        EMAIL_CANCELLED_STATUS
    );
    let second_email_id = select_uuid(
        &app,
        INVITATION_TABLE,
        "latest_email_id",
        &[("id", invitation_id.into())],
    )
    .await;
    assert_ne!(second_email_id, first_email_id);
    let second_token_hash = stored_token_hash(&app, invitation_id).await;
    assert_ne!(second_token_hash, first_token_hash);

    app.deliver_due_emails().await;
    let received = shared_mail_server().await.emails_to(&recipient).await;
    assert_eq!(received.len(), 1, "only the resent email goes out");
    let link = link_in(
        &received[0],
        &format!("{}{INVITATION_LINK_PATH}", app.base_url()),
    );
    assert_eq!(
        second_token_hash,
        Some(hex::encode(<sha2::Sha256 as sha2::Digest>::digest(
            token_in(&link).as_bytes()
        )))
    );
    let pending = tenant_invitations(&owner, tenant_id).await;
    assert_eq!(pending[0]["status"], PENDING_INVITATION_STATUS);
}

#[tokio::test]
async fn resending_needs_email() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let invitation_id =
        invitation_id(invite(&owner, tenant_id, "newcomer@example.test", MEMBER_ROLE).await);

    assert_refused(
        &owner
            .session
            .post(
                &format!(
                    "{}/{invitation_id}/resend",
                    tenant_invitations_path(tenant_id)
                ),
                json!({}),
            )
            .await,
        StatusCode::CONFLICT,
        RESEND_NEEDS_EMAIL_MESSAGE,
    );
    assert_eq!(stored_token_hash(&app, invitation_id).await, None);
}

#[tokio::test]
async fn revoking_cancels_the_unsent_email() {
    let app = TestApp::spawn_with_email().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let recipient = unique_recipient("revoked");
    let invitation_id = invitation_id(invite(&owner, tenant_id, &recipient, MEMBER_ROLE).await);
    let email_id = select_uuid(
        &app,
        INVITATION_TABLE,
        "latest_email_id",
        &[("id", invitation_id.into())],
    )
    .await;

    let revoked = revoke(&owner, tenant_id, invitation_id).await;
    assert_eq!(revoked.status, StatusCode::NO_CONTENT, "{}", revoked.body);

    assert_eq!(email_status(&app, email_id).await, EMAIL_CANCELLED_STATUS);
    app.deliver_due_emails().await;
    assert!(
        shared_mail_server()
            .await
            .emails_to(&recipient)
            .await
            .is_empty()
    );
}

async fn delivery_method(app: &TestApp) -> Value {
    let tenant_id = create_tenant(app, "acme").await;
    let owner = join_tenant(app, tenant_id, "owner", OWNER_ROLE).await;
    let viewer = join_tenant(app, tenant_id, "viewer", VIEWER_ROLE).await;
    let delivery_path = format!("{}/delivery", tenant_invitations_path(tenant_id));
    assert_eq!(
        viewer.session.get(&delivery_path).await.status,
        StatusCode::FORBIDDEN
    );
    let delivery = owner.session.get(&delivery_path).await;
    assert_eq!(delivery.status, StatusCode::OK, "{}", delivery.body);
    delivery.body
}

#[tokio::test]
async fn the_delivery_method_follows_the_configuration() {
    let with_email = delivery_method(&TestApp::spawn_with_email().await).await;
    assert_eq!(with_email["method"], "EMAIL");
    assert_eq!(with_email["unavailable_reason"], Value::Null);

    let with_both = delivery_method(&TestApp::spawn_with_email_and_single_sign_on().await).await;
    assert_eq!(with_both["method"], "EMAIL");

    let with_single_sign_on = delivery_method(&TestApp::spawn_with_single_sign_on().await).await;
    assert_eq!(with_single_sign_on["method"], "SINGLE_SIGN_ON");

    let with_neither = delivery_method(&TestApp::spawn().await).await;
    assert_eq!(with_neither["method"], "UNAVAILABLE");
    assert_eq!(
        with_neither["unavailable_reason"],
        INVITATIONS_UNAVAILABLE_MESSAGE
    );
}

#[tokio::test]
async fn pending_invitations_show_their_email_status() {
    let app = TestApp::spawn_with_email().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    invite(&owner, tenant_id, &unique_recipient("status"), MEMBER_ROLE).await;

    let queued = tenant_invitations(&owner, tenant_id).await;
    assert_eq!(queued[0]["email_delivery"]["status"], EMAIL_PENDING_STATUS);

    app.deliver_due_emails().await;
    let sent = tenant_invitations(&owner, tenant_id).await;
    assert_eq!(sent[0]["email_delivery"]["status"], EMAIL_SENT_STATUS);
    assert_eq!(sent[0]["email_delivery"]["last_error"], Value::Null);
}

#[tokio::test]
async fn removing_a_member_revokes_the_invitations_they_sent() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let admin = join_tenant(&app, tenant_id, "admin", ADMIN_ROLE).await;
    let (accomplice_user, accomplice_session) = invitee(&app, "accomplice").await;

    let backdoor_id =
        invitation_id(invite(&admin, tenant_id, &accomplice_user.email, ADMIN_ROLE).await);
    let unrelated_id =
        invitation_id(invite(&owner, tenant_id, "friend@example.test", VIEWER_ROLE).await);

    let removal = owner
        .session
        .delete(&format!("/member/{}", admin.user.id))
        .await;
    assert!(removal.status.is_success(), "{}", removal.body);

    let backdoor = accept(&accomplice_session, backdoor_id).await;
    assert_eq!(
        backdoor.status,
        StatusCode::NOT_FOUND,
        "a removed member's invitations cannot bring anyone back: {}",
        backdoor.body
    );
    assert_eq!(backdoor.body["error"], INVITATION_NOT_FOUND_MESSAGE);
    let remaining: Vec<String> = tenant_invitations(&owner, tenant_id)
        .await
        .iter()
        .map(|invitation| string_field(invitation, "id"))
        .collect();
    assert_eq!(
        remaining,
        vec![unrelated_id.to_string()],
        "invitations sent by others stay pending"
    );
}

#[tokio::test]
async fn accepting_while_already_a_member_keeps_the_existing_role() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let (invitee_user, invitee_session) = invitee(&app, "invitee").await;
    let invitation_id =
        invitation_id(invite(&owner, tenant_id, &invitee_user.email, ADMIN_ROLE).await);
    add_member(&app, tenant_id, invitee_user.id, VIEWER_ROLE).await;

    let accepted = accept(&invitee_session, invitation_id).await;
    assert_eq!(accepted.status, StatusCode::OK, "{}", accepted.body);
    assert_eq!(accepted.body["tenant_id"], tenant_id.to_string());

    let joined = memberships(&invitee_session).await;
    assert_eq!(joined.len(), 1);
    assert_eq!(
        joined[0]["role_name"], VIEWER_ROLE,
        "an invitation never silently changes an existing member's role"
    );
    assert!(tenant_invitations(&owner, tenant_id).await.is_empty());
}
