use crate::support::canvas_builder::string_field;
use crate::support::constants::{
    MEMBER_ROLE, ONE_MEMBER_LIMIT_MESSAGE, OWNER_ROLE, PLATFORM_MEMBER,
    TENANT_MEMBER_LIMIT_FOR_TESTS, TENANT_MEMBER_LIMIT_REACHED_CODE, THREE_MEMBER_LIMIT_MESSAGE,
};
use crate::support::tenants::{TenantMember, create_tenant, create_user, join_tenant, sign_in};
use crate::support::test_app::JsonResponse;
use crate::support::{DeploymentFlags, TestApp};
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

fn member_limit(tenant_member_limit: u64) -> DeploymentFlags {
    DeploymentFlags {
        tenant_member_limit: Some(tenant_member_limit),
        ..DeploymentFlags::default()
    }
}

fn tenant_invitations_path(tenant_id: Uuid) -> String {
    format!("/tenant/{tenant_id}/invitation")
}

async fn invite(inviter: &TenantMember, tenant_id: Uuid, email: &str) -> JsonResponse {
    inviter
        .session
        .post(
            &tenant_invitations_path(tenant_id),
            json!({ "email": email, "role_name": MEMBER_ROLE }),
        )
        .await
}

fn invitation_id(response: &JsonResponse) -> String {
    assert_eq!(response.status, StatusCode::CREATED, "{}", response.body);
    string_field(&response.body, "id")
}

fn assert_limit_reached(response: &JsonResponse, message: &str) {
    assert_eq!(response.status, StatusCode::CONFLICT, "{}", response.body);
    assert_eq!(
        response.body["error_code"], TENANT_MEMBER_LIMIT_REACHED_CODE,
        "{}",
        response.body
    );
    assert_eq!(response.body["error"], message, "{}", response.body);
}

#[tokio::test]
async fn the_invitation_that_would_exceed_the_limit_is_refused() {
    let app =
        TestApp::spawn_with_deployment_flags(member_limit(TENANT_MEMBER_LIMIT_FOR_TESTS)).await;
    app.login_as_admin().await;
    let tenant_id = create_tenant(&app, "Small team").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    join_tenant(&app, tenant_id, "member", MEMBER_ROLE).await;

    let last_place = invite(&owner, tenant_id, "third@example.test").await;
    let last_invitation_id = invitation_id(&last_place);

    let refused = invite(&owner, tenant_id, "fourth@example.test").await;
    assert_limit_reached(&refused, THREE_MEMBER_LIMIT_MESSAGE);
    let invitations = owner.session.get(&tenant_invitations_path(tenant_id)).await;
    assert_eq!(
        invitations.body.as_array().expect("invitations").len(),
        1,
        "a refused invitation is not stored: {}",
        invitations.body
    );

    let revoked = owner
        .session
        .delete(&format!(
            "{}/{last_invitation_id}",
            tenant_invitations_path(tenant_id)
        ))
        .await;
    assert!(revoked.status.is_success(), "{}", revoked.body);
    assert_eq!(
        invite(&owner, tenant_id, "fourth@example.test")
            .await
            .status,
        StatusCode::CREATED,
        "revoking an invitation frees a place"
    );
}

#[tokio::test]
async fn accepting_a_counted_invitation_works_at_the_limit() {
    let app =
        TestApp::spawn_with_deployment_flags(member_limit(TENANT_MEMBER_LIMIT_FOR_TESTS)).await;
    app.login_as_admin().await;
    let tenant_id = create_tenant(&app, "Full team").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    join_tenant(&app, tenant_id, "member", MEMBER_ROLE).await;
    let invitee = create_user(&app, "invitee", PLATFORM_MEMBER).await;
    let invitee_session = sign_in(&app, &invitee).await;
    let invitation = invite(&owner, tenant_id, &invitee.email).await;
    let counted_invitation_id = invitation_id(&invitation);

    let accepted = invitee_session
        .post(
            &format!("/invitation/{counted_invitation_id}/accept"),
            json!({}),
        )
        .await;

    assert!(accepted.status.is_success(), "{}", accepted.body);
    assert_eq!(
        invitee_session
            .in_tenant(tenant_id)
            .get("/canvas")
            .await
            .status,
        StatusCode::OK
    );
    assert_limit_reached(
        &invite(&owner, tenant_id, "one-too-many@example.test").await,
        THREE_MEMBER_LIMIT_MESSAGE,
    );
}

#[tokio::test]
async fn a_platform_administrator_can_appoint_an_owner_past_the_limit() {
    let app = TestApp::spawn_with_deployment_flags(member_limit(1)).await;
    app.login_as_admin().await;
    let tenant_id = create_tenant(&app, "Solo").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    assert_limit_reached(
        &invite(&owner, tenant_id, "second@example.test").await,
        ONE_MEMBER_LIMIT_MESSAGE,
    );
    let appointee = create_user(&app, "appointee", PLATFORM_MEMBER).await;

    let appointed = app
        .put(
            &format!("/platform/tenant/{tenant_id}/owner/{}", appointee.id),
            json!({}),
        )
        .await;

    assert_eq!(
        appointed.status,
        StatusCode::NO_CONTENT,
        "{}",
        appointed.body
    );
    assert_eq!(
        owner
            .session
            .get("/member")
            .await
            .body
            .as_array()
            .expect("members")
            .len(),
        2
    );
}

#[tokio::test]
async fn without_a_limit_any_number_of_people_can_be_invited() {
    let app = TestApp::spawn_with_single_sign_on().await;
    app.login_as_admin().await;
    let tenant_id = create_tenant(&app, "Open team").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;

    for invitee_number in 0..(TENANT_MEMBER_LIMIT_FOR_TESTS * 2) {
        let invited = invite(
            &owner,
            tenant_id,
            &format!(
                "invitee-{invitee_number}-{}@example.test",
                Uuid::new_v4().simple()
            ),
        )
        .await;
        assert_eq!(invited.status, StatusCode::CREATED, "{}", invited.body);
    }
}
