use crate::support::canvas_builder::string_field;
use crate::support::constants::{
    ADMIN_ROLE, ALREADY_A_MEMBER_MESSAGE, ALREADY_INVITED_MESSAGE, EXPIRED_INVITATION_STATUS,
    INVALID_EMAIL_MESSAGE, INVITATION_EXPIRED_MESSAGE, INVITATION_NOT_FOUND_MESSAGE,
    INVITATION_TABLE, MEMBER_ROLE, OWNER_ROLE, PENDING_INVITATION_STATUS, PLATFORM_MEMBER,
    ROLE_TOO_POWERFUL_MESSAGE, SINGLE_SIGN_ON_REQUIRED_MESSAGE, UNKNOWN_ROLE_MESSAGE, VIEWER_ROLE,
};
use crate::support::database::{now, update_rows};
use crate::support::test_app::JsonResponse;
use crate::support::workspaces::{
    TestUser, WorkspaceMember, add_member, create_user, create_workspace, join_workspace, sign_in,
};
use crate::support::{TestApp, UserSession};
use chrono::Duration;
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

fn workspace_invitations_path(workspace: Uuid) -> String {
    format!("/tenant/{workspace}/invitation")
}

async fn invite(
    inviter: &WorkspaceMember,
    workspace: Uuid,
    email: &str,
    role_name: &str,
) -> JsonResponse {
    inviter
        .session
        .post(
            &workspace_invitations_path(workspace),
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

async fn revoke(owner: &WorkspaceMember, workspace: Uuid, invitation_id: Uuid) -> JsonResponse {
    owner
        .session
        .delete(&format!(
            "{}/{invitation_id}",
            workspace_invitations_path(workspace)
        ))
        .await
}

async fn accept(invitee: &UserSession, invitation_id: Uuid) -> JsonResponse {
    invitee
        .post(&format!("/invitation/{invitation_id}/accept"), json!({}))
        .await
}

async fn workspace_invitations(owner: &WorkspaceMember, workspace: Uuid) -> Vec<Value> {
    let invitations = owner
        .session
        .get(&workspace_invitations_path(workspace))
        .await;
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
    let workspace = create_workspace(&app, "acme").await;
    let owner = join_workspace(&app, workspace, "owner", OWNER_ROLE).await;
    let (invitee_user, invitee_session) = invitee(&app, "invitee").await;

    let invited = invite(&owner, workspace, &invitee_user.email, VIEWER_ROLE).await;
    assert_eq!(invited.body["status"], PENDING_INVITATION_STATUS);
    let invitation_id = invitation_id(invited);

    let accepted = accept(&invitee_session, invitation_id).await;
    assert_eq!(accepted.status, StatusCode::OK, "{}", accepted.body);
    assert_eq!(accepted.body["tenant_id"], workspace.to_string());

    let joined = memberships(&invitee_session).await;
    assert_eq!(joined.len(), 1);
    assert_eq!(joined[0]["tenant_id"], workspace.to_string());
    assert_eq!(joined[0]["role_name"], VIEWER_ROLE);
    assert!(
        workspace_invitations(&owner, workspace).await.is_empty(),
        "an accepted invitation is no longer pending"
    );
}

#[tokio::test]
async fn only_the_invited_email_can_accept() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let workspace = create_workspace(&app, "acme").await;
    let owner = join_workspace(&app, workspace, "owner", OWNER_ROLE).await;
    let (invitee_user, invitee_session) = invitee(&app, "invitee").await;
    let (_, stranger_session) = invitee(&app, "stranger").await;

    let invitation_id = invitation_id(
        invite(
            &owner,
            workspace,
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
    let workspace = create_workspace(&app, "acme").await;
    let owner = join_workspace(&app, workspace, "owner", OWNER_ROLE).await;
    let (invitee_user, invitee_session) = invitee(&app, "invitee").await;

    let revoked_id =
        invitation_id(invite(&owner, workspace, &invitee_user.email, MEMBER_ROLE).await);
    let revocation = revoke(&owner, workspace, revoked_id).await;
    assert!(revocation.status.is_success(), "{}", revocation.body);
    assert_refused(
        &accept(&invitee_session, revoked_id).await,
        StatusCode::NOT_FOUND,
        INVITATION_NOT_FOUND_MESSAGE,
    );
    assert_refused(
        &revoke(&owner, workspace, revoked_id).await,
        StatusCode::NOT_FOUND,
        INVITATION_NOT_FOUND_MESSAGE,
    );

    let expiring_id =
        invitation_id(invite(&owner, workspace, &invitee_user.email, MEMBER_ROLE).await);
    expire(&app, expiring_id).await;

    let listed = workspace_invitations(&owner, workspace).await;
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
    let workspace = create_workspace(&app, "acme").await;
    let owner = join_workspace(&app, workspace, "owner", OWNER_ROLE).await;
    let (invitee_user, invitee_session) = invitee(&app, "invitee").await;
    let invitation_id =
        invitation_id(invite(&owner, workspace, &invitee_user.email, MEMBER_ROLE).await);

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
    let workspace = create_workspace(&app, "acme").await;
    let owner = join_workspace(&app, workspace, "owner", OWNER_ROLE).await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;
    let newcomer_email = "newcomer@example.test";

    assert_refused(
        &invite(&owner, workspace, &member.user.email, VIEWER_ROLE).await,
        StatusCode::CONFLICT,
        ALREADY_A_MEMBER_MESSAGE,
    );

    let first_id = invitation_id(invite(&owner, workspace, newcomer_email, MEMBER_ROLE).await);
    assert_refused(
        &invite(&owner, workspace, " Newcomer@Example.test ", MEMBER_ROLE).await,
        StatusCode::CONFLICT,
        ALREADY_INVITED_MESSAGE,
    );

    assert!(
        revoke(&owner, workspace, first_id)
            .await
            .status
            .is_success()
    );
    let second_id = invitation_id(invite(&owner, workspace, newcomer_email, MEMBER_ROLE).await);
    expire(&app, second_id).await;
    assert_eq!(
        invite(&owner, workspace, newcomer_email, MEMBER_ROLE)
            .await
            .status,
        StatusCode::CREATED,
        "an expired invitation does not block a fresh one"
    );
}

#[tokio::test]
async fn nobody_invites_into_a_role_more_powerful_than_their_own() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let workspace = create_workspace(&app, "acme").await;
    join_workspace(&app, workspace, "owner", OWNER_ROLE).await;
    let admin = join_workspace(&app, workspace, "admin", ADMIN_ROLE).await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;

    assert_refused(
        &invite(&admin, workspace, "future.owner@example.test", OWNER_ROLE).await,
        StatusCode::FORBIDDEN,
        ROLE_TOO_POWERFUL_MESSAGE,
    );
    assert_eq!(
        invite(&admin, workspace, "future.admin@example.test", ADMIN_ROLE)
            .await
            .status,
        StatusCode::CREATED
    );
    let by_member = invite(&member, workspace, "friend@example.test", VIEWER_ROLE).await;
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
        &invite(&admin, workspace, "someone@example.test", "SUPERUSER").await,
        StatusCode::BAD_REQUEST,
        UNKNOWN_ROLE_MESSAGE,
    );
    assert_refused(
        &invite(&admin, workspace, "not-an-email", VIEWER_ROLE).await,
        StatusCode::BAD_REQUEST,
        INVALID_EMAIL_MESSAGE,
    );
}

#[tokio::test]
async fn received_invitations_list_only_what_the_caller_can_accept() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let first = create_workspace(&app, "first").await;
    let second = create_workspace(&app, "second").await;
    let third = create_workspace(&app, "third").await;
    let first_owner = join_workspace(&app, first, "first", OWNER_ROLE).await;
    let second_owner = join_workspace(&app, second, "second", OWNER_ROLE).await;
    let third_owner = join_workspace(&app, third, "third", OWNER_ROLE).await;
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
async fn inviting_requires_a_single_sign_on_provider() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "acme").await;
    let owner = join_workspace(&app, workspace, "owner", OWNER_ROLE).await;

    assert_refused(
        &invite(&owner, workspace, "newcomer@example.test", MEMBER_ROLE).await,
        StatusCode::CONFLICT,
        SINGLE_SIGN_ON_REQUIRED_MESSAGE,
    );
    assert!(workspace_invitations(&owner, workspace).await.is_empty());
}

