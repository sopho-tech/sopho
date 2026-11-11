use crate::support::TestApp;
use crate::support::canvas_builder::string_field;
use crate::support::constants::{OWNER_ROLE, SUSPENDED_STATUS, TENANT_VISIBILITY};
use crate::support::records::{
    insert_canvas, insert_conversation, insert_dashboard, insert_dashboard_summary,
    insert_notebook, insert_sql_cell, insert_suggested_question,
};
use crate::support::tenants::{
    TenantMember, add_member, create_tenant, join_tenant, set_membership_status,
};
use reqwest::StatusCode;
use serde_json::Value;
use uuid::Uuid;

struct TwoTenants {
    first: Uuid,
    second: Uuid,
    first_owner: TenantMember,
    second_owner: TenantMember,
}

async fn two_tenants(app: &TestApp) -> TwoTenants {
    let first = create_tenant(app, "first").await;
    let second = create_tenant(app, "second").await;
    TwoTenants {
        first_owner: join_tenant(app, first, "first-owner", OWNER_ROLE).await,
        second_owner: join_tenant(app, second, "second-owner", OWNER_ROLE).await,
        first,
        second,
    }
}

async fn assert_readable_only_in_its_tenant(tenants: &TwoTenants, path: &str) {
    let in_own_tenant = tenants.first_owner.session.get(path).await;
    assert_eq!(
        in_own_tenant.status,
        StatusCode::OK,
        "{path}: {}",
        in_own_tenant.body
    );
    assert_eq!(
        tenants.second_owner.session.get(path).await.status,
        StatusCode::NOT_FOUND,
        "{path} must not be readable from another tenant"
    );
}

fn canvas_names(page: &Value) -> Vec<String> {
    page["data"]
        .as_array()
        .expect("canvas page data")
        .iter()
        .map(|canvas| string_field(canvas, "name"))
        .collect()
}

#[tokio::test]
async fn canvas_of_another_tenant_is_not_readable() {
    let app = TestApp::spawn().await;
    let tenants = two_tenants(&app).await;
    let canvas_id = insert_canvas(
        &app,
        tenants.first,
        "First canvas",
        Uuid::new_v4(),
        TENANT_VISIBILITY,
    )
    .await;

    assert_readable_only_in_its_tenant(&tenants, &format!("/canvas/{canvas_id}")).await;
}

#[tokio::test]
async fn canvas_list_excludes_other_tenants() {
    let app = TestApp::spawn().await;
    let tenants = two_tenants(&app).await;
    insert_canvas(
        &app,
        tenants.first,
        "first tenant canvas",
        Uuid::new_v4(),
        TENANT_VISIBILITY,
    )
    .await;
    insert_canvas(
        &app,
        tenants.second,
        "second tenant canvas",
        Uuid::new_v4(),
        TENANT_VISIBILITY,
    )
    .await;

    let page = tenants
        .first_owner
        .session
        .get("/canvas?page=0&page_size=50")
        .await;
    assert_eq!(page.status, StatusCode::OK, "{}", page.body);
    assert_eq!(
        page.body["total_items"], 1,
        "the list must count only the requesting tenant"
    );
    assert_eq!(canvas_names(&page.body), vec!["first tenant canvas"]);
}

#[tokio::test]
async fn notebook_of_another_tenant_is_not_readable() {
    let app = TestApp::spawn().await;
    let tenants = two_tenants(&app).await;
    let canvas_id = insert_canvas(
        &app,
        tenants.first,
        "First canvas",
        Uuid::new_v4(),
        TENANT_VISIBILITY,
    )
    .await;
    let notebook_id = insert_notebook(&app, tenants.first, canvas_id, "Notebook").await;

    assert_readable_only_in_its_tenant(&tenants, &format!("/notebook/{notebook_id}")).await;
}

#[tokio::test]
async fn cell_of_another_tenant_is_not_readable() {
    let app = TestApp::spawn().await;
    let tenants = two_tenants(&app).await;
    let canvas_id = insert_canvas(
        &app,
        tenants.first,
        "First canvas",
        Uuid::new_v4(),
        TENANT_VISIBILITY,
    )
    .await;
    let notebook_id = insert_notebook(&app, tenants.first, canvas_id, "Notebook").await;
    let cell_id = insert_sql_cell(&app, tenants.first, notebook_id, "First cell", "SELECT 1").await;

    assert_readable_only_in_its_tenant(&tenants, &format!("/cell/{cell_id}")).await;
}

#[tokio::test]
async fn dashboard_of_another_tenant_is_not_readable() {
    let app = TestApp::spawn().await;
    let tenants = two_tenants(&app).await;
    let canvas_id = insert_canvas(
        &app,
        tenants.first,
        "First canvas",
        Uuid::new_v4(),
        TENANT_VISIBILITY,
    )
    .await;
    let dashboard_id = insert_dashboard(&app, tenants.first, canvas_id, "Dashboard").await;

    assert_readable_only_in_its_tenant(&tenants, &format!("/dashboard/{dashboard_id}")).await;
}

#[tokio::test]
async fn connection_of_another_tenant_is_not_readable() {
    let app = TestApp::spawn().await;
    let tenants = two_tenants(&app).await;
    let connection = app
        .create_sqlite_demo_connection_as(&tenants.first_owner.session)
        .await;

    assert_readable_only_in_its_tenant(
        &tenants,
        &format!("/connection/{}", string_field(&connection, "id")),
    )
    .await;
}

