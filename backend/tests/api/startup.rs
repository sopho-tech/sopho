use crate::support::TestApp;
use crate::support::canvas_builder::string_field;
use crate::support::constants::{DEMO_CANVAS_NAME, OWNER_ROLE, TENANT_VISIBILITY};
use crate::support::records::{insert_canvas, insert_notebook, insert_sql_cell};
use crate::support::tenants::{create_tenant, join_tenant};
use reqwest::StatusCode;
use serde_json::Value;
use uuid::Uuid;

async fn admin_memberships(app: &TestApp) -> Vec<Value> {
    let memberships = app.get("/tenant").await;
    assert_eq!(memberships.status, StatusCode::OK, "{}", memberships.body);
    memberships.body.as_array().expect("memberships").clone()
}

#[tokio::test]
async fn a_fresh_database_boots_into_a_usable_default_tenant() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;

    let memberships = admin_memberships(&app).await;
    assert_eq!(
        memberships.len(),
        1,
        "the admin owns exactly the default tenant"
    );
    assert_eq!(memberships[0]["role_name"], OWNER_ROLE);
    assert!(
        memberships[0]["permissions"]
            .as_array()
            .expect("permissions")
            .iter()
            .any(|permission| permission == "canvas:create")
    );

    let canvases = app.get("/canvas?page=0&page_size=50").await;
    assert_eq!(canvases.status, StatusCode::OK, "{}", canvases.body);
    assert!(
        canvases.body["data"]
            .as_array()
            .expect("canvases")
            .iter()
            .any(|canvas| string_field(canvas, "name") == DEMO_CANVAS_NAME),
        "demo data is seeded into the default tenant"
    );
}

#[tokio::test]
async fn booting_twice_changes_nothing() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let first_boot = admin_memberships(&app).await;

    app.restart_initialization().await;
    let second_boot = admin_memberships(&app).await;

    assert_eq!(second_boot.len(), 1);
    assert_eq!(second_boot[0]["tenant_id"], first_boot[0]["tenant_id"]);
}

#[tokio::test]
async fn existence_checks_report_missing_rows_as_missing() {
    let app = TestApp::spawn().await;
    let tenant_id = create_tenant(&app, "existence").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let canvas_id =
        insert_canvas(&app, tenant_id, "canvas", owner.user.id, TENANT_VISIBILITY).await;
    let notebook_id = insert_notebook(&app, tenant_id, canvas_id, "notebook").await;
    let cell_id = insert_sql_cell(&app, tenant_id, notebook_id, "cell", "SELECT 1").await;
    let unknown_id = Uuid::new_v4();

    for (existing_path, missing_path) in [
        (
            format!("/notebook/{notebook_id}"),
            format!("/notebook/{unknown_id}"),
        ),
        (format!("/cell/{cell_id}"), format!("/cell/{unknown_id}")),
    ] {
        assert_eq!(
            owner.session.get(&existing_path).await.status,
            StatusCode::OK
        );
        assert_eq!(
            owner.session.get(&missing_path).await.status,
            StatusCode::NOT_FOUND,
            "{missing_path} is reported as missing, not as a failure"
        );
    }
}
