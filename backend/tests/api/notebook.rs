use crate::support::TestApp;
use crate::support::canvas_builder::{column_names, create_canvas, create_sql_cell, string_field};
use crate::support::constants::{DEMO_ORDER_COUNT, ORDER_COUNT_QUERY, TOP_CATEGORIES_QUERY};
use reqwest::StatusCode;
use serde_json::{Value, json};

async fn notebook_cell_names(app: &TestApp, notebook_id: &str) -> Vec<String> {
    let notebook = app.get(&format!("/notebook/{notebook_id}/cells")).await;
    assert_eq!(notebook.status, StatusCode::OK, "{}", notebook.body);
    notebook.body["cells"]
        .as_array()
        .expect("notebook cells")
        .iter()
        .map(|cell| string_field(cell, "name"))
        .collect()
}

#[tokio::test]
async fn creating_a_canvas_creates_its_notebook_and_dashboard() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;

    let canvas = create_canvas(&app, "Sales overview").await;

    let fetched = app.get(&format!("/canvas/{}", canvas.canvas_id)).await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.body["name"], "Sales overview");
    assert_eq!(fetched.body["status"], "ACTIVE");
    assert!(
        notebook_cell_names(&app, &canvas.notebook_id)
            .await
            .is_empty()
    );
    assert_eq!(canvas.dashboard["layout"], Value::Null);
}

#[tokio::test]
async fn sql_cells_can_be_created_updated_and_deleted() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    let connection = app.create_sqlite_demo_connection().await;
    let connection_id = string_field(&connection, "id");
    let canvas = create_canvas(&app, "Cell lifecycle").await;

    let cell = create_sql_cell(
        &app,
        &canvas.notebook_id,
        &connection_id,
        "Order count",
        ORDER_COUNT_QUERY,
    )
    .await;
    let cell_path = format!("/cell/{}", string_field(&cell, "id"));
    assert_eq!(cell["cell_type"], "SQL");

    let mut renamed_cell = app.get(&cell_path).await.body;
    renamed_cell["name"] = json!("Orders in total");
    let updated = app.put(&cell_path, renamed_cell).await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.body);
    assert_eq!(app.get(&cell_path).await.body["name"], "Orders in total");

    let deleted = app.delete(&cell_path).await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert_eq!(app.get(&cell_path).await.status, StatusCode::NOT_FOUND);
    assert!(
        notebook_cell_names(&app, &canvas.notebook_id)
            .await
            .is_empty()
    );
}

#[tokio::test]
async fn cells_can_be_reordered() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    let connection = app.create_sqlite_demo_connection().await;
    let connection_id = string_field(&connection, "id");
    let canvas = create_canvas(&app, "Reordering").await;
    for cell_name in ["First", "Second", "Third"] {
        create_sql_cell(
            &app,
            &canvas.notebook_id,
            &connection_id,
            cell_name,
            ORDER_COUNT_QUERY,
        )
        .await;
    }
    let third_cell = app
        .get(&format!("/notebook/{}/cells", canvas.notebook_id))
        .await
        .body["cells"][2]
        .clone();

    let reordered = app
        .patch(
            &format!("/cell/{}/reorder", string_field(&third_cell, "id")),
            json!({ "movement_type": "UP" }),
        )
        .await;

    assert_eq!(reordered.status, StatusCode::OK, "{}", reordered.body);
    assert_eq!(
        notebook_cell_names(&app, &canvas.notebook_id).await,
        ["First", "Third", "Second"]
    );
}

#[tokio::test]
async fn sql_cells_execute_their_query() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    let connection = app.create_sqlite_demo_connection().await;
    let connection_id = string_field(&connection, "id");
    let canvas = create_canvas(&app, "Execution").await;
    let cell = create_sql_cell(
        &app,
        &canvas.notebook_id,
        &connection_id,
        "Top categories",
        TOP_CATEGORIES_QUERY,
    )
    .await;

    let executed = app
        .post(
            &format!("/cell/{}/execute", string_field(&cell, "id")),
            json!({}),
        )
        .await;

    assert_eq!(executed.status, StatusCode::OK, "{}", executed.body);
    assert_eq!(column_names(&executed.body), ["category", "product_count"]);
    assert_eq!(executed.body["data"].as_array().expect("rows").len(), 5);
}

#[tokio::test]
async fn execute_preview_runs_the_selected_sql() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    let connection = app.create_sqlite_demo_connection().await;
    let connection_id = string_field(&connection, "id");
    let canvas = create_canvas(&app, "Selected SQL").await;
    let cell = create_sql_cell(
        &app,
        &canvas.notebook_id,
        &connection_id,
        "Top categories",
        TOP_CATEGORIES_QUERY,
    )
    .await;

    let preview = app
        .post(
            &format!("/cell/{}/execute-preview", string_field(&cell, "id")),
            json!({
                "content": ORDER_COUNT_QUERY,
                "cell_type": "SQL",
            }),
        )
        .await;

    assert_eq!(preview.status, StatusCode::OK, "{}", preview.body);
    assert_eq!(
        preview.body["data"],
        json!([{ "order_count": DEMO_ORDER_COUNT }])
    );
}