#[tokio::test]
async fn conversation_of_another_tenant_is_not_readable() {
    let app = TestApp::spawn().await;
    let tenants = two_tenants(&app).await;
    let connection = app
        .create_sqlite_demo_connection_as(&tenants.first_owner.session)
        .await;
    let conversation_id = insert_conversation(
        &app,
        tenants.first,
        "Conversation",
        Uuid::new_v4(),
        TENANT_VISIBILITY,
        string_field(&connection, "id")
            .parse()
            .expect("connection id"),
    )
    .await;

    assert_readable_only_in_its_tenant(
        &tenants,
        &format!("/conversational_analytics/{conversation_id}"),
    )
    .await;
}

#[tokio::test]
async fn suggested_question_of_another_tenant_is_not_visible() {
    let app = TestApp::spawn().await;
    let tenants = two_tenants(&app).await;
    let connection = app
        .create_sqlite_demo_connection_as(&tenants.first_owner.session)
        .await;
    let connection_id: Uuid = string_field(&connection, "id")
        .parse()
        .expect("connection id");
    insert_suggested_question(&app, tenants.first, connection_id, "How many orders?").await;
    let path = format!("/suggested_question/connection/{connection_id}");

    let owner_view = tenants.first_owner.session.get(&path).await;
    assert_eq!(owner_view.status, StatusCode::OK, "{}", owner_view.body);
    assert_eq!(owner_view.body.as_array().expect("questions").len(), 1);
    assert_eq!(
        tenants.second_owner.session.get(&path).await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn ai_summary_of_another_tenant_is_not_visible() {
    let app = TestApp::spawn().await;
    let tenants = two_tenants(&app).await;
    let canvas_id = insert_canvas(
        &app,
        tenants.first,
        "First canvas",
        Uuid::new_v4(),
        TENANT_VISIBILITY,
    )
    .await;
    let dashboard_id = insert_dashboard(&app, tenants.first, canvas_id, "Dashboard").await;
    insert_dashboard_summary(&app, tenants.first, dashboard_id, "secret summary").await;
    let path = format!("/ai_summary/dashboard/{dashboard_id}");

    let owner_view = tenants.first_owner.session.get(&path).await;
    assert_eq!(owner_view.status, StatusCode::OK, "{}", owner_view.body);
    assert_eq!(
        owner_view.body["dashboard"]["summary_text"],
        "secret summary"
    );
    assert_eq!(
        tenants.second_owner.session.get(&path).await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn non_member_cannot_resolve_tenant_context() {
    let app = TestApp::spawn().await;
    let tenants = two_tenants(&app).await;

    let response = tenants
        .second_owner
        .session
        .in_tenant(tenants.first)
        .get("/canvas")
        .await;

    assert_eq!(response.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn suspended_member_cannot_resolve_tenant_context() {
    let app = TestApp::spawn().await;
    let tenant_id = create_tenant(&app, "suspended").await;
    let member = join_tenant(&app, tenant_id, "suspended", OWNER_ROLE).await;
    set_membership_status(&app, tenant_id, member.user.id, SUSPENDED_STATUS).await;

    let response = member.session.get("/canvas").await;

    assert_eq!(response.status, StatusCode::FORBIDDEN);
    assert_eq!(response.body["message"], "Membership suspended");
}

#[tokio::test]
async fn user_belonging_to_two_tenants_is_scoped_to_each_separately() {
    let app = TestApp::spawn().await;
    let tenants = two_tenants(&app).await;
    add_member(
        &app,
        tenants.second,
        tenants.first_owner.user.id,
        OWNER_ROLE,
    )
    .await;
    insert_canvas(
        &app,
        tenants.first,
        "first canvas",
        Uuid::new_v4(),
        TENANT_VISIBILITY,
    )
    .await;
    insert_canvas(
        &app,
        tenants.second,
        "second canvas",
        Uuid::new_v4(),
        TENANT_VISIBILITY,
    )
    .await;

    for (tenant_id, expected_name) in [
        (tenants.first, "first canvas"),
        (tenants.second, "second canvas"),
    ] {
        let page = tenants
            .first_owner
            .session
            .in_tenant(tenant_id)
            .get("/canvas?page=0&page_size=50")
            .await;
        assert_eq!(page.status, StatusCode::OK, "{}", page.body);
        assert_eq!(page.body["total_items"], 1);
        assert_eq!(canvas_names(&page.body), vec![expected_name]);
    }
}

#[tokio::test]
async fn tenant_context_carries_the_roles_permissions() {
    let app = TestApp::spawn().await;
    let tenant_id = create_tenant(&app, "owner").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;

    let memberships = owner.session.get("/tenant").await;
    assert_eq!(memberships.status, StatusCode::OK, "{}", memberships.body);
    let membership = memberships
        .body
        .as_array()
        .expect("memberships")
        .iter()
        .find(|membership| membership["tenant_id"] == tenant_id.to_string())
        .expect("the owner belongs to the tenant")
        .clone();
    assert_eq!(membership["role_name"], OWNER_ROLE);
    let permissions = membership["permissions"].as_array().expect("permissions");
    for expected_permission in ["canvas:read", "tenant:delete"] {
        assert!(
            permissions
                .iter()
                .any(|permission| permission == expected_permission),
            "the owner role carries {expected_permission}: {permissions:?}"
        );
    }
}
