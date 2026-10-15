use crate::support::TestApp;
use crate::support::canvas_builder::{
    axis_chart_content, create_canvas, create_chart_cell, create_sql_cell, pie_chart_content,
    string_field,
};
use crate::support::constants::TOP_CATEGORIES_QUERY;
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn dashboard_layout_round_trips() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    let connection = app.create_sqlite_demo_connection().await;
    let connection_id = string_field(&connection, "id");
    let canvas = create_canvas(&app, "Dashboard layout").await;
    let sql_cell = create_sql_cell(
        &app,
        &canvas.notebook_id,
        &connection_id,
        "Top categories",
        TOP_CATEGORIES_QUERY,
    )
    .await;
    let sql_cell_id = string_field(&sql_cell, "id");
    let bar_chart = create_chart_cell(
        &app,
        &canvas.notebook_id,
        &connection_id,
        "Bar chart",
        axis_chart_content("BAR", &sql_cell_id),
    )
    .await;
    let pie_chart = create_chart_cell(
        &app,
        &canvas.notebook_id,
        &connection_id,
        "Pie chart",
        pie_chart_content(&sql_cell_id),
    )
    .await;
    let layout = json!([
        {
            "cell_id": bar_chart["id"],
            "notebook_id": canvas.notebook_id,
            "x_position": 0,
            "y_position": 0,
            "x_size": 6,
            "y_size": 4,
        },
        {
            "cell_id": pie_chart["id"],
            "notebook_id": canvas.notebook_id,
            "x_position": 6,
            "y_position": 0,
            "x_size": 6,
            "y_size": 4,
        },
    ]);
    let dashboard_path = format!("/dashboard/{}", string_field(&canvas.dashboard, "id"));
    let mut dashboard = canvas.dashboard.clone();
    dashboard["layout"] = layout.clone();

    let updated = app.put(&dashboard_path, dashboard).await;

    assert_eq!(updated.status, StatusCode::OK, "{}", updated.body);
    assert_eq!(app.get(&dashboard_path).await.body["layout"], layout);
}

#[tokio::test]
async fn deleting_a_canvas_removes_it() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let canvas = create_canvas(&app, "Short lived").await;
    let canvas_path = format!("/canvas/{}", canvas.canvas_id);

    let deleted = app.delete(&canvas_path).await;

    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let fetched = app.get(&canvas_path).await;
    assert_eq!(fetched.status, StatusCode::NOT_FOUND);
    assert_eq!(fetched.body["error"], "Canvas not found");
}
