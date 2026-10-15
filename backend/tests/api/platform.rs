use crate::support::TestApp;
use crate::support::canvas_builder::string_field;
use crate::support::constants::{OWNER_ROLE, PLATFORM_MEMBER, TENANT_CREATE_PERMISSION};
use crate::support::database::count_rows;
use crate::support::workspaces::{create_user, sign_in};
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn only_platform_administrators_create_workspaces() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let workspaces_before = count_rows(&app, "tenant", &[]).await;

    let member = sign_in(&app, &create_user(&app, "member", PLATFORM_MEMBER).await).await;
    let refused = member
        .post("/tenant", json!({ "name": "Side project" }))
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.body);
    assert_eq!(
        refused.body["error"],
        format!("Permission denied: {TENANT_CREATE_PERMISSION}")
    );
    assert_eq!(
        count_rows(&app, "tenant", &[]).await,
        workspaces_before,
        "a refused request creates nothing"
    );

    let created = app
        .post("/tenant", json!({ "name": "  Analytics  " }))
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    assert_eq!(created.body["name"], "Analytics");
    let created_id = string_field(&created.body, "id");
    let memberships = app.get("/tenant").await;
    let membership = memberships
        .body
        .as_array()
        .expect("memberships")
        .iter()
        .find(|membership| membership["tenant_id"] == created_id)
        .expect("the creator belongs to the new workspace")
        .clone();
    assert_eq!(membership["role_name"], OWNER_ROLE);

    assert_eq!(
        app.post("/tenant", json!({ "name": "   " })).await.status,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn current_user_reports_platform_permissions() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;

    let administrator_profile = app.get("/auth/me").await;
    assert_eq!(
        administrator_profile.body["platform_permissions"],
        json!([TENANT_CREATE_PERMISSION])
    );

    let member = sign_in(&app, &create_user(&app, "member", PLATFORM_MEMBER).await).await;
    let member_profile = member.get("/auth/me").await;
    assert_eq!(member_profile.body["platform_permissions"], json!([]));
}
