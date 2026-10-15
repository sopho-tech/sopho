use crate::support::constants::{
    ADMIN_ROLE, CANVAS_RESOURCE, LAST_OWNER_MESSAGE, MEMBER_NOT_FOUND_MESSAGE, MEMBER_ROLE,
    MEMBER_TOO_POWERFUL_MESSAGE, OWNER_ROLE, PRIVATE_VISIBILITY, ROLE_TOO_POWERFUL_MESSAGE,
    VIEWER_ACCESS, VIEWER_ROLE,
};
use crate::support::records::insert_canvas;
use crate::support::test_app::JsonResponse;
use crate::support::workspaces::{create_workspace, join_workspace};
use crate::support::{TestApp, UserSession};
use reqwest::StatusCode;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

async fn change_role(session: &UserSession, user_id: Uuid, role_name: &str) -> JsonResponse {
    session
        .put(
            &format!("/member/{user_id}"),
            json!({ "role_name": role_name }),
        )
        .await
}

async fn remove_member(session: &UserSession, user_id: Uuid) -> JsonResponse {
    session.delete(&format!("/member/{user_id}")).await
}

async fn capabilities_seen_by(session: &UserSession) -> HashMap<String, (bool, bool)> {
    let members = session.get("/member").await;
    assert_eq!(members.status, StatusCode::OK, "{}", members.body);
    members
        .body
        .as_array()
        .expect("members")
        .iter()
        .map(|member| {
            (
                member["user_id"]
                    .as_str()
                    .expect("member user id")
                    .to_string(),
                (
                    member["capabilities"]["can_change_role"]
                        .as_bool()
                        .expect("can_change_role"),
                    member["capabilities"]["can_remove"]
                        .as_bool()
                        .expect("can_remove"),
                ),
            )
        })
        .collect()
}

fn assert_refused(response: &JsonResponse, status: StatusCode, message: &str) {
    assert_eq!(response.status, status, "{}", response.body);
    assert_eq!(response.body["error"], message);
}

#[tokio::test]
async fn an_admin_cannot_promote_anyone_to_owner() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "promotion").await;
    join_workspace(&app, workspace, "owner", OWNER_ROLE).await;
    let admin = join_workspace(&app, workspace, "admin", ADMIN_ROLE).await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;

    assert_refused(
        &change_role(&admin.session, member.user.id, OWNER_ROLE).await,
        StatusCode::FORBIDDEN,
        ROLE_TOO_POWERFUL_MESSAGE,
    );
    assert_refused(
        &change_role(&admin.session, admin.user.id, OWNER_ROLE).await,
        StatusCode::FORBIDDEN,
        ROLE_TOO_POWERFUL_MESSAGE,
    );

    let demoted = change_role(&admin.session, member.user.id, VIEWER_ROLE).await;
    assert_eq!(demoted.status, StatusCode::OK, "{}", demoted.body);
    assert_eq!(demoted.body["role_name"], VIEWER_ROLE);
}

#[tokio::test]
async fn an_admin_cannot_change_or_remove_an_owner() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "owner_protection").await;
    let owner = join_workspace(&app, workspace, "owner", OWNER_ROLE).await;
    let admin = join_workspace(&app, workspace, "admin", ADMIN_ROLE).await;
    let fellow_admin = join_workspace(&app, workspace, "fellow", ADMIN_ROLE).await;

    assert_refused(
        &change_role(&admin.session, owner.user.id, MEMBER_ROLE).await,
        StatusCode::FORBIDDEN,
        MEMBER_TOO_POWERFUL_MESSAGE,
    );
    assert_refused(
        &remove_member(&admin.session, owner.user.id).await,
        StatusCode::FORBIDDEN,
        MEMBER_TOO_POWERFUL_MESSAGE,
    );
    let removal = remove_member(&admin.session, fellow_admin.user.id).await;
    assert!(
        removal.status.is_success(),
        "an admin manages fellow admins: {} {}",
        removal.status,
        removal.body
    );
}

#[tokio::test]
async fn the_last_owner_cannot_be_demoted_or_removed() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "last_owner").await;
    let owner = join_workspace(&app, workspace, "owner", OWNER_ROLE).await;

    assert_refused(
        &change_role(&owner.session, owner.user.id, ADMIN_ROLE).await,
        StatusCode::CONFLICT,
        LAST_OWNER_MESSAGE,
    );
    assert_refused(
        &remove_member(&owner.session, owner.user.id).await,
        StatusCode::CONFLICT,
        LAST_OWNER_MESSAGE,
    );

    let successor = join_workspace(&app, workspace, "successor", MEMBER_ROLE).await;
    let promoted = change_role(&owner.session, successor.user.id, OWNER_ROLE).await;
    assert_eq!(promoted.status, StatusCode::OK, "{}", promoted.body);
    let stepped_down = change_role(&owner.session, owner.user.id, ADMIN_ROLE).await;
    assert_eq!(stepped_down.status, StatusCode::OK, "{}", stepped_down.body);
    assert_eq!(
        stepped_down.body["role_name"], ADMIN_ROLE,
        "a second owner frees the first to step down"
    );
}