#[tokio::test]
async fn removing_a_member_revokes_the_invitations_they_sent() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let workspace = create_workspace(&app, "acme").await;
    let owner = join_workspace(&app, workspace, "owner", OWNER_ROLE).await;
    let admin = join_workspace(&app, workspace, "admin", ADMIN_ROLE).await;
    let (accomplice_user, accomplice_session) = invitee(&app, "accomplice").await;

    let backdoor_id =
        invitation_id(invite(&admin, workspace, &accomplice_user.email, ADMIN_ROLE).await);
    let unrelated_id =
        invitation_id(invite(&owner, workspace, "friend@example.test", VIEWER_ROLE).await);

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
    let remaining: Vec<String> = workspace_invitations(&owner, workspace)
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
    let workspace = create_workspace(&app, "acme").await;
    let owner = join_workspace(&app, workspace, "owner", OWNER_ROLE).await;
    let (invitee_user, invitee_session) = invitee(&app, "invitee").await;
    let invitation_id =
        invitation_id(invite(&owner, workspace, &invitee_user.email, ADMIN_ROLE).await);
    add_member(&app, workspace, invitee_user.id, VIEWER_ROLE).await;

    let accepted = accept(&invitee_session, invitation_id).await;
    assert_eq!(accepted.status, StatusCode::OK, "{}", accepted.body);
    assert_eq!(accepted.body["tenant_id"], workspace.to_string());

    let joined = memberships(&invitee_session).await;
    assert_eq!(joined.len(), 1);
    assert_eq!(
        joined[0]["role_name"], VIEWER_ROLE,
        "an invitation never silently changes an existing member's role"
    );
    assert!(workspace_invitations(&owner, workspace).await.is_empty());
}