#[tokio::test]
async fn removed_member_loses_access_immediately() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "removal").await;
    let owner = join_workspace(&app, workspace, "owner", OWNER_ROLE).await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;
    let canvas_id = insert_canvas(
        &app,
        workspace,
        "private",
        owner.user.id,
        PRIVATE_VISIBILITY,
    )
    .await;
    let granted = owner
        .session
        .post(
            "/grant",
            json!({
                "resource_type": CANVAS_RESOURCE,
                "resource_id": canvas_id,
                "principal_user_id": member.user.id,
                "access_level": VIEWER_ACCESS,
            }),
        )
        .await;
    assert_eq!(granted.status, StatusCode::CREATED, "{}", granted.body);

    let removal = remove_member(&owner.session, member.user.id).await;
    assert!(removal.status.is_success(), "{}", removal.body);

    assert_eq!(
        member.session.get("/canvas").await.status,
        StatusCode::NOT_FOUND,
        "a removed member must lose the workspace on the next request"
    );
    let sharing = owner
        .session
        .get(&format!("/grant/{CANVAS_RESOURCE}/{canvas_id}"))
        .await;
    assert_eq!(sharing.status, StatusCode::OK, "{}", sharing.body);
    assert_eq!(
        sharing.body["grants"],
        json!([]),
        "the removed member's grants go with them"
    );
    assert_refused(
        &remove_member(&owner.session, member.user.id).await,
        StatusCode::NOT_FOUND,
        MEMBER_NOT_FOUND_MESSAGE,
    );
}

#[tokio::test]
async fn member_capabilities_match_what_the_rules_allow() {
    let app = TestApp::spawn().await;
    let workspace = create_workspace(&app, "capabilities").await;
    let owner = join_workspace(&app, workspace, "owner", OWNER_ROLE).await;
    let admin = join_workspace(&app, workspace, "admin", ADMIN_ROLE).await;
    let member = join_workspace(&app, workspace, "member", MEMBER_ROLE).await;
    let owner_id = owner.user.id.to_string();
    let admin_id = admin.user.id.to_string();
    let member_id = member.user.id.to_string();

    let seen_by_admin = capabilities_seen_by(&admin.session).await;
    assert_eq!(seen_by_admin[&owner_id], (false, false));
    assert_eq!(seen_by_admin[&admin_id], (true, true));
    assert_eq!(seen_by_admin[&member_id], (true, true));

    let seen_by_member = capabilities_seen_by(&member.session).await;
    assert!(
        seen_by_member
            .values()
            .all(|capabilities| *capabilities == (false, false))
    );
    let refused = change_role(&member.session, admin.user.id, VIEWER_ROLE).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.body);
    assert!(
        refused.body["error"]
            .as_str()
            .expect("error message")
            .starts_with("Permission denied: "),
        "a member fails the capability gate: {}",
        refused.body
    );

    let seen_by_owner = capabilities_seen_by(&owner.session).await;
    assert_eq!(
        seen_by_owner[&owner_id],
        (false, false),
        "the last owner cannot be changed, even by themselves"
    );
    assert_eq!(seen_by_owner[&admin_id], (true, true));

    let co_owner = join_workspace(&app, workspace, "co_owner", MEMBER_ROLE).await;
    let promoted = change_role(&owner.session, co_owner.user.id, OWNER_ROLE).await;
    assert_eq!(promoted.status, StatusCode::OK, "{}", promoted.body);
    assert_eq!(
        capabilities_seen_by(&admin.session).await[&owner_id],
        (false, false),
        "an owner stays out of an admin's reach even when not the last one"
    );
    assert_eq!(
        capabilities_seen_by(&owner.session).await[&owner_id],
        (true, true),
        "with a co-owner, an owner may step down"
    );

    let roles_for_admin = admin.session.get("/role").await;
    assert_eq!(
        roles_for_admin.status,
        StatusCode::OK,
        "{}",
        roles_for_admin.body
    );
    let assignable_for_admin: Vec<(String, bool)> = roles_for_admin
        .body
        .as_array()
        .expect("roles")
        .iter()
        .map(|role| {
            (
                role["name"].as_str().expect("role name").to_string(),
                role["assignable"].as_bool().expect("assignable"),
            )
        })
        .collect();
    assert_eq!(
        assignable_for_admin,
        vec![
            (OWNER_ROLE.to_string(), false),
            (ADMIN_ROLE.to_string(), true),
            (MEMBER_ROLE.to_string(), true),
            (VIEWER_ROLE.to_string(), true),
        ]
    );
    let roles_for_owner = owner.session.get("/role").await;
    assert!(
        roles_for_owner
            .body
            .as_array()
            .expect("roles")
            .iter()
            .all(|role| role["assignable"] == true)
    );
}
